# JSONC Specification

Requirements for JSONC (JSON with Comments), quoted verbatim from the
[JSONC specification](https://jsonc.org/), repository
[JSONC-org/JSONC](https://github.com/JSONC-org/JSONC), file `index.md` at
commit `84b09994253d4da0fa8e619cfbe9aa6f6229790d`. The specification is a
draft and subject to change. The Media Type section and the appendices'
explanatory prose are intentionally omitted because they do not apply to a
parser or command line tool.

The JSONC specification is licensed under the MIT License.
Copyright (c) 2025 The JSONC Organisation and Contributors.

> jsonc[json-syntax-with-comments]
> JSONC follows the same syntax rules as JSON with the addition of JavaScript
> style comments. Comments can be either single-line or multi-line.
> See https://jsonc.org/#syntax

> jsonc[comments-where-whitespace-allowed]
> They can be placed anywhere insignificant whitespace is allowed in regular JSON.
> See https://jsonc.org/#syntax

> jsonc[comments-do-not-affect-data]
> Comments provide context alongside the data, but they MUST NOT affect
> consumption. Removing all comments MUST yield the same data representation of
> the JSON values.
> See https://jsonc.org/#syntax

> jsonc[single-line-comment]
> Single-line comments start with `//` and continue until a line ending is
> encountered.
> See https://jsonc.org/#single-line-comments

> jsonc[line-ending]
> Refers to the sequence of (often invisible) control characters that indicate the
> end of a line. In JSONC, this sequence can be represented by a line feed (LF), a
> carriage return followed by a line feed (CRLF) or a lone carriage return (CR).
> In terms of Unicode, LF corresponds to U+000A and CR corresponds to U+000D.
> See https://jsonc.org/#conventions-and-terminology

> jsonc[multi-line-comment]
> Multi-line comments (sometimes called block comments) start with `/*` and end
> with `*/`. They can span multiple lines, but can also be used on a single line.
> See https://jsonc.org/#multi-line-comments

> jsonc[no-nested-comments]
> Multi-line comments cannot be nested. The closing of the nested comment will be
> interpreted as the end of the outer comment.
> See https://jsonc.org/#multi-line-comments

> jsonc[trailing-commas]
> JSONC parsers MAY support trailing commas.
> See https://jsonc.org/#trailing-commas

> jsonc[comments-ignored]
> Comments in JSONC are ignored during parsing, allowing developers to annotate
> their JSON data without affecting its structure or content.
> See https://jsonc.org/#semantics

> jsonc[filename-extension]
> The recommended file extension for JSONC documents is `.jsonc`.
> See https://jsonc.org/#filename-extension

> jsonc[mode-line]
> The extension `.json` SHOULD be avoided, but if it's used, there SHOULD be a
> mode line present at the start of the file to indicate that it's actually a
> JSONC file:
> See https://jsonc.org/#filename-extension

> jsonc[no-hash-comments]
> Single-line comments that start with `#` are not supported.
> See https://jsonc.org/#comments-starting-with-
