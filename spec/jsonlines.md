# JSON Lines Specification

Requirements for the JSON Lines format, derived from
[jsonlines.org](https://jsonlines.org/).

> jsonlines[utf8]
> Files MUST be encoded in UTF-8
> See https://jsonlines.org/#utf-8-encoding

> jsonlines[byte-order-marker]
> a byte order mark (U+FEFF) must NOT be included
> See https://jsonlines.org/#utf-8-encoding

> jsonlines[each-line-is-a-valid-json-value]
> The most common values will be objects or arrays, but any JSON value is
> permitted. e.g. null is a valid value but a blank line is not
> See https://jsonlines.org/#each-line-is-a-valid-json-value

> jsonlines[newline-delimiter]
> Line Terminator is '\n'
> See https://jsonlines.org/#line-terminator-is-n

> jsonlines[end-of-file]
> If a line terminator follows the last JSON value in a file, it must be the last byte in the file
> See https://jsonlines.org/#line-terminator-is-n
