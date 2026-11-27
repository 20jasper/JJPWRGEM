---
cargo/jjpwrgem-parse: minor (Added)
---

add a `Node` view that carries its document, where `Node::kind` gives typed access:

- `Kind::Null`
- `Kind::Bool`
- `Kind::String` with `Str::raw`, `Str::unescape` and `Str::span`
- `Kind::Number` with `Number::mantissa`, `Number::exponent`, `Number::span` and `Number::text`, which returns a `NumberText` with `NumberText::mantissa` and `NumberText::exponent`
- `Kind::Array` with `Array::items`, `Array::len`, `Array::is_empty` and `Array::node`
- `Kind::Object` with `Object::members`, `Object::get` and `Object::node`

`Node` also has `Node::span`, `Node::text`, `Node::traverse`, `Node::path_at`, `Node::member_span`, `Node::as_str`, `Node::as_number`, `Node::as_bool`, `Node::as_array` and `Node::as_object`.
