---
cargo/jjpwrgem-lsp: minor (Performance)
cargo/jjpwrgem-parse: patch (Performance)
---

use incremental document sync in the LSP, avoiding a full-document transfer on every edit. In the LSP diagnostics benchmark (small edit at the end of the document), twitter improves from 1.9ms to 670µs, citm_catalog from 3.8ms to 1.85ms, and canada from 10.5ms to 5.9ms. Peak RSS for canada drops from 29.8MB to 22.6MB.
