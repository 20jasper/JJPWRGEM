---
cargo/jjpwrgem: patch (Changed)
cargo/jjpwrgem-parse: patch (Changed)
---

Format option combinations are now validated by `FormatConfig::validate` in `jjpwrgem-parse`. Invalid combinations of `jjp format` flags report the rule, for example `error: uglify cannot be combined with a preferred width`, followed by the usage line, and exit with code 2.
