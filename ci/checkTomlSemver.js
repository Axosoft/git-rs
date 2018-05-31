var fs = require('fs');
var readline = require('readline');

const versionRegex = /^version = "(.+)"$/;
const tagName = process.env.TRAVIS_TAG || process.env.APPVEYOR_REPO_TAG_NAME;

const lineReader = readline.createInterface({
  input: fs.createReadStream('gitrs_server/Cargo.toml'),
  output: null,
  console: false
});

let foundVersion;

lineReader.on('line', (line) => {
  const capture = versionRegex.exec(line);
  if (capture  !== null) {
    foundVersion = capture[1];
  }
});

lineReader.on('close', () => {
  if (foundVersion && foundVersion === tagName) {
    process.exit(0);
  } else {
    process.exit(1);
  }
});
