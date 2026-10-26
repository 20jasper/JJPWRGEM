// @ts-check
import { defineConfig } from "@vscode/test-cli";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";

const launchArgs = [
  "--skip-welcome",
  "--disable-extensions",
  "--disable-extension",
  "vscode.json-language-features",
  "--skip-release-notes",
];
const exe = process.platform === "win32" ? "jjp.exe" : "jjp";
const pathDirs = (process.env.PATH ?? "").split(path.delimiter).filter(Boolean);
const hasJjp = (/** @type {string} */ dir) =>
  fs.existsSync(path.join(dir, exe));

/** Links the `jjp` on PATH under another name */
function installCustomJjp() {
  const dir = pathDirs.find(hasJjp);
  if (!dir) {
    throw new Error(`${exe} not found on PATH`);
  }
  const source = path.join(dir, exe);
  const target = path.join(
    fs.mkdtempSync(path.join(os.tmpdir(), "jjpwrgem-custom-bin-")),
    `custom-${exe}`,
  );
  try {
    fs.symlinkSync(fs.realpathSync(source), target);
  } catch {
    fs.copyFileSync(source, target);
  }
  return target;
}

/**
 * test-cli has no `settings` option, so seed a user data dir
 * @param {Partial<Record<import("./src/settings.js").SettingKey | "editor.defaultFormatter", string>>} settings
 */
function withSettings(settings) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "jjpwrgem-user-data-"));
  fs.mkdirSync(path.join(dir, "User"));
  fs.writeFileSync(
    path.join(dir, "User", "settings.json"),
    JSON.stringify(settings),
  );
  return [...launchArgs, `--user-data-dir=${dir}`];
}

const editor = { "editor.defaultFormatter": "20jasper.jjpwrgem-vscode" };

export default defineConfig([
  {
    label: "default",
    files: "out/src/test/extension.test.js",
    launchArgs: withSettings(editor),
  },
  {
    label: "custom-path",
    files: "out/src/test/custom-path.test.js",
    launchArgs: withSettings({
      ...editor,
      "jjpwrgem.path": installCustomJjp(),
    }),
    env: { PATH: pathDirs.filter((dir) => !hasJjp(dir)).join(path.delimiter) },
  },
]);
