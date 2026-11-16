---
cargo/lspresso: patch (Added)
---

`Client::initialize` and `Client::spawn_initialized` take `impl Into<Initialize>`, allowing options to be passed backwards compatibly
