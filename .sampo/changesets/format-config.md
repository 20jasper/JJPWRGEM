---
cargo/jjpwrgem-parse: patch (Added)
cargo/jjpwrgem: patch (Changed)
---

add `format::FormatConfig` validation shared by every front end; format option conflicts exit with code 2 and a usage line, and an explicit `--preferred-width` conflicts even at its default
