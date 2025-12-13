<!-- GENERATED FILE - update the templates in the xtask -->

# JJPWRGEM

JJPWRGEM JSON Parser With Really Good Error Messages

An RFC 8259 compliant JSON Parser and formatter!

![A logo of an axolotl riding a skateboard](./logo.webp)

```
$ echo -en "{\"coolKey\"}" | jjp check
error: expected colon after key, found `}`
 --> stdin:1:11
  |
1 | {"coolKey"}
  |  ---------^
  |  |
  |  expected due to `"coolKey"`
  |
help: insert colon and placeholder value
  |
1 | {"coolKey": "🐟🛹"}
  |           ++++++++

```

## Table of contents

- [Table of contents](#table-of-contents)
- [Installation](#installation)
- [Stability](#stability)
- [FAQ](#faq)
- [Motivations](#motivations)

## Installation

### Precompiled

```bash
mise use -g github:20jasper/jjpwrgem
```

See [releases](https://github.com/20jasper/JJPWRGEM/releases) for shell and powershell installation instructions and raw binaries

Note: node adds ~60ms of overhead

```bash
npm install -g jjpwrgem
```

### From source

```bash
cargo install --path .
```

## Stability

JJPWRGEM is in its infancy and extremely likely to have breaking changes (properly marked with semver of course!)

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
- extensions such as trailing commas or comments are not allowed
- surrogates are not validated, eg a lone continuation byte is valid

### Is it blazingly fast™?

Axolotls can't walk so fast, so skateboards are pretty fast 🛹🐟

jjpwrgem can parse and pretty print a 1.7MB JSON file in around ~11ms and the average package.json in ~500 microseconds

See the [benchmarks](/benchmarks.md) for more info!

## FAQ

### What does JJPWRGEM stand for?

JJPWRGEM JSON Parser With Really Good Error Messages. I was inspired by GNU to make a recursive acronym

### How do you pronounce JJPWRGEM?

/ˈdʒeɪ dʒeɪ ˈpaʊər dʒɛm/ JAY-jay-POW-er-jem

### But why is it called that?

It sounds cool and the name isn't taken on any package managers

### Why is the logo an axolotl riding a skateboard?

It's cool

### How long is an axolotl?

According to the San Diego zoo, "[a]n axolotl can reach 12 inches in length, but on average grows to about 9 inches[^axolotlFact]"

[^axolotlFact]: https://animals.sandiegozoo.org/animals/axolotl

## Motivations

I originally started this project to practice finite state machines, but got back into it when hearing about the internals of some formatters and compilers!

I am heavily inspired by the Rust compiler's error messages. I love that unhelpful errors are considered bugs

I checked out several JSON parsers and formatters, and none provided much context on _why_ a key was missing. Errors ranged from "expected closing on byte 10" to a snapshot of source code for that character, but none were up to my standards
