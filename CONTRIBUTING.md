# Contributing

## Quick start

Install the following

- [Rust](https://rust-lang.org/tools/install/)
- [mise](https://mise.jdx.dev/installing-mise.html)

Install global dependencies

```bash
mise trust
mise use -g github:cargo-bins/cargo-binstall # allows mise to grab prebuilt binaries instead of building from source
mise i
mise submodules
```

## Scripts

Mise will lazily install other tools as needed. Run `mise tasks` for a full list

This project uses `hk` for git hooks, which can be run on demand via `hk check --all` and `hk fix --all`

### Testing

```bash
mise run test
```

Much testing is done via snapshot tests, if any need review, run the following

```bash
cargo insta review
```

### Local Installation

Install `jjp` on your PATH for easier testing

```bash
mise run install
```

## Changesets

Uses [sampo](https://github.com/bruits/sampo) changesets. Run `mise run changeset` (`sampo add`) and mark the affected packages like so

```md
---
jjpwrgem-parse: patch (Performance)
jjpwrgem: patch (Performance)
---

Scan strings with SIMD
```
