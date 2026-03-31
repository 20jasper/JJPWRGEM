# UTF-8 Specification

Requirements for valid UTF-8 encoding, derived from
[RFC 3629](https://datatracker.ietf.org/doc/html/rfc3629).

## encoding shapes

Multi-byte sequences use **continuation bytes** for all bytes after the lead. A continuation
byte matches the pattern `10xxxxxx` (byte range 0x80–0xBF, `UTF8-tail = %x80-BF` per RFC 3629 §4)
and carries 6 bits of payload. See [RFC 3629 §4](https://datatracker.ietf.org/doc/html/rfc3629#section-4).

> utf8[encoding.ascii]
> A single byte in 0x00–0x7F represents a code point in U+0000–U+007F, with the byte
> value equal to the code point value (`UTF8-1 = %x00-7F` per RFC 3629 §4).
> See [RFC 3629 §3](https://datatracker.ietf.org/doc/html/rfc3629#section-3) and
> [RFC 3629 §4](https://datatracker.ietf.org/doc/html/rfc3629#section-4).

> utf8[encoding.two-byte]
> A lead byte in 0xC2–0xDF followed by one continuation byte represents a code point in
> U+0080–U+07FF (`UTF8-2 = %xC2-DF UTF8-tail` per RFC 3629 §4).
> See [RFC 3629 §3](https://datatracker.ietf.org/doc/html/rfc3629#section-3) and
> [RFC 3629 §4](https://datatracker.ietf.org/doc/html/rfc3629#section-4).

> utf8[encoding.three-byte]
> A three-byte sequence represents a code point in U+0800–U+FFFF per RFC 3629 §4:
> `UTF8-3 = %xE0 %xA0-BF UTF8-tail / %xE1-EC 2( UTF8-tail ) / %xED %x80-9F UTF8-tail / %xEE-EF 2( UTF8-tail )`
> See [RFC 3629 §3](https://datatracker.ietf.org/doc/html/rfc3629#section-3) and
> [RFC 3629 §4](https://datatracker.ietf.org/doc/html/rfc3629#section-4).

> utf8[encoding.four-byte]
> A four-byte sequence represents a code point in U+10000–U+10FFFF per RFC 3629 §4:
> `UTF8-4 = %xF0 %x90-BF 2( UTF8-tail ) / %xF1-F3 3( UTF8-tail ) / %xF4 %x80-8F 2( UTF8-tail )`
> See [RFC 3629 §3](https://datatracker.ietf.org/doc/html/rfc3629#section-3) and
> [RFC 3629 §4](https://datatracker.ietf.org/doc/html/rfc3629#section-4).

## validate — validity constraints

> utf8[validate.invalid-lead]
> A continuation byte (0x80–0xBF) in lead position MUST be rejected. These bytes match the
> `10xxxxxx` pattern (`UTF8-tail = %x80-BF` per RFC 3629 §4) and can never start a valid
> UTF-8 sequence. RFC 3629 §3: "Implementations of the decoding algorithm above MUST
> protect against decoding invalid sequences."
> See [RFC 3629 §3](https://datatracker.ietf.org/doc/html/rfc3629#section-3).

> utf8[validate.max-sequence-length]
> UTF-8 sequences MUST NOT exceed 4 bytes. RFC 3629 §3: "In UTF-8, characters from the
> U+0000..U+10FFFF range (the UTF-16 accessible range) are encoded using sequences of 1 to
> 4 octets." Bytes 0xF8–0xFF have 5 or more leading `1`-bits and imply sequences of 5 or
> more bytes, which MUST be rejected.
> See [RFC 3629 §3](https://datatracker.ietf.org/doc/html/rfc3629#section-3).

> utf8[validate.expected-continuation]
> In a multi-byte sequence, all bytes after the lead byte MUST be valid continuation bytes
> (`UTF8-tail`). RFC 3629 §4: `UTF8-tail = %x80-BF`. A non-continuation byte where one is
> required is invalid. RFC 3629 §3: "Implementations of the decoding algorithm above MUST
> protect against decoding invalid sequences."
> See [RFC 3629 §3](https://datatracker.ietf.org/doc/html/rfc3629#section-3) and
> [RFC 3629 §4](https://datatracker.ietf.org/doc/html/rfc3629#section-4).

> utf8[validate.no-surrogates]
> UTF-16 surrogate code points (U+D800–U+DFFF) MUST NOT appear in UTF-8 encoded text.
> RFC 3629 §3: "The definition of UTF-8 prohibits encoding character numbers between
> U+D800 and U+DFFF, which are reserved for use with the UTF-16 encoding form (as surrogate
> pairs) and do not directly represent characters."
> See [RFC 3629 §3](https://datatracker.ietf.org/doc/html/rfc3629#section-3).

> utf8[validate.max-codepoint]
> Code points above U+10FFFF MUST NOT be encoded. RFC 3629 §3: "In UTF-8, characters from
> the U+0000..U+10FFFF range (the UTF-16 accessible range) are encoded using sequences of 1
> to 4 octets." U+10FFFF is the maximum Unicode scalar value, corresponding to the highest
> row in the RFC 3629 §3 encoding table (`0001 0000-0010 FFFF`). Lead bytes 0xF5–0xF7 are
> 4-byte leads whose minimum encoded code point (U+140000, U+180000, U+1C0000 respectively)
> already exceeds U+10FFFF and MUST be rejected without reading continuation bytes.
> See [RFC 3629 §3](https://datatracker.ietf.org/doc/html/rfc3629#section-3).

> utf8[validate.no-overlong]
> Each code point MUST use the shortest possible encoding. RFC 3629 §3: "It is important
> to note that the rows of the table are mutually exclusive, i.e., there is only one valid
> way to encode a given character." A sequence using more bytes than the minimum is an
> overlong encoding and MUST be rejected. Lead bytes 0xC0–0xC1 are 2-byte leads that would
> encode U+0000–U+007F using 2 bytes instead of 1 and MUST be rejected without reading
> continuation bytes.
> See [RFC 3629 §3](https://datatracker.ietf.org/doc/html/rfc3629#section-3).

> utf8[validate.unfinished]
> A multi-byte sequence MUST be complete before the byte stream ends. RFC 3629 §3:
> "Implementations of the decoding algorithm above MUST protect against decoding invalid
> sequences."
> See [RFC 3629 §3](https://datatracker.ietf.org/doc/html/rfc3629#section-3).
