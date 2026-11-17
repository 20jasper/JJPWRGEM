#!/usr/bin/env node

const { existsSync } = require("fs");
const { join } = require("path");
const { spawnSync } = require("child_process");

const binPath = join(
  __dirname,
  "node_modules",
  ".bin_real",
  process.platform === "win32" ? "jjp.exe" : "jjp",
);

if (existsSync(binPath)) {
  const result = spawnSync(binPath, process.argv.slice(2), {
    stdio: "inherit",
  });
  if (result.error) {
    console.error(result.error);
    process.exit(1);
  }
  process.exit(result.status);
} else {
  console.error(require("./notice"));
  require("./binary").run("jjp");
}
