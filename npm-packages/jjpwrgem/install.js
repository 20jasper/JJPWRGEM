#!/usr/bin/env node

const { join } = require("path");
const { install, getPackage } = require("./binary");
const { maybeOptimizePackage } = require("./optimize");
const { artifactDownloadUrl } = require("./package.json");

const isPnpm = /\bpnpm\//.test(process.env.npm_config_user_agent || "");

Promise.resolve(install(false)).then(() => {
  if (!artifactDownloadUrl) return;
  if (!isPnpm) {
    const pkg = getPackage();
    maybeOptimizePackage(
      join(pkg.installDirectory, pkg.binaries.jjp),
      join(__dirname, "run-jjp.js"),
    );
  }
  if (process.platform === "win32" || isPnpm) {
    console.error(require("./notice"));
  }
});
