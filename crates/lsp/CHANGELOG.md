# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## 0.1.7 — 2026-11-21

### Performance

- [001e1c1](https://github.com/20jasper/JJPWRGEM/commit/001e1c175137ce8c4017dab2dc8b2b8b38b2cacc) migrate off of tower-lsp-server to a customized lsp-server, removing Tokio and avoiding intermediate deserialization. Uses 1.6x less RAM and benches are on average 1.7x faster. Binary is 31% smaller.

## 0.1.6 — 2026-11-17

### Documentation

- [4ee81fa](https://github.com/20jasper/JJPWRGEM/commit/4ee81fa8e591e767287bd3831b88316bb5ef6350) docs: document the format settings in the extension and language server readmes, with a Helix example

### Fixed

- [d209a12](https://github.com/20jasper/JJPWRGEM/commit/d209a12ee24961da18226ec88b7228b10b4bd9fd) fix broken doc links

### Added

- [83336de](https://github.com/20jasper/JJPWRGEM/commit/83336de3065bd62a2ec970b787e1373900d4d9ac) the language server and VS Code extension read `jjpwrgem.format` settings (`mode`, `preferredWidth`, `endOfLine`) from initialization options and `workspace/didChangeConfiguration`. Settings that do not apply are ignored instead of failing

## 0.1.5 — 2026-11-13

### Added

- [e290179](https://github.com/20jasper/JJPWRGEM/commit/e290179a9384994623b0e60a50aa42e25135fc9a) add folding ranges
- [515dd1f](https://github.com/20jasper/JJPWRGEM/commit/515dd1f455756ba196192aadf51a484f9581ef3b) add document symbols

## 0.1.4 — 2026-11-12

### Added

- [dad7da6](https://github.com/20jasper/JJPWRGEM/commit/dad7da64c0f6d15ef823f2743b106b199a0059e4) add pull diagnostics

### Documentation

- [8cd1606](https://github.com/20jasper/JJPWRGEM/commit/8cd160680d871e923a1a36eb71cebea3f6287b20) docs: update performance claims

### Performance

- [9f2a798](https://github.com/20jasper/JJPWRGEM/commit/9f2a798305ece123ccad6e98a04ca7ed82fe8a98) use incremental document sync in the LSP with the line-index crate for edit position conversion, avoiding a full-document transfer on every edit. In LSP diagnostics benchmarks, peak memory is up to around 25% lower on the largest document.

## [0.1.3](https://github.com/20jasper/JJPWRGEM/compare/jjpwrgem-lsp-v0.1.2...jjpwrgem-lsp-v0.1.3) - 2026-10-26

### Documentation

- add vscode extension URL
- update vscode and LSP perf claims
- add metadata and docs for vscode extension

## [0.1.1](https://github.com/20jasper/JJPWRGEM/compare/jjpwrgem-lsp-v0.1.0...jjpwrgem-lsp-v0.1.1) - 2026-05-24

### Added

- add lsp with diagnostics, formatting, and code actions

### Deprecated

- deprecated!(parse): removed line and column numbers from error and display impl

### Fixed

- support utf16 and utf8 in LSP
