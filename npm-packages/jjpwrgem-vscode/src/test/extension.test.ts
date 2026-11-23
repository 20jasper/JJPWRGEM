import * as assert from "assert";
import * as vscode from "vscode";
import * as fs from "fs";
import * as os from "os";
import * as path from "path";

suite("Extension Smoke Tests", function () {
  this.timeout(15000);

  let whenReady: Promise<void>;

  suiteSetup(async () => {
    const extension = vscode.extensions.getExtension<{
      whenReady: Promise<void>;
    }>("20jasper.jjpwrgem-vscode")!;
    if (!extension.isActive) {
      await extension.activate();
    }
    whenReady = extension.exports.whenReady;
  });

  test("Extension should be present and activate", async () => {
    const extension = vscode.extensions.getExtension(
      "20jasper.jjpwrgem-vscode",
    );
    assert.ok(extension, "jjpwrgem-vscode extension should be installed");
    assert.ok(extension.isActive, "Extension should be active");
  });

  test("Built-in JSON language features should be disabled", async () => {
    assert.strictEqual(
      vscode.extensions.getExtension("vscode.json-language-features"),
      undefined,
      "built-in json extension should be disabled",
    );
  });

  test("Open a JSON file on disk and format it", async () => {
    await whenReady;

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
      formatted.trim(),
      '{ "foo": "bar" }',
      "Formatted document should contain expected JSON keys",
    );

    await vscode.commands.executeCommand("workbench.action.closeActiveEditor");
    fs.rmSync(tmpDir, { recursive: true, force: true });
  });

  test("Open a JSONC file and get jsonc diagnostics", async () => {
    await whenReady;

    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "jjpwrgem-smoke-"));
    const filePath = path.join(tmpDir, "smoke.jsonc");
    const content = `{
  // line
  "a": 1, /* block */
  "b": ,
}
`;
    fs.writeFileSync(filePath, content, "utf8");

    const uri = vscode.Uri.file(filePath);
    const doc = await vscode.workspace.openTextDocument(uri);
    const editor = await vscode.window.showTextDocument(doc);

    assert.strictEqual(
      editor.document.languageId,
      "jsonc",
      "File should be recognized as jsonc",
    );

    const getJjpDiagnostics = () =>
      vscode.languages
        .getDiagnostics(uri)
        .filter((d) => d.source === "jjpwrgem");

    let diagnostics = getJjpDiagnostics();
    for (let i = 0; i < 100 && diagnostics.length === 0; i++) {
      await new Promise((resolve) => setTimeout(resolve, 100));
      diagnostics = getJjpDiagnostics();
    }

    assert.strictEqual(
      diagnostics.length,
      1,
      "Only the missing value should be reported",
    );
    assert.strictEqual(
      diagnostics[0]!.message,
      "expected json value, found `,`",
    );

    assert.strictEqual(
      vscode.languages.getDiagnostics(uri).length,
      1,
      "No diagnostics besides jjpwrgem's should be present",
    );

    await vscode.commands.executeCommand("workbench.action.closeActiveEditor");
    fs.rmSync(tmpDir, { recursive: true, force: true });
  });
});
