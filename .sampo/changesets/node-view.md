---
cargo/jjpwrgem-parse: minor (Changed)
---

read documents through a `Node` view that carries its document. `Document::root` returns a `Node`, and `Node::kind` gives typed access:

- `Kind::Null`
- `Kind::Bool`
- `Kind::String` with `Str::raw`, `Str::unescape` and `Str::span`
- `Kind::Number` with `Number::mantissa`, `Number::exponent`, `Number::span` and `Number::text`, which returns a `NumberText` with `NumberText::mantissa` and `NumberText::exponent`
- `Kind::Array` with `Array::items`, `Array::len`, `Array::is_empty` and `Array::node`
- `Kind::Object` with `Object::members`, `Object::get` and `Object::node`

`Node` also has `Node::span`, `Node::text`, `Node::traverse`, `Node::path_at` and `Node::member_span`.

`Value::children`, `Value::child_at`, `Value::path_at`, `Value::traverse` and `Value::member_span` are no longer public, use `Node` instead.
