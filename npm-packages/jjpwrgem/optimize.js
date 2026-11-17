// vendored from esbuild v0.28.2
const fs = require("fs");
const os = require("os");
const path = require("path");

/** @returns {boolean} */
function isYarn() {
  const { npm_config_user_agent } = process.env;
  if (npm_config_user_agent) {
    return /\byarn\//.test(npm_config_user_agent);
  }
  return false;
}

/**
 * @param {string} binPath
 * @param {string} toPath
 * @returns {void}
 */
function maybeOptimizePackage(binPath, toPath) {
  // This package contains a "bin/esbuild" JavaScript file that finds and runs
  // the appropriate binary executable. However, this means that running the
  // "esbuild" command runs another instance of "node" which is way slower than
  // just running the binary executable directly.
  //
  // Here we optimize for this by replacing the JavaScript file with the binary
  // executable at install time. This optimization does not work on Windows
  // because on Windows the binary executable must be called "esbuild.exe"
  // instead of "esbuild".
  //
  // This also doesn't work with Yarn both because of lack of support for binary
  // files in Yarn 2+ (see https://github.com/yarnpkg/berry/issues/882) and
  // because Yarn (even Yarn 1?) may run the same install scripts in the same
  // place multiple times from different platforms, especially when people use
  // Docker. Avoid idempotency issues by just not optimizing when using Yarn.
  //
  // This optimization also doesn't apply when npm's "--ignore-scripts" flag is
  // used since in that case this install script will not be run.
  if (os.platform() !== "win32" && !isYarn()) {
    const tempPath = path.join(path.dirname(toPath), "bin-esbuild");
    try {
      // First link the binary with a temporary file. If this fails and throws an
      // error, then we'll just end up doing nothing. This uses a hard link to
      // avoid taking up additional space on the file system.
      fs.linkSync(binPath, tempPath);

      // Then use rename to atomically replace the target file with the temporary
      // file. If this fails and throws an error, then we'll just end up leaving
      // the temporary file there, which is harmless.
      fs.renameSync(tempPath, toPath);

      // If this install script is being re-run, then "renameSync" will fail
      // since the underlying inode is the same (it just returns without doing
      // anything, and without throwing an error). In that case we should remove
      // the file manually.
      fs.unlinkSync(tempPath);
    } catch {
      // Ignore errors here since this optimization is optional
    }
  }
}

module.exports = { maybeOptimizePackage };
