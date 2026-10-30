# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.3](https://github.com/20jasper/JJPWRGEM/compare/jjpwrgem-v0.7.2...jjpwrgem-v0.7.3) - 2026-10-30

### Documentation

- refresh benchmarks ([#500](https://github.com/20jasper/JJPWRGEM/pull/500))

### Performance

lowers memory usage and improves speed significantly across the board

#### CLI time (median of 100 runs)

| file         | command     |   v0.7.2 |     main |  change |
| ------------ | ----------- | -------: | -------: | ------: |
| canada       | `format -u` | 14.32 ms |  9.27 ms | −35.3% |
| canada       | `format`    | 27.82 ms | 19.07 ms | −31.5% |
| canada       | `check`     | 12.32 ms |  7.82 ms | −36.6% |
| citm_catalog | `format -u` |  5.27 ms |  3.80 ms | −28.0% |
| citm_catalog | `format`    |  9.15 ms |  6.54 ms | −28.5% |
| citm_catalog | `check`     |  4.49 ms |  3.62 ms | −19.5% |
| twitter      | `format -u` |  3.10 ms |  2.38 ms | −23.2% |
| twitter      | `format`    |  4.32 ms |  3.33 ms | −22.9% |
| twitter      | `check`     |  2.64 ms |  2.13 ms | −19.3% |

#### CLI peak memory

| file         | command     | v0.7.2 |   main |  change |
| ------------ | ----------- | -----: | -----: | ------: |
| canada       | `format -u` | 7.9 MB | 7.8 MB |  −1.2% |
| canada       | `format`    | 22.9 MB | 17.0 MB | −25.7% |
| canada       | `check`     | 5.9 MB | 5.7 MB |  −3.9% |
| citm_catalog | `format -u` | 4.6 MB | 4.2 MB |  −9.0% |
| citm_catalog | `format`    | 8.8 MB | 6.9 MB | −21.8% |
| citm_catalog | `check`     | 4.1 MB | 3.8 MB |  −7.2% |
| twitter      | `format -u` | 4.0 MB | 3.7 MB |  −7.2% |
| twitter      | `format`    | 5.6 MB | 4.7 MB | −16.3% |
| twitter      | `check`     | 3.6 MB | 3.2 MB | −10.0% |

#### LSP server (open, 20 full-document edits, format)

| file         | memory v0.7.2 | memory main |  change | edit → diagnostics v0.7.2 | edit → diagnostics main |  change |
| ------------ | ------------: | ----------: | ------: | ------------------------: | ----------------------: | ------: |
| canada       |       50.4 MB |     37.4 MB | −25.8% |                   28.7 ms |                 23.2 ms | −19.2% |
| citm_catalog |       19.1 MB |     14.8 MB | −22.4% |                   11.2 ms |                  9.4 ms | −16.7% |
| twitter      |       11.5 MB |     10.0 MB | −12.8% |                    6.4 ms |                  5.7 ms | −10.0% |


- *(cli)* validate utf8 with simdutf8 ([#507](https://github.com/20jasper/JJPWRGEM/pull/507))
- *(cli)* print the output in one newline terminated write ([#505](https://github.com/20jasper/JJPWRGEM/pull/505))
- *(format)* indent with compile time slices ([#504](https://github.com/20jasper/JJPWRGEM/pull/504))
- build releases with fat lto, one codegen unit and abort on panic ([#497](https://github.com/20jasper/JJPWRGEM/pull/497))
- *(parse)* return the error instead of unreachable! for the array open bracket ([#508](https://github.com/20jasper/JJPWRGEM/pull/508))
- *(parse)* collect container children on a shared scratch stack ([#502](https://github.com/20jasper/JJPWRGEM/pull/502))
- *(parse)* inline the small helpers on the per token and per value paths ([#496](https://github.com/20jasper/JJPWRGEM/pull/496))
- *(parse)* split object parsing happy and sad path ([#495](https://github.com/20jasper/JJPWRGEM/pull/495))
- *(parse)* split sad and happy array paths ([#494](https://github.com/20jasper/JJPWRGEM/pull/494))
- *(parse)* split mantissa success and error path ([#493](https://github.com/20jasper/JJPWRGEM/pull/493))
- *(parse)* scan strings with simd ([#492](https://github.com/20jasper/JJPWRGEM/pull/492))

## [0.7.2](https://github.com/20jasper/JJPWRGEM/compare/jjpwrgem-v0.7.1...jjpwrgem-v0.7.2) - 2026-10-26

### Documentation

- update vscode and LSP perf claims
- add vscode extension URL
- add metadata and docs for vscode extension

### Fixed

- *(deps)* update tokio to 1.54.1 for RUSTSEC-2026-0386
- *(deps)* update rust deps (minor + patch) ([#393](https://github.com/20jasper/JJPWRGEM/pull/393))

## [0.7.1](https://github.com/20jasper/JJPWRGEM/compare/jjpwrgem-v0.7.0...jjpwrgem-v0.7.1) - 2026-06-06

### Documentation

- add readme to npm package

## [0.7.0](https://github.com/20jasper/JJPWRGEM/compare/jjpwrgem-v0.6.3...jjpwrgem-v0.7.0) - 2026-06-01

### Added

- *(cli)* [**breaking**] replace --json-lines flag with --parser <type>
- *(parse)* [**breaking**] builders and structs for configurations
- [**breaking**] add range to value struct and visitor methods

### Deprecated

- *(cli)* [**breaking**] remove cr line ending option

## [0.6.3](https://github.com/20jasper/JJPWRGEM/compare/jjpwrgem-v0.6.2...jjpwrgem-v0.6.3) - 2026-05-26

### Added

- *(cli)* json lines support
- *(parse)* JSON lines formatting and checking

## [0.6.2](https://github.com/20jasper/JJPWRGEM/compare/jjpwrgem-v0.6.1...jjpwrgem-v0.6.2) - 2026-05-25

### Fixed

- out of bounds index on multibyte error source

## [0.6.1](https://github.com/20jasper/JJPWRGEM/compare/jjpwrgem-v0.6.0...jjpwrgem-v0.6.1) - 2026-05-24

### Added

- add lsp with diagnostics, formatting, and code actions

### Deprecated

- deprecated!(parse): removed line and column numbers from error and display impl

### Fixed

- set limit of 128 nesting depth
- support utf16 and utf8 in LSP

### Tests

- fuzz json parsing

## [0.6.0](https://github.com/20jasper/JJPWRGEM/compare/jjpwrgem-v0.5.5...jjpwrgem-v0.6.0) - 2026-05-24

### Added

- [**breaking**] inline objects based on width
- [**breaking**] number only array will fill format instead of expanding
- [**breaking**] prettier number normalization
- improve exponent diagnostics
- provide rich utf8 error context to the cli
- [**breaking**] change pretty default indent to 2 spaces
- [**breaking**] non empty arrays put items on newline

### Documentation

- add xtask to generate readmes

### Fixed

- jjpwrgem crate include snapshots ([#278](https://github.com/20jasper/JJPWRGEM/pull/278))
- account for delimiters in array width measurement
- parse value visitor now handles object key events properly
- spacial case empty object to be on one line
- *(deps)* update rust crate toml to v1 ([#168](https://github.com/20jasper/JJPWRGEM/pull/168))

### Performance

- remove iterator abstraction layers
- bytewise parsing for exponents
- bytewise parsing for mantissa
- *(parse)* skip whitespace with portable SIMD (u8x32)
- *(parse)* replace next_if whitespace loop with peek fast path + byte scan
- uglify_serializable now uses faster serialization from the crate instead of deferring to serde_json

### Tests

- add prettier conformance suite and test runners
- add gungraun instruction-count benchmarks

## [0.5.5](https://github.com/20jasper/JJPWRGEM/releases/tag/jjpwrgem-v0.5.5) - 2026-4-12

### Added

- Add bytes2chars crate
- Add InvalidSequenceLength error variant
- Add prettify_value_into and uglify_value_into
- Provide rich utf8 error context to the cli

### Documentation

- Fix changelog generation to include breaking features
- Organize benchmarks into multiple files, and replace manual throughput charts
- Reflect CPU requirements in readme and update stability

### Fixed

- Parse value visitor now handles object key events properly

### Performance

- Uglify_serializable now uses faster serialization from the crate instead of deferring to serde_json
- Replace next_if whitespace loop with peek fast path + byte scan
- Skip whitespace with portable SIMD (u8x32)

## [0.5.4](https://github.com/20jasper/JJPWRGEM/releases/tag/jjpwrgem-v0.5.4) - 2025-12-23

### Added

- experimental vscode extension

### Performance

- don't build ast when validating syntax
- don't build ast when uglifying

```bash
Summary
  jjp format -u  < xtask/bench/data/json-benchmark/data/canada.json ran
    1.50 ± 0.05 times faster than jjpv0.5.3 format -u  < xtask/bench/data/json-benchmark/data/canada.json
  jjp check  < xtask/bench/data/json-benchmark/data/canada.json ran
    1.64 ± 0.04 times faster than jjpv0.5.3 check  < xtask/bench/data/json-benchmark/data/canada.json
  jjpv0.5.3 format   < xtask/bench/data/json-benchmark/data/canada.json ran
    1.00 ± 0.03 times faster than jjp format  < xtask/bench/data/json-benchmark/data/canada.json
```

## [0.5.3](https://github.com/20jasper/JJPWRGEM/releases/tag/jjpwrgem-v0.5.2) - 2025-12-15

### fixed

- removed tarball from npm release

## [0.5.2](https://github.com/20jasper/JJPWRGEM/releases/tag/jjpwrgem-v0.5.2) - 2025-12-15

### Added

- use custom npm package builder and release instead of dists'

### Removed

- unused console.table in npm installer

## [0.5.1](https://github.com/20jasper/JJPWRGEM/releases/tag/jjpwrgem-v0.5.1) - 2025-12-14

### Added

- end of line option

## [0.5.0](https://github.com/20jasper/JJPWRGEM/releases/tag/jjpwrgem-v0.5.0) - 2025-12-13

### Added

- preferred width

### Documentation

- *perf*: npm installation overhead
- *perf*: add benchmarks docs and include throughput and speed benchmarks

### Performance
- benchmarks for uglification and prettifying with various CLI tools

## [0.4.0](https://github.com/20jasper/JJPWRGEM/releases/tag/jjpwrgem-v0.4.0) - 2025-12-10

### Changed

- default prettified indentation is now two spaces
- non-empty arrays write each item on its own line
- keep empty objects on a single line

### Tests

- add coverage for hard-to-format inputs
- add regression test for deeply nested JSON

## [0.3.3](https://github.com/20jasper/JJPWRGEM/releases/tag/jjpwrgem-v0.3.3) - 2025-12-09

### Fixed

- show error message when no input comes to stdin

### Performance

- only cache successful tokens in TokenStream
- track count of digits instead of vec of chars

## [0.3.2](https://github.com/20jasper/JJPWRGEM/releases/tag/jjpwrgem-v0.3.2) - 2025-12-08

### Performance

- write delimiters and indentation directly to buffer, avoiding intermediate allocations

## [0.3.1](https://github.com/20jasper/JJPWRGEM/releases/tag/jjpwrgem-v0.3.1) - 2025-12-08

### Performance

- avoid using fmt machinery in hot paths, instead pushing directly

## [0.3.0](https://github.com/20jasper/JJPWRGEM/releases/tag/jjpwrgem-v0.3.0) - 2025-12-08

### Added

- axolotl logo in version screen
- consistent key ordering

### Deprecated

- removed help subcommand

### Documentation

- autogenerate examples and add examples to subcommands
- update readme with correct command
- add xtask to generate readmes

### Performance

- TokenStream iterator instead of collecting into intermediary Vec


## [0.2.2](https://github.com/20jasper/JJPWRGEM/releases/tag/jjpwrgem-v0.2.2) - 2025-12-07

### Documentation

- add mise installer steps
- update readme with new command format and installation instructions. removes extra notes

### Performance

- join_into utility to declaratively avoid allocating delimiter strings
- write to single buffer instead of allocating buffer per JSON value
- don't use anstream for content without ansi

## [0.2.0](https://github.com/20jasper/JJPWRGEM/releases/tag/jjpwrgem-v0.2.0) - 2025-12-06

### Added

- subcommands - check and format with uglify flag


## [0.1.5](https://github.com/20jasper/JJPWRGEM/releases/tag/jjpwrgem-v0.1.5) - 2025-12-05

Test for publishing flow


## [0.1.4](https://github.com/20jasper/JJPWRGEM/releases/tag/jjpwrgem-v0.1.4) - 2025-12-05

### Feature
- pretty format JSON
- error messages on failure

