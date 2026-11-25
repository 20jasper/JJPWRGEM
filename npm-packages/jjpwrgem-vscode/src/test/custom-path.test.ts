import * as assert from "assert";
import * as vscode from "vscode";
import { spawnSync } from "child_process";
import { type SettingKey } from "../settings";
import { activateExtension, formatDocument, withTempFile } from "./helpers";

const pathSetting = "jjpwrgem.path" satisfies SettingKey;

// `jjp` is taken off PATH, so this only passes if `jjpwrgem.path` is used
suite("Custom jjpwrgem.path", function () {
  this.timeout(15000);

  test("formats using the configured executable", async () => {
    assert.ok(spawnSync("jjp").error, "jjp is on PATH");
    assert.strictEqual(
      vscode.workspace.getConfiguration().get(pathSetting),
      process.env.JJPWRGEM_TEST_CUSTOM_PATH,
    );
    await activateExtension();

    const edits = await withTempFile(
      "smoke.json",
      `{"foo":"bar"}`,
      formatDocument,
    );

    assert.deepStrictEqual(
      edits.map((edit) => edit.newText.trim()),
      ['{ "foo": "bar" }'],
    );
  });
});
