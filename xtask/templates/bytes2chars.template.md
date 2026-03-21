# bytes2chars

lazily decodes utf-8 [`char`]s from bytes

provides lazy, fallible analogs to [`str::Chars`] ([`Utf8Chars`]) and [`str::CharIndices`] ([`Utf8CharIndices`]), as well as a lower-level push-based [`Utf8Decoder`]

[`str::Chars`]: core::str::Chars
[`str::CharIndices`]: core::str::CharIndices
[`Utf8Chars`]: crate::Utf8Chars
[`Utf8CharIndices`]: crate::Utf8CharIndices

## design goals

- rich errors—what went wrong and where
- lazy
- `no-std`
- performance

## quick start

prefer iterators like [`Utf8CharIndices`] or [`Utf8Chars`] if you have access to a byte iterator. [`Utf8Chars`] still tracks bytes for error context, so it's purely a convenience wrapper

if you receive bytes in chunks, use the push-based [`Utf8Decoder`]

## examples

### iterator api

```rust
# use bytes2chars::{Result, Utf8CharIndices, Utf8Chars};
# fn main() -> Result<()> {
let input = b"\xF0\x9F\xA6\x80 rust".iter().copied();

// decode into an iterator of chars and their positions
let indexed = Utf8CharIndices::from(input.clone()).collect::<Result<Vec<_>>>()?;
let expected = vec![(0, '🦀'), (4, ' '), (5, 'r'), (6, 'u'), (7, 's'), (8, 't')];
assert_eq!(indexed, expected);

// convenience wrapper to decode into an iterator of chars
let chars = Utf8Chars::from(input).collect::<Result<String>>()?;
assert_eq!(chars, "🦀 rust");
# Ok(())
# }
```

### error handling

```rust
# use bytes2chars::{Error, ErrorKind, Result, Utf8Chars};
# fn main() -> Result<()> {
let err = Utf8Chars::from(b"hello \x80 world".iter().copied())
    .collect::<Result<String>>()
    .unwrap_err();

assert_eq!(err, Error { range: 6..7, kind: ErrorKind::InvalidLead(0x80) });
assert_eq!(
  err.to_string(),
  "invalid utf-8 at bytes 6..7: byte 0x80 cannot start a UTF-8 sequence"
);
# Ok(())
# }
```

### push based decoder

```rust
# use bytes2chars::Utf8Decoder;
# fn main() -> bytes2chars::Result<()> {
let mut decoder = Utf8Decoder::new(0);
assert_eq!(decoder.push(0xF0), None); // accumulating
assert_eq!(decoder.push(0x9F), None);
assert_eq!(decoder.push(0xA6), None);
assert_eq!(decoder.push(0x80), Some(Ok((0, '🦀')))); // complete
decoder.finish()?; // check for truncated sequence
# Ok(())
# }
```

## alternatives

### [`std::str::from_utf8`](https://doc.rust-lang.org/std/str/fn.from_utf8.html)

eager and error context provides a range but not a particular cause

### [`utf8-decode`](https://docs.rs/utf8-decode/latest/utf8_decode/index.html)

also lazy. error provides a range but not a particular cause. does not provide a push based decoder
