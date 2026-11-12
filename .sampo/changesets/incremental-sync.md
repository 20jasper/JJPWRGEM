---
cargo/jjpwrgem-lsp: patch (Performance)
cargo/jjpwrgem: patch (Performance)
---

use incremental document sync in the LSP with the line-index crate for edit position conversion, avoiding a full-document transfer on every edit. In LSP diagnostics benchmarks, peak memory is up to around 25% lower on the largest document.
