import { defineConfig } from "@vscode/test-cli";

export default defineConfig({
  files: "out/test/**/*.test.js",
  launchArgs: ["--skip-welcome", "--disable-extensions", "--skip-release-notes"],
  settings: {
    "editor.defaultFormatter": "20jasper.jjpwrgem-vscode",
  },
});
