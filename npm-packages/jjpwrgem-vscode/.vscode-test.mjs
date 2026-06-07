import { defineConfig } from "@vscode/test-cli";

export default defineConfig({
  files: "out/test/**/*.test.js",
  launchArgs: [
    "--skip-welcome",
    "--disable-extensions",
    "--disable-extension",
    "vscode.json-language-features",
    "--skip-release-notes",
  ],
  settings: {
    "editor.defaultFormatter": "20jasper.jjpwrgem-vscode",
  },
});
