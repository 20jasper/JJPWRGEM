# lspresso

## 0.2.2 — 2026-11-12

### Added

- [9f2a798](https://github.com/20jasper/JJPWRGEM/commit/9f2a798305ece123ccad6e98a04ca7ed82fe8a98) add a direct incremental document change helper
- [415b94d](https://github.com/20jasper/JJPWRGEM/commit/415b94d4b704bf5b64dee7220abc5aaf00030b30) add granular timings via Client::timings. Splits up time spent receiving, processing, and responding to requests. Real clients vary in speed so split out the language server's work from IO

## 0.2.1 — 2026-11-10

### Changed

- [7ae30cd](https://github.com/20jasper/JJPWRGEM/commit/7ae30cd260b827d14f493cadf55208385317571e) improve test failure ergonomics by display formatting errors instead of debug printing

## 0.2.0 — 2026-11-10

### Added

- [beb36cd](https://github.com/20jasper/JJPWRGEM/commit/beb36cd9700ee0ca6498299a310211b7389360b7) add capabilities for vscode, emacs, neovim and helix
- [205c4a3](https://github.com/20jasper/JJPWRGEM/commit/205c4a3f339c3a708a7390958ad1aef5eff18a0c) add retrieval of pushed diagnostics
- [f0bc149](https://github.com/20jasper/JJPWRGEM/commit/f0bc149c46affe539e5a567eed0b08dfadac8b4e) add document tracking and relevant protocol and convenience methods

### Changed

- [8b3b64d](https://github.com/20jasper/JJPWRGEM/commit/8b3b64de129c16915923a83220e927f7436c803c) **⚠️ breaking change:** Protocol trait to expose full configuration and higher level convenience layer is now default
- [6eb283b](https://github.com/20jasper/JJPWRGEM/commit/6eb283b14c59651f6ad5f353ca776c46bef635f4) **⚠️ breaking change:** track document text, negotiate the server's position encoding, add `did_change` for incremental edits, and include the tracked text in `did_save` when the server asks for it. `did_save` now takes only the URI.

