// Derived from cargo-dist (https://github.com/axodotdev/cargo-dist)
// Copyright (c) 2022-2024 Axo Developer Co.
// Licensed under MIT OR Apache-2.0

const { Package } = require("./binary-install");
const os = require("os");
const libc = require("detect-libc");

const error = (msg) => {
  console.error(msg);
  process.exit(1);
};

const {
  name,
  artifactDownloadUrl,
  supportedPlatforms,
  glibcMinimum,
} = require("./package.json");

const getPlatform = () => {
  const rawOsType = os.type();
  const rawArchitecture = os.arch();

  // We want to use rust-style target triples as the canonical key
  // for a platform, so translate the "os" library's concepts into rust ones
  let osType = "";
  switch (rawOsType) {
    case "Windows_NT":
      osType = "pc-windows-msvc";
      break;
    case "Darwin":
      osType = "apple-darwin";
      break;
    case "Linux":
      osType = "unknown-linux-gnu";
      break;
  }

  let arch = "";
  switch (rawArchitecture) {
    case "x64":
      arch = "x86_64";
      break;
    case "arm64":
      arch = "aarch64";
      break;
  }

  if (rawOsType === "Linux") {
    if (libc.familySync() !== "glibc") {
      error(
        `jjp only supports glibc on Linux, found ${libc.familySync() || "an unknown libc"}.`,
      );
    }
    const glibcVersion = libc.versionSync();
    const [major, minor] = glibcVersion
      ? glibcVersion.split(".").map(Number)
      : [];
    if (
      glibcVersion &&
      (major < glibcMinimum.major ||
        (major === glibcMinimum.major && minor < glibcMinimum.series))
    ) {
      error(
        `jjp needs glibc ${glibcMinimum.major}.${glibcMinimum.series} or newer, found ${major}.${minor}.`,
      );
    }
  }

  // Assume the above succeeded and build a target triple to look things up with.
  // If any of it failed, this lookup will fail and we'll handle it like normal.
  let targetTriple = `${arch}-${osType}`;
  let platform = supportedPlatforms[targetTriple];

  if (!platform) {
    error(
      `Platform with type "${rawOsType}" and architecture "${rawArchitecture}" is not supported by ${name}.\nYour system must be one of the following:\n\n${Object.keys(
        supportedPlatforms,
      ).join(",")}`,
    );
  }

  return platform;
};

const getPackage = () => {
  const platform = getPlatform();
  const url = `${artifactDownloadUrl}/${platform.artifactName}`;
  let filename = platform.artifactName;
  let ext = platform.zipExt;
  let binary = new Package(platform, name, url, filename, ext, platform.bins);

  return binary;
};

const install = (suppressLogs) => {
  if (!artifactDownloadUrl || artifactDownloadUrl.length === 0) {
    console.warn("in demo mode, not installing binaries");
    return;
  }
  const pkg = getPackage();

  return pkg.install(suppressLogs);
};

const run = (binaryName) => {
  const pkg = getPackage();

  pkg.run(binaryName);
};

module.exports = {
  install,
  run,
  getPackage,
};
