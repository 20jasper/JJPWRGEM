---
cargo/jjpwrgem-parse: minor (Changed)
---

read documents through a `Node` view that carries its document. `Document::root` returns a `Node`, and `Node::kind` gives typed access:

- `Kind::Null`
- `Kind::Bool`
- `Kind::String` with `Str::raw` and `Str::unescape`
- `Kind::Number` with `Number::mantissa` and `Number::exponent`
- `Kind::Array` with `Array::items`
- `Kind::Object` with `Object::members` and `Object::get`

`Value::children`, `Value::child_at`, `Value::path_at`, `Value::traverse` and `Value::member_span` are no longer public, use `Node` instead.
