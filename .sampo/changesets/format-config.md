---
cargo/jjpwrgem-parse: patch (Added)
cargo/jjpwrgem: patch (Changed)
---

add `format::FormatConfig` and its conversion into a `FormatRequest`, shared by every front end. Format option conflicts exit with code 2 and a usage line. An explicit `--preferred-width` conflicts even at its default
