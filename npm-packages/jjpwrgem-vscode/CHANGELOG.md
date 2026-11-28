# Change Log

All notable changes to the "jjpwrgem-vscode" extension will be documented in this file.

Check [Keep a Changelog](http://keepachangelog.com/) for recommendations on how to structure this file.

## [Unreleased]

## 0.1.1 — 2026-11-28

### Added

- [e063070](https://github.com/20jasper/JJPWRGEM/commit/e0630703268c9fce34e02d4bbfbf583970b51b19) add the `jjpwrgem.trace.server` setting.

## 0.1.0 — 2026-11-25

### Added

- [5f75684](https://github.com/20jasper/JJPWRGEM/commit/5f756844c645a5fc1d9a80a2d1e8aa938f9712e3) support jsonc diagnostics.
- [e42c731](https://github.com/20jasper/JJPWRGEM/commit/e42c731a9029f960e542fd08818364d974768e8c) add selection ranges.

### Changed

- [12cb8b9](https://github.com/20jasper/JJPWRGEM/commit/12cb8b9ca8029fb8af523de05201284919430a0d) update error message about failing to start jjp when a custom path is set.

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