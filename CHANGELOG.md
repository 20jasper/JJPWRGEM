# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.4](https://github.com/20jasper/JJPWRGEM/releases/tag/jjpwrgem-v0.1.4) - 2025-12-05

### Fixed

- fix placeholder error message ([#39](https://github.com/20jasper/JJPWRGEM/pull/39))
- fixture constants
- fix lint error

### Other

- proper exit codes and new snapshot format
- release binary ([#67](https://github.com/20jasper/JJPWRGEM/pull/67))
- add cargo diet ([#66](https://github.com/20jasper/JJPWRGEM/pull/66))
- Integrate cargo-shear for unused dependency detection ([#65](https://github.com/20jasper/JJPWRGEM/pull/65))
- workspace, ui, cli, and parse crates ([#63](https://github.com/20jasper/JJPWRGEM/pull/63))
- Update GitHub Actions workflows to use actions/checkout@v6 ([#64](https://github.com/20jasper/JJPWRGEM/pull/64))
- avoid string allocations ([#62](https://github.com/20jasper/JJPWRGEM/pull/62))
- set opt level to 3
- Change error to box inner error to reduce Result size
- Misc, update resources, add script for installing packages, set coverage location, make ast public, split up match branches for diagnostics for clearer coverage
- Update readme with fun axolotl fact
- reformat tests and run prettify and uglify on passing ([#60](https://github.com/20jasper/JJPWRGEM/pull/60))
- range naming consistency ([#59](https://github.com/20jasper/JJPWRGEM/pull/59))
- handle uescapes ([#58](https://github.com/20jasper/JJPWRGEM/pull/58))
- lefthook fix ([#57](https://github.com/20jasper/JJPWRGEM/pull/57))
- reorder error kinds ([#56](https://github.com/20jasper/JJPWRGEM/pull/56))
- update pronunciation
- branding ([#55](https://github.com/20jasper/JJPWRGEM/pull/55))
- lefthook format on commit precommit and lint/test/format on push ([#54](https://github.com/20jasper/JJPWRGEM/pull/54))
- expected escape diagnostics ([#53](https://github.com/20jasper/JJPWRGEM/pull/53))
- Diagnostics for expected quote ([#52](https://github.com/20jasper/JJPWRGEM/pull/52))
- string escapes ([#51](https://github.com/20jasper/JJPWRGEM/pull/51))
- leading 0s following digits 1-9 now fail in integer ([#50](https://github.com/20jasper/JJPWRGEM/pull/50))
- Update to gracefully handle invalid utf encoding errors
- refactor to use From impls
- lots of conformance tests
- arrays happy path and expected closing brace/value ([#47](https://github.com/20jasper/JJPWRGEM/pull/47))
- exponents ([#46](https://github.com/20jasper/JJPWRGEM/pull/46))
- Handle fractions ([#45](https://github.com/20jasper/JJPWRGEM/pull/45))
- allow values after number ([#43](https://github.com/20jasper/JJPWRGEM/pull/43))
- don't escape forward slash in json char display ([#42](https://github.com/20jasper/JJPWRGEM/pull/42))
- Replace accidentally removed doc comments ([#41](https://github.com/20jasper/JJPWRGEM/pull/41))
- Positive and negative integers ([#40](https://github.com/20jasper/JJPWRGEM/pull/40))
- Add Dependabot configuration for weekly Cargo and GitHub Actions updates ([#37](https://github.com/20jasper/JJPWRGEM/pull/37))
- conformance testing ([#38](https://github.com/20jasper/JJPWRGEM/pull/38))
- Update thiserror to 2.0.17 ([#36](https://github.com/20jasper/JJPWRGEM/pull/36))
- add snapshot tests ([#35](https://github.com/20jasper/JJPWRGEM/pull/35))
- Escape characters in error message
- update expected key patch to add closing curly when no chars follow
- Update expected colon tests to add colon and curly if relevant
- Don't suggest closing curly brace if expected key and other character is there
- Expected value diagnostics
- unescaped control patch ([#32](https://github.com/20jasper/JJPWRGEM/pull/32))
- Token after end patches ([#31](https://github.com/20jasper/JJPWRGEM/pull/31))
- decouple diagnostics from display ([#29](https://github.com/20jasper/JJPWRGEM/pull/29))
- Context and Patches, ExpectedCommaOrClosedCurlyBrace, ExpectedOpenCurlyBrace  ([#28](https://github.com/20jasper/JJPWRGEM/pull/28))
- token context and patches to expected colon and value ([#25](https://github.com/20jasper/JJPWRGEM/pull/25))
- format-tokens
- add watch mode for installing crate locally
- rename crate and binary
- Use annotate-snippets to display errors. Builds errors, context, and patches from errors ([#21](https://github.com/20jasper/JJPWRGEM/pull/21))
- provide context to expected key errors ([#20](https://github.com/20jasper/JJPWRGEM/pull/20))
- Handle control characters in strings ([#19](https://github.com/20jasper/JJPWRGEM/pull/19))
- whitespace ([#17](https://github.com/20jasper/JJPWRGEM/pull/17))
- escape quotes ([#16](https://github.com/20jasper/JJPWRGEM/pull/16))
- Include line and column numbers in errors ([#15](https://github.com/20jasper/JJPWRGEM/pull/15))
- prettify ([#14](https://github.com/20jasper/JJPWRGEM/pull/14))
- handle unterminated quotes ([#13](https://github.com/20jasper/JJPWRGEM/pull/13))
- naming conventions ([#12](https://github.com/20jasper/JJPWRGEM/pull/12))
- error messaging for unterminated values ([#11](https://github.com/20jasper/JJPWRGEM/pull/11))
- Split out Object parsing and rename state variants ([#10](https://github.com/20jasper/JJPWRGEM/pull/10))
- error handling ([#9](https://github.com/20jasper/JJPWRGEM/pull/9))
- simplify value parsing
- uglify arbitrarily nested json
- Handle single arbitrarily nested object
- Make State track some context instead of passing around an optional value
- simplify kv to map test helper
- Use rstest templates
- format
- make uglify public
- Uglify objects and primitives
- boolean tokens and add table tests for primitive values
- Handle various values as top level values
- Return Value from parser
- Move tokens to own module
- Make value enum to handle AST better
- Allow null as an object value
- Handles multiple keys
- update todos
- format
- add justfile
- add ci
- Remove string parsing logic from parser
- Initial tokenizer
- WIP
- use start and end states instead of words
- update colon to colon character
- add mermaid chart
- impl From Error for all Into<String>
- update states file to show current functionality
- add todos
- refactor string logic
- handle one json key
- Character after end
- unrecognized character
- unmatched char error
- tighten error handling. Use enum specific variant for empty string
- handle all states for json objects with no keys
- Add object states
- init

### Removed

- removed From<Into<String>> impl for token and added for bool ([#18](https://github.com/20jasper/JJPWRGEM/pull/18))
