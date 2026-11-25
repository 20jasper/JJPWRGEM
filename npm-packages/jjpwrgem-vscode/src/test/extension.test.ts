import * as assert from "assert";
import * as vscode from "vscode";
import {
  describeStartError,
  INSTALL_ACTION,
  SETTINGS_ACTION,
} from "../startError";
import {
  activateExtension,
  formatDocument,
  jjpwrgemDiagnostics,
  withTempFile,
} from "./helpers";

suite("Extension Smoke Tests", function () {
  this.timeout(15000);

  suiteSetup(activateExtension);

  test("Extension should be present and activate", () => {
    const extension = vscode.extensions.getExtension(
      "20jasper.jjpwrgem-vscode",
    );
    assert.ok(extension, "jjpwrgem-vscode extension should be installed");
    assert.ok(extension.isActive, "Extension should be active");
  });

  test("Built-in JSON language features should be disabled", () => {
    assert.strictEqual(
      vscode.extensions.getExtension("vscode.json-language-features"),
      undefined,
      "built-in json extension should be disabled",
    );
  });

  test("Open a JSON file on disk and format it", async () => {
    await withTempFile("smoke.json", `{"foo":"bar"}`, async (uri) => {
      const doc = await vscode.workspace.openTextDocument(uri);
      assert.strictEqual(
        doc.languageId,
        "json",
        "File should be recognized as json",
      );

      const edit = new vscode.WorkspaceEdit();
      edit.set(uri, await formatDocument(uri));
      await vscode.workspace.applyEdit(edit);

      assert.strictEqual(
        doc.getText().trim(),
        '{ "foo": "bar" }',
        "Formatted document should contain expected JSON keys",
      );
    });
  });

  test("Open a JSONC file and get jsonc diagnostics", async () => {
    const content = `{
  // line
  "a": 1, /* block */
  "b": ,
}
`;
    await withTempFile("smoke.jsonc", content, async (uri) => {
      const doc = await vscode.workspace.openTextDocument(uri);
      await vscode.window.showTextDocument(doc);
      assert.strictEqual(
        doc.languageId,
        "jsonc",
        "File should be recognized as jsonc",
      );

      const diagnostics = await jjpwrgemDiagnostics(uri);

      assert.deepStrictEqual(
        diagnostics.map((d) => d.message),
        ["expected json value, found `,`"],
        "Only the missing value should be reported",
      );
      assert.strictEqual(
        vscode.languages.getDiagnostics(uri).length,
        1,
        "No diagnostics besides jjpwrgem's should be present",
      );
    });
  });
});

suite("describeStartError", () => {
  const cases = [
    {
      name: "path unset",
      command: "jjp",
      custom: false,
      error: new Error("spawn jjp ENOENT"),
      message:
        "Could not start the jjpwrgem language server: spawn jjp ENOENT. Is jjp installed and on your PATH?",
      actions: [INSTALL_ACTION],
    },
    {
      name: "path set",
      command: "/opt/bad/jjp",
      custom: true,
      error: new Error("spawn /opt/bad/jjp ENOENT"),
      message:
        'Could not start the jjpwrgem language server with "/opt/bad/jjp": spawn /opt/bad/jjp ENOENT. Check the jjpwrgem.path setting.',
      actions: [SETTINGS_ACTION],
    },
  ];

  for (const { name, command, custom, error, message, actions } of cases) {
    test(name, () => {
      assert.deepStrictEqual(describeStartError(command, custom, error), {
        message,
        actions,
      });
    });
  }
});
