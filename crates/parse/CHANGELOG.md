# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
- join_into utility to declaritively avoid allocating delimiter strings
- write to single buffer instead of allocating buffer per JSON value
