import * as assert from "assert";
import * as vscode from "vscode";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";
import { readFile } from "fs";
import { promisify } from "util";

suite("Extension Smoke Tests", function () {
  this.timeout(15000);

  test("Extension should be present and activate", async () => {
    const extension = vscode.extensions.all.find(
      (ext) => ext.packageJSON && ext.packageJSON.name === "jjpwrgem-vscode",
    );
    assert.ok(extension, "jjpwrgem-vscode extension should be installed");
    if (!extension!.isActive) {
      await extension!.activate();
    }
    assert.ok(extension!.isActive, "Extension should be active");
  });

  test("Open a JSON file on disk and format it", async () => {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "jjpwrgem-smoke-"));
    const filePath = path.join(tmpDir, "smoke.json");
    const content = `{"foo":"bar"}`;
    fs.writeFileSync(filePath, content, "utf8");

    const uri = vscode.Uri.file(filePath);
    const doc = await vscode.workspace.openTextDocument(uri);
    const editor = await vscode.window.showTextDocument(doc);

    assert.strictEqual(
      editor.document.languageId,
      "json",
      "File should be recognized as json",
    );

    const edits = await vscode.commands.executeCommand<
      vscode.TextEdit[] | undefined
    >("vscode.executeFormatDocumentProvider", doc.uri, {
      tabSize: 2,
      insertSpaces: true,
    });
    if (edits && edits.length > 0) {
      const workspaceEdit = new vscode.WorkspaceEdit();
      workspaceEdit.set(doc.uri, edits);
      await vscode.workspace.applyEdit(workspaceEdit);
      await doc.save();
    }

    const formatted = editor.document.getText();
    assert.equal(
      formatted,
      '{\n  "foo": "bar"\n}\n',
      "Formatted document should contain expected JSON keys",
    );

    try {
      await vscode.commands.executeCommand(
        "workbench.action.closeActiveEditor",
      );
    } catch {}
    fs.rmSync(tmpDir, { recursive: true, force: true });
  });
});
