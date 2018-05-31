const checksum = require('checksum');
const fs = require('fs');
const mkdirp = require('mkdirp');
const path = require('path');
const request = require('request');
const tar = require('tar');
const url = require('url');

const config = {
  buildDirectory: '',
  expectedChecksum: '',
  gitRsBinaryName: '',
  gitRsBinaryPath: '',
  source: '',
  target: process.env.TARGET,
  tempFile: '',
  vendorDirectory: ''
};

switch (process.env.TARGET) {
  case 'x86_64-apple-darwin':
    config.expectedChecksum = 'f92ff67688ddc9ce48ba50e9e9ed8cf49e958a697ca2571edce898a4b9dae474';
    config.source = url.parse(
      'https://github.com/desktop/dugite-native/releases/download/v2.17.1-2/dugite-native-v2.17.1-macOS.tar.gz'
    );
    config.gitRsBinaryName = 'git_server';
    break;
  case 'x86_64-unknown-linux-gnu':
    config.expectedChecksum = 'a3750dade1682d1805623661e006f842c6bbf9cc4e450ed161e49edeb2847a86';
    config.source = url.parse(
      'https://github.com/desktop/dugite-native/releases/download/v2.17.1-2/dugite-native-v2.17.1-ubuntu.tar.gz'
    );
    config.gitRsBinaryName = 'git_server';
    break;
  case 'x86_64-pc-windows-msvc':
    config.expectedChecksum = '6a7f166a8211c60d724cc23ef378a059375a67f1c352f5a44846dd0c84285f30';
    config.source = url.parse(
      'https://github.com/desktop/dugite-native/releases/download/v2.17.1-2/dugite-native-v2.17.1-win32.tar.gz'
    );
    config.gitRsBinaryName = 'git_server.exe';
    break;
}

config.buildDirectory = path.join(process.cwd(), 'build');
config.gitRsBinaryPath = path.join(process.cwd(), 'gitrs_server', 'target', 'release', config.gitRsBinaryName);
config.vendorDirectory = path.join(config.buildDirectory, 'vendor');
config.tempFile = path.join(config.buildDirectory, 'git.tar.gz');

const getFileChecksum = async (filePath) => new Promise((resolve) => {
  checksum.file(filePath, { algorithm: 'sha256' } , (_, hash) => resolve(hash));
});

const unpackFile = async (filePath, destinationPath) => tar.extract({ 
  cwd: destinationPath,
  file: filePath
});

const packBundle = async (sourcePaths, destinationFile) => tar.create(
  {
    gzip: true,
    file: destinationFile
  },
  sourcePaths
);


const bundleGit = (config) => {
  mkdirp(config.buildDirectory, (error) => {
    if (error) {
      console.log(`Could not create build directory`);
      process.exit(1);
    }
  });

  const options = {
    url: config.source
  };
  const req = request.get(options);
  req.pipe(fs.createWriteStream(config.tempFile));

  req.on('error', (error) => {
    console.log('Failed to fetch Git binaries');
    process.exit(1);
  });

  req.on('response', (res) => {
    if (res.statusCode !== 200) {
      console.log(`Non-200 response returned from ${config.source.toString()} - (${res.statusCode})`);
      process.exit(1);
    }
  });

  req.on('end', async () => {
    const checksum = await getFileChecksum(config.tempFile, config);
    if (checksum !== config.expectedChecksum) {
      console.log(`Checksum validation failed. Expected ${config.expectedChecksum} but got ${checksum}`);
      process.exit(1);
    }

    mkdirp(config.vendorDirectory, (error) => {
      if (error) {
        console.log(`Could not create ${vendor} directory to extract files to`);
        process.exit(1);
      }
    });

    fs.copyFile(config.gitRsBinaryPath, path.join(config.buildDirectory, config.gitRsBinaryName), (error) => {
      if (error) {
        console.log(`Could not copy git-rs binaries`);
        process.exit(1);
      }
    });

    try {
      await unpackFile(config.tempFile, config.vendorDirectory);
    } catch (error) {
      console.log('Could not extract git archive');
      process.exit(1);
    }

    try {
      await packBundle([config.vendorDirectory, path.join(config.buildDirectory, config.gitRsBinaryName)], `${process.env.TARGET}.tar.gz`);
    } catch (error) {
      console.log('Could not build git-rs archive');
      console.error(error);
      process.exit(1);
    }
  });
}

bundleGit(config);