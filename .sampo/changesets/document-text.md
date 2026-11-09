---
cargo/lspresso: minor (Changed)
---

**⚠️ breaking change:** track document text, negotiate the server's position encoding, add `did_change` for incremental edits, and include the tracked text in `did_save` when the server asks for it. `did_save` now takes only the URI.
