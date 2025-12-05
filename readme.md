# JJPWRGEM

JJPWRGEM JSON Parser With Really Good Error Messages

An RFC 8259 compliant JSON Parser!

![A logo of an axolotl riding a skateboard](./logo.webp)

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

## Stability

JJPWRGEM is in its infancy and extremely likely to have breaking changes (properly marked with semver of course!)

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
- surrogates are not validated, eg a lone continuation byte is valid

## FAQ

### What does JJPWRGEM stand for?

JJPWRGEM JSON Parser With Really Good Error Messages. I was inspired by GNU to make a recursive acronym

### How do you pronounce JJPWRGEM?

/ˈdʒeɪ dʒeɪ ˈpaʊər dʒɛm/ JAY-jay-POW-er-jem

### But why is it called that?

It sounds cool and the name isn't taken on any package managers

### Why is the logo an axolotl riding a skateboard?

It's cool

### Is it blazingly fast™?

Axolotls can't walk so fast, so relatively, yes

### How long is an axolotl?

According to the San Diego zoo, "[a]n axolotl can reach 12 inches in length, but on average grows to about 9 inches[^axolotlFact]"

[^axolotlFact]: https://animals.sandiegozoo.org/animals/axolotl

## Motivations

I originally started this project to practice finite state machines, but got back into it when hearing about the internals of some formatters and compilers!

I am heavily inspired by the Rust compiler's error messages. I love that unhelpful errors are considered bugs

I checked out several JSON parsers and formatters, and none provided much context on _why_ a key was missing. Errors ranged from "expected closing on byte 10" to a snapshot of source code for that character, but none were up to my standards

## Notes

I went with annotate snippets over other libraries like codespan reporting since it better supports patches and workflows that don't require files, like reading from stdin

## references

### parsers/diagnostics

https://rustc-dev-guide.rust-lang.org/diagnostics/error-codes.html

https://github.com/rust-lang/rust/pull/27475

OXC

### Accessibility

https://afixt.com/accessible-by-design-improving-command-line-interfaces-for-all-users/

miette
