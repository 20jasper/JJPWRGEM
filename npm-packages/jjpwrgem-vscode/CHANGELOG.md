# Change Log

All notable changes to the "jjpwrgem-vscode" extension will be documented in this file.

Check [Keep a Changelog](http://keepachangelog.com/) for recommendations on how to structure this file.

## [Unreleased]

## 0.0.7 — 2026-11-17

### Documentation

- [4ee81fa](https://github.com/20jasper/JJPWRGEM/commit/4ee81fa8e591e767287bd3831b88316bb5ef6350) docs: document the format settings in the extension and language server readmes, with a Helix example

### Added

- [83336de](https://github.com/20jasper/JJPWRGEM/commit/83336de3065bd62a2ec970b787e1373900d4d9ac) the language server and VS Code extension read `jjpwrgem.format` settings (`mode`, `preferredWidth`, `endOfLine`) from initialization options and `workspace/didChangeConfiguration`. Settings that do not apply are ignored instead of failing

## [0.0.6] - 2026-11-11

- Update the README with current large-file performance claims

## [0.0.5] - 2026-10-26

- Add `jjpwrgem.path` setting to choose the `jjp` executable

## [0.0.1] - 2026-06-04

- Initial release