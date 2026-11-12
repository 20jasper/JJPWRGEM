# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
