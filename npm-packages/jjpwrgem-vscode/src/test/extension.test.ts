import * as assert from "assert";
import * as vscode from "vscode";
import {
  describeStartError,
  INSTALL_ACTION,
  SETTINGS_ACTION,
} from "../startError";
import { activateExtension, formatDocument, withTempFile } from "./helpers";

suite("Extension Smoke Tests", function () {
  this.timeout(15000);

  suiteSetup(activateExtension);

  test("extension is installed and active", () => {
    const extension = vscode.extensions.getExtension(
      "20jasper.jjpwrgem-vscode",
    );
    assert.ok(extension, "jjpwrgem-vscode extension should be installed");
    assert.ok(extension.isActive, "extension should be active");
  });

  test("built-in JSON language features are disabled", () => {
    assert.strictEqual(
      vscode.extensions.getExtension("vscode.json-language-features"),
      undefined,
    );
  });

  test("formats a json file", async () => {
    await withTempFile("smoke.json", `{"foo":"bar"}`, async (uri) => {
      const doc = await vscode.workspace.openTextDocument(uri);
      assert.strictEqual(doc.languageId, "json");

      const edit = new vscode.WorkspaceEdit();
      edit.set(uri, await formatDocument(uri));
      await vscode.workspace.applyEdit(edit);

      assert.strictEqual(doc.getText().trim(), '{ "foo": "bar" }');
    });
  });

  test("reports jsonc diagnostics for a missing value", async () => {
    const content = `{
  // line
  "a": 1, /* block */
  "b": ,
}
`;
    await withTempFile("smoke.jsonc", content, async (uri) => {
      const doc = await vscode.workspace.openTextDocument(uri);
      await vscode.window.showTextDocument(doc);
      assert.strictEqual(doc.languageId, "jsonc");

      const jjpDiagnostics = () =>
        vscode.languages
          .getDiagnostics(uri)
          .filter((d) => d.source === "jjpwrgem");

      const diagnostics = await new Promise<vscode.Diagnostic[]>(
        (resolve, reject) => {
          const check = () => {
            const found = jjpDiagnostics();
            if (found.length > 0) {
              clearTimeout(timer);
              subscription.dispose();
              resolve(found);
            }
          };
          const timer = setTimeout(() => {
            subscription.dispose();
            reject(new Error("no jjpwrgem diagnostics"));
          }, 10000);
          const subscription = vscode.languages.onDidChangeDiagnostics(check);
          check();
        },
      );

      assert.deepStrictEqual(
        diagnostics.map((d) => d.message),
        ["expected json value, found `,`"],
      );
      assert.strictEqual(
        vscode.languages.getDiagnostics(uri).length,
        1,
        "only jjpwrgem diagnostics should be present",
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
    {
      name: "non-Error rejection",
      command: "jjp",
      custom: false,
      error: "boom",
      message:
        "Could not start the jjpwrgem language server: boom. Is jjp installed and on your PATH?",
      actions: [INSTALL_ACTION],
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

  test("action labels are what the user sees", () => {
    assert.strictEqual(INSTALL_ACTION, "Installation instructions");
    assert.strictEqual(SETTINGS_ACTION, "Open settings");
  });
});
