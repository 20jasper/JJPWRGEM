---
cargo/lspresso: patch (Added)
---

add granular timings via Client::timings. Splits up time spent receiving, processing, and responding to requests. Real clients vary in speed so split out the language server's work from IO
