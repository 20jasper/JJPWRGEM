---
cargo/jjpwrgem-lsp: patch (Added)
cargo/jjpwrgem: patch (Added)
npm/jjpwrgem-vscode: patch (Added)
---

the language server and VS Code extension read `jjpwrgem.format` settings (`mode`, `preferredWidth`, `endOfLine`) from initialization options and `workspace/didChangeConfiguration`. Settings that do not apply are ignored instead of failing
