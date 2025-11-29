# JJPWRGEM

JJPWRGEM JSON Parser With Really Good Error Messages

An (eventually) RFC 8259 compliant JSON Parser

Currently passes 261/319 tests from @nst's amazing [JSONTestSuite](https://github.com/nst/JSONTestSuite)!

Failures are due to high levels of recursion and escaping

```
echo -en '{"coolKey"}' | jjp
error: expected colon after key, found `}`
  ╭▸ stdin:1:11
  │
1 │ {"coolKey"}
  │  ┬────────━
  │  │
  │  expected due to `"coolKey"`
  ╰╴
help: insert colon and placeholder value
  ╭╴
1 │ {"coolKey": "🐟🛹"}
  ╰╴          ++++++++
```

## Motivations

I originally started this project to practice finite state machines, but got back into it when hearing about the internals of some formatters and compilers!

I am heavily inspired by the Rust compiler's error messages. I love the idea that unhelpful errors are considered bugs

I checked out several JSON parsers and formatters, and none provided much context on _why_ a key was missing. Errors ranged from "expected closing on byte 10" to a snapshot of source code for that character, but none were up to my standards

## indeterminate handling

How cases undefined by the spec are handled

- numbers of any size or length are allowed
  - the original precision will be maintained
  - -0 is not equal to 0 and will persist
- the last duplicate key is stored
  - escaped and unescaped characters are considered not equal
- parsing will fail if BOM is included
- only utf8 encoding is supported
- no limitations on nesting or length
- extensions such as trailing commas or comments are not allowed

## Notes

I went with annotate snippets over other libraries like codespan reporting since it better supports patches and workflows that don't require files, like reading from stdin

## references

https://rustc-dev-guide.rust-lang.org/diagnostics/error-codes.html
https://github.com/rust-lang/rust/pull/27475
OXC
