# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## 0.16.0 — 2026-11-28

### Changed

- [e063070](https://github.com/20jasper/JJPWRGEM/commit/e0630703268c9fce34e02d4bbfbf583970b51b19) move `CheckParser` and `CheckInputSettings` to the cli, take an optional short flag in `Spec::shorthand`, and add a string setting kind.
- [0893627](https://github.com/20jasper/JJPWRGEM/commit/08936274d0a683059f99a5877776d4f127083451) `Document::root` returns a `Node` instead of a `Value`.

### Added

- [e063070](https://github.com/20jasper/JJPWRGEM/commit/e0630703268c9fce34e02d4bbfbf583970b51b19) add settings and choice enum macros to define agnostic settings structs with rust like syntax.
- [0893627](https://github.com/20jasper/JJPWRGEM/commit/08936274d0a683059f99a5877776d4f127083451) add a `Node` view that carries its document, so invalid indexing is impossible. provides accessors like `as_str` and `kind` for exhaustive matching

### Removed

- [0893627](https://github.com/20jasper/JJPWRGEM/commit/08936274d0a683059f99a5877776d4f127083451) remove `Value`, `Key`, `ObjectEntries` and `quoted` from the public API, use `Node` instead.

## 0.15.0 — 2026-11-25

### Changed

- [a103b5c](https://github.com/20jasper/JJPWRGEM/commit/a103b5c649d66af6250539d010f204b3004a6c0f) `Document::parse` and `validate_str` now take `SyntaxOptions`, which can allow trailing commas. `SyntaxOptions::JSON` keeps strict json.
- [9385563](https://github.com/20jasper/JJPWRGEM/commit/93855636e62d98e62904a4ecaec8466ccae86cdb) explain that comments are not allowed in json and suggest removing them.

### Added

- [e42c731](https://github.com/20jasper/JJPWRGEM/commit/e42c731a9029f960e542fd08818364d974768e8c) add `Value::is_container` and `Value::is_primitive`
- [d2939b8](https://github.com/20jasper/JJPWRGEM/commit/d2939b870a7a87b2ffda342c7cc363f797736493) accept `/* */` block comments with `SyntaxOptions::JSONC`.
- [1c5ede9](https://github.com/20jasper/JJPWRGEM/commit/1c5ede9e616fb6ce8171dcd03d5028c3a6ae3f6c) check jsonc with `--parser jsonc`.
- [e42c731](https://github.com/20jasper/JJPWRGEM/commit/e42c731a9029f960e542fd08818364d974768e8c) add `Value::child_at` and `Value::path_at`
- [9385563](https://github.com/20jasper/JJPWRGEM/commit/93855636e62d98e62904a4ecaec8466ccae86cdb) add `SyntaxOptions::comments` and `SyntaxOptions::JSONC` to accept `//` line comments.

## 0.14.4 — 2026-11-17

### Added

- [337a419](https://github.com/20jasper/JJPWRGEM/commit/337a419e0575590d3b57ae919ebfa374b8a9f997) add `settings` metadata describing every format option, for generating or describing settings in integrations like editor extensions
- [eff36b5](https://github.com/20jasper/JJPWRGEM/commit/eff36b510b5c7ec962cf875461f7228d704ffe06) add `--mode <pretty|uglify>` with `-u` as a shorthand for `--mode uglify`, and `format::FormatMode`
- [eff36b5](https://github.com/20jasper/JJPWRGEM/commit/eff36b510b5c7ec962cf875461f7228d704ffe06) add `format::FormatConfig` and its conversion into a `FormatRequest`, shared by every front end. Format option conflicts exit with code 2 and a usage line. An explicit `--preferred-width` conflicts even at its default

### Fixed

- [d209a12](https://github.com/20jasper/JJPWRGEM/commit/d209a12ee24961da18226ec88b7228b10b4bd9fd) fix broken doc links

## 0.14.3 — 2026-11-13

### Added

- [515dd1f](https://github.com/20jasper/JJPWRGEM/commit/515dd1f455756ba196192aadf51a484f9581ef3b) add `Value::span`, `Value::children` and `Value::traverse`
- [515dd1f](https://github.com/20jasper/JJPWRGEM/commit/515dd1f455756ba196192aadf51a484f9581ef3b) add `Document::source`
- [a6ca17f](https://github.com/20jasper/JJPWRGEM/commit/a6ca17f7e9018291f5336e22e92ce128d323052c) add `ast::quoted`

## 0.14.2 — 2026-11-12

### Added

- [9f2a798](https://github.com/20jasper/JJPWRGEM/commit/9f2a798305ece123ccad6e98a04ca7ed82fe8a98) add methods to access source text

## [0.14.1](https://github.com/20jasper/JJPWRGEM/compare/jjpwrgem-parse-v0.14.0...jjpwrgem-parse-v0.14.1) - 2026-10-31

### Performance

- *(serde)* find escapes with the lexer's simd scan ([#514](https://github.com/20jasper/JJPWRGEM/pull/514))
- *(serde)* write numbers with itoa and zmij ([#513](https://github.com/20jasper/JJPWRGEM/pull/513))
- *(serde)* write map keys straight into the output ([#511](https://github.com/20jasper/JJPWRGEM/pull/511))
- *(parse)* return the error instead of unreachable! for the array open bracket ([#508](https://github.com/20jasper/JJPWRGEM/pull/508))
- *(format)* indent with compile time slices ([#504](https://github.com/20jasper/JJPWRGEM/pull/504))
- *(parse)* collect container children on a shared scratch stack ([#502](https://github.com/20jasper/JJPWRGEM/pull/502))
- *(parse)* inline the small helpers on the per token and per value paths ([#496](https://github.com/20jasper/JJPWRGEM/pull/496))
- *(parse)* split object parsing happy and sad path ([#495](https://github.com/20jasper/JJPWRGEM/pull/495))
- *(parse)* split sad and happy array paths ([#494](https://github.com/20jasper/JJPWRGEM/pull/494))
- *(parse)* split mantissa success and error path ([#493](https://github.com/20jasper/JJPWRGEM/pull/493))
- *(parse)* scan strings with simd ([#492](https://github.com/20jasper/JJPWRGEM/pull/492))

## [0.14.0](https://github.com/20jasper/JJPWRGEM/compare/jjpwrgem-parse-v0.13.2...jjpwrgem-parse-v0.14.0) - 2026-06-01

### Added

- *(cli)* [**breaking**] replace --json-lines flag with --parser <type>
- *(parse)* [**breaking**] builders and structs for configurations
- [**breaking**] add range to value struct and visitor methods

### Deprecated

- *(cli)* [**breaking**] remove cr line ending option

## [0.13.2](https://github.com/20jasper/JJPWRGEM/compare/jjpwrgem-parse-v0.13.1...jjpwrgem-parse-v0.13.2) - 2026-05-26

### Added

- *(parse)* JSON lines formatting and checking

## [0.13.1](https://github.com/20jasper/JJPWRGEM/compare/jjpwrgem-parse-v0.13.0...jjpwrgem-parse-v0.13.1) - 2026-05-25

### Fixed

- out of bounds index on multibyte error source

## [0.13.0](https://github.com/20jasper/JJPWRGEM/compare/jjpwrgem-parse-v0.12.1...jjpwrgem-parse-v0.13.0) - 2026-05-24

### Fixed

- set limit of 128 nesting depth

## [0.12.1](https://github.com/20jasper/JJPWRGEM/compare/jjpwrgem-parse-v0.12.0...jjpwrgem-parse-v0.12.1) - 2026-05-23

### Fixed

- *(deps)* loosen dependency versions

## [0.12.0](https://github.com/20jasper/JJPWRGEM/compare/jjpwrgem-parse-v0.11.1...jjpwrgem-parse-v0.12.0) - 2026-05-23

### Deprecated

- [**breaking**] flatten parse module hierarchy and tighten internal visibility

## [0.11.0](https://github.com/20jasper/JJPWRGEM/releases/tag/jjpwrgem-parse-v0.11.0) - 2026-05-22

### Added

- [**breaking**] inline objects based on width
- [**breaking**] number only array will fill format instead of expanding
- [**breaking**] prettier number normalization
- improve exponent diagnostics
- provide rich utf8 error context to the cli
- *(parse)* add prettify_value_into and uglify_value_into
- serde feature for serializing serde::Serializable and serde_json::Value ([#91](https://github.com/20jasper/JJPWRGEM/pull/91))
- end of line option
- [**breaking**] preferred width
- [**breaking**] change pretty default indent to 2 spaces
- [**breaking**] non empty arrays put items on newline
- consistent key ordering

### Deprecated

- [**breaking**] error helpers, diagnostic constants, Emitter, join_into, UglifyEmitVisitor
- deprecated!(parse): removed line and column numbers from error and display impl

### Documentation

- add xtask to generate readmes

### Fixed

- account for delimiters in array width measurement
- parse value visitor now handles object key events properly
- spacial case empty object to be on one line

### Performance

- remove iterator abstraction layers
- bytewise parsing for exponents
- bytewise parsing for mantissa
- *(parse)* skip whitespace with portable SIMD (u8x32)
- *(parse)* replace next_if whitespace loop with peek fast path + byte scan
- uglify_serializable now uses faster serialization from the crate instead of deferring to serde_json
- don't build ast when uglifying from string slice
- don't build ast when validating syntax
- track count of digits instead of vec of chars
- only cache successful tokens in TokenStream
- write delimiters and indentation directly to buffer, avoiding intermediate allocations
- avoid using fmt machinery in hot paths, instead pushing directly
- TokenStream iterator instead of collecting into intermediary Vec
- join_into utility to declaratively avoid allocating delimiter strings
- write to single buffer instead of allocating buffer per JSON value
