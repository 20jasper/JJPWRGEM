---
cargo/jjpwrgem-parse: minor (Changed)
---

`Document::parse` and `validate_str` now take `SyntaxOptions`, which can allow trailing commas. `SyntaxOptions::JSON` keeps strict json.
