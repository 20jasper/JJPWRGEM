import * as assert from "assert";
import * as vscode from "vscode";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";
import { spawnSync } from "child_process";
import { getSetting } from "../settings";

// `jjp` is off PATH here (see .vscode-test.mjs), so this only passes if `jjpwrgem.path` is used
suite("Custom jjpwrgem.path", function () {
  this.timeout(15000);

  test("formats using the configured executable", async () => {
    assert.ok(spawnSync("jjp").error, "jjp is on PATH");
    assert.notStrictEqual(getSetting("jjpwrgem.path"), "jjp");

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
