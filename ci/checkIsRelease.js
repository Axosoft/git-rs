const semverRegex = /^\d+\.\d+\.+\d(-RC\d+)?$/;
const tagName = process.env.TRAVIS_TAG || process.env.APPVEYOR_REPO_TAG_NAME;
if (semverRegex.test(tagName)) {
  process.exit(0);
} else {
  process.exit(1);
}