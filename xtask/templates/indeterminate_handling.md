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
- extensions are not allowed unless enabled with `SyntaxOptions`: trailing commas are not valid in json but are valid in jsonc, and comments are not supported
- surrogates are not validated, eg a lone continuation byte is valid
