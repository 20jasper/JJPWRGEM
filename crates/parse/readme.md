<!-- GENERATED FILE - update the templates in the xtask -->

# jjpwrgem-parse

JSON parser and formatter with rich errors. Currently unstable—expect breaking changes.

Tracey is used to track spec compliance.

[![jsonlines: 60%](https://img.shields.io/badge/spec:jsonlines-60%25-yellow)](https://github.com/20jasper/JJPWRGEM/blob/main/spec/jsonlines.md)

## Indeterminate Handling

How cases undefined by the spec are handled

- numbers of any size or length are allowed
  - the original precision will be maintained
  - -0 is not equal to 0 and will persist
- the last duplicate key is stored
  - escaped and unescaped characters are considered not equal
- parsing will fail if BOM is included
- only utf8 encoding is supported
- no limitations on nesting or length
- extensions are not allowed unless enabled with `SyntaxOptions`: trailing commas and line comments are not valid in json but are valid in jsonc
- surrogates are not validated, eg a lone continuation byte is valid
