const checksum = require('checksum');
const fs = require('fs');
const mkdirp = require('mkdirp');
const path = require('path');
const request = require('request');
const tar = require('tar');
const url = require('url');
const zip = require('cross-zip');

const config = {
  buildDirectory: '',
  expectedChecksum: '',
  gitRsBinaryName: '',
  gitRsBinaryPath: '',
  source: '',
  target: process.env.TARGET,
  tempFile: '',
  vendorDirectoryName: 'vendor',
  vendorDirectoryPath: ''
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

config.buildDirectory = path.join(process.cwd(), 'git-rs');
config.gitRsBinaryPath = path.join(process.cwd(), 'gitrs_server', 'target', 'release', config.gitRsBinaryName);
config.vendorDirectoryPath = path.join(config.buildDirectory, config.vendorDirectoryName);
config.tempFile = path.join(config.buildDirectory, 'git.tar.gz');

const getFileChecksum = async (filePath) => new Promise((resolve) => {
  checksum.file(filePath, { algorithm: 'sha256' } , (_, hash) => resolve(hash));
});

const bundleGit = ({
  buildDirectory,
  expectedChecksum,
  gitRsBinaryName,
  gitRsBinaryPath,
  source,
  target,
  tempFile,
  vendorDirectoryName,
  vendorDirectoryPath
}) => {
  mkdirp.sync(buildDirectory);

  const options = {
    url: source
  };
  const req = request.get(options);
  req.pipe(fs.createWriteStream(tempFile));

  req.on('error', (error) => {
    console.log('Failed to fetch Git binaries');
    process.exit(1);
  });

  req.on('response', (res) => {
    if (res.statusCode !== 200) {
      console.log(`Non-200 response returned from ${source.toString()} - (${res.statusCode})`);
      process.exit(1);
    }
  });

  req.on('end', async () => {
    const checksum = await getFileChecksum(tempFile, config);
    if (checksum !== expectedChecksum) {
      console.log(`Checksum validation failed. Expected ${expectedChecksum} but got ${checksum}`);
      process.exit(1);
    }

    mkdirp.sync(vendorDirectoryPath);

    fs.copyFileSync(gitRsBinaryPath, path.join(buildDirectory, gitRsBinaryName));

    try {
      await tar.extract({
        cwd: vendorDirectoryPath,
        file: tempFile
      });
    } catch (error) {
      console.log('Could not extract git archive');
      process.exit(1);
    }

    fs.unlinkSync(path.join(tempFile));

    try {
      zip.zipSync(buildDirectory, `${process.env.TARGET}.zip`);
    } catch (error) {
      console.log('Could not build git-rs archive');
      console.error(error);
      process.exit(1);
    }
  });
}

bundleGit(config);
