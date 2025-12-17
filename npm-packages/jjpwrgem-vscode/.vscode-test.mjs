import { defineConfig } from "@vscode/test-cli";

export default defineConfig({
  files: "out/test/**/*.test.js",
  launchArgs: [
    "./test-fixtures/test.code-workspace",
    "--skip-welcome",
    "--disable-extensions",
    "--skip-release-notes",
    "--enable-proposed-api",
  ],
  settings: {
    "editor.defaultFormatter": "20jasper.jjpwrgem-vscode",
  },
});
