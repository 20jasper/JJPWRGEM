#!/usr/bin/env node

const { chmodSync, copyFileSync, linkSync, unlinkSync } = require("fs");
const { join } = require("path");
const { install, getPackage } = require("./binary");
const { artifactDownloadUrl } = require("./package.json");

const place = (source, target) => {
  try {
    unlinkSync(target);
  } catch {}
  try {
    linkSync(source, target);
  } catch {
    copyFileSync(source, target);
  }
  if (process.platform !== "win32") {
    chmodSync(target, 0o755);
  }
};

Promise.resolve(install(false))
  .then(() => {
    if (!artifactDownloadUrl) return;
    const pkg = getPackage();
    place(
      join(pkg.installDirectory, pkg.binaries.jjp),
      join(__dirname, "bin", "jjp.exe"),
    );
  })
  .catch((e) => {
    console.error(`Failed to install jjp: ${e.message}`);
    process.exit(1);
  });
