---
cargo/jjpwrgem-parse: patch (Added)
---

add `Document` accessors to get data without ranges:

- `Document::as_null`
- `Document::as_bool`
- `Document::as_str`
- `Document::as_array`
- `Document::as_object`
- `Document::as_number`
