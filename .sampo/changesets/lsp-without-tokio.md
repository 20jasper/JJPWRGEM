---
cargo/jjpwrgem-lsp: minor (Performance)
cargo/jjpwrgem: patch (Performance)
---

migrate off of tower-lsp-server to a customized lsp-server, removing Tokio and avoiding intermediate deserialization. Uses 1.6x less RAM and benches are on average 1.7x faster. Binary is 31% smaller.
