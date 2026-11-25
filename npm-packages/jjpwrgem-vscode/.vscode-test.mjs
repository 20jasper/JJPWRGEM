// @ts-check
import { defineConfig } from "@vscode/test-cli";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";

const launchArgs = [
  "--skip-welcome",
  "--disable-extension=vscode.json-language-features",
  "--skip-release-notes",
];

const exe = process.platform === "win32" ? "jjp.exe" : "jjp";
const hasJjp = (/** @type {string} */ dir) =>
  fs.existsSync(path.join(dir, exe));

function customPathConfig() {
  const dirs = (process.env.PATH ?? "").split(path.delimiter);
  const source = dirs.find(hasJjp);
  if (!source) {
    throw new Error(`${exe} not found on PATH`);
  }

  const root = fs.mkdtempSync(path.join(os.tmpdir(), "jjpwrgem-custom-path-"));
  const custom = path.join(root, `custom-${exe}`);
  try {
    fs.symlinkSync(fs.realpathSync(path.join(source, exe)), custom);
  } catch {
    fs.copyFileSync(path.join(source, exe), custom);
  }

  const userDataDir = path.join(root, "user-data");
  fs.mkdirSync(path.join(userDataDir, "User"), { recursive: true });
  fs.writeFileSync(
    path.join(userDataDir, "User", "settings.json"),
    JSON.stringify({ "jjpwrgem.path": custom }),
  );

  return {
    label: "custom-path",
    files: "out/src/test/custom-path.test.js",
    launchArgs: [...launchArgs, `--user-data-dir=${userDataDir}`],
    env: {
      JJPWRGEM_TEST_CUSTOM_PATH: custom,
      PATH: dirs.filter((dir) => !hasJjp(dir)).join(path.delimiter),
    },
  };
}

export default defineConfig([
  { label: "default", files: "out/src/test/extension.test.js", launchArgs },
  customPathConfig(),
]);
