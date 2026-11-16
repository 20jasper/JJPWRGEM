---
cargo/jjpwrgem-lsp: patch (Added)
cargo/jjpwrgem: patch (Added)
---

the language server reads `format` settings (`mode`, `preferredWidth`, `endOfLine`) from initialization options and `workspace/didChangeConfiguration`, and ignores settings that do not apply instead of failing
