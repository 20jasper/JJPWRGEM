---
cargo/jjpwrgem-lsp: minor (Performance)
cargo/jjpwrgem: patch (Performance)
---

use incremental document sync in the LSP with the line-index crate for edit position conversion, avoiding a full-document transfer on every edit. In the LSP diagnostics benchmark, small edits are roughly 45–65% faster on large fixtures, with up to around 25% lower peak RSS.
