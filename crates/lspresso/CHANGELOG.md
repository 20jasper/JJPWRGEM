# lspresso

## 0.2.0 — 2026-11-10

### Added

- [beb36cd](https://github.com/20jasper/JJPWRGEM/commit/beb36cd9700ee0ca6498299a310211b7389360b7) add capabilities for vscode, emacs, neovim and helix
- [205c4a3](https://github.com/20jasper/JJPWRGEM/commit/205c4a3f339c3a708a7390958ad1aef5eff18a0c) add retrieval of pushed diagnostics
- [f0bc149](https://github.com/20jasper/JJPWRGEM/commit/f0bc149c46affe539e5a567eed0b08dfadac8b4e) add document tracking and relevant protocol and convenience methods

### Changed

- [8b3b64d](https://github.com/20jasper/JJPWRGEM/commit/8b3b64de129c16915923a83220e927f7436c803c) **⚠️ breaking change:** Protocol trait to expose full configuration and higher level convenience layer is now default
- [6eb283b](https://github.com/20jasper/JJPWRGEM/commit/6eb283b14c59651f6ad5f353ca776c46bef635f4) **⚠️ breaking change:** track document text, negotiate the server's position encoding, add `did_change` for incremental edits, and include the tracked text in `did_save` when the server asks for it. `did_save` now takes only the URI.

