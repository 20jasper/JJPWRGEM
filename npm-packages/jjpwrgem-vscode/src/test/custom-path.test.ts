import * as assert from "assert";
import * as vscode from "vscode";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";
import { spawnSync } from "child_process";
import { type SettingKey } from "../settings";

const exe = process.platform === "win32" ? "jjp.exe" : "jjp";
const hasJjp = (dir: string) => fs.existsSync(path.join(dir, exe));
const pathSetting = "jjpwrgem.path" satisfies SettingKey;
const setPathSetting = (value: string | undefined) =>
  vscode.workspace
    .getConfiguration()
    .update(pathSetting, value, vscode.ConfigurationTarget.Global);

// `jjp` is taken off PATH, so this only passes if `jjpwrgem.path` is used
suite("Custom jjpwrgem.path", function () {
  this.timeout(15000);

  const originalPath = process.env.PATH;
  const dirs = (originalPath ?? "").split(path.delimiter);
  let tmpDir: string;

  suiteSetup(async () => {
    const source = dirs.find(hasJjp);
    assert.ok(source, `${exe} not found on PATH`);

    tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "jjpwrgem-custom-bin-"));
    const custom = path.join(tmpDir, `custom-${exe}`);
    try {
      fs.symlinkSync(fs.realpathSync(path.join(source, exe)), custom);
    } catch {
      fs.copyFileSync(path.join(source, exe), custom);
    }

    process.env.PATH = dirs.filter((dir) => !hasJjp(dir)).join(path.delimiter);
    await setPathSetting(custom);
  });

  suiteTeardown(async () => {
    process.env.PATH = originalPath;
    await setPathSetting(undefined);
    fs.rmSync(tmpDir, { recursive: true, force: true });
  });

  test("formats using the configured executable", async () => {
    assert.ok(spawnSync("jjp").error, "jjp is on PATH");

    const extension = vscode.extensions.getExtension<{
      whenReady: Promise<void>;
    }>("20jasper.jjpwrgem-vscode")!;
    await extension.activate();
    await extension.exports.whenReady;

    const dir = fs.mkdtempSync(path.join(os.tmpdir(), "jjpwrgem-smoke-"));
    const file = path.join(dir, "smoke.json");
    fs.writeFileSync(file, `{"foo":"bar"}`);

    const edits = await vscode.commands.executeCommand<vscode.TextEdit[]>(
      "vscode.executeFormatDocumentProvider",
      vscode.Uri.file(file),
      { tabSize: 2, insertSpaces: true },
    );
    assert.ok(edits?.length, "no formatting edits");

    fs.rmSync(dir, { recursive: true, force: true });
  });
});
