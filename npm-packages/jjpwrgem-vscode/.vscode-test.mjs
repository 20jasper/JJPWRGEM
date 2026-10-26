// @ts-check
import { defineConfig } from "@vscode/test-cli";

const launchArgs = [
  "--skip-welcome",
  "--disable-extensions",
  "--disable-extension",
  "vscode.json-language-features",
  "--skip-release-notes",
];

export default defineConfig([
  { label: "default", files: "out/src/test/extension.test.js", launchArgs },
  {
    label: "custom-path",
    files: "out/src/test/custom-path.test.js",
    launchArgs,
  },
]);
