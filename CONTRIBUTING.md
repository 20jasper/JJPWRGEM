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

## Releases

Releases are driven by [sampo](https://github.com/bruits/sampo) changesets. Add one to any PR that changes a published crate

```bash
mise run changeset
```

Pick each crate the change touches and a bump, tagged with its changelog section, e.g. `cargo/jjpwrgem-parse: patch (Performance)`. Name `cargo/jjpwrgem` too when CLI users should see the change. The description becomes the changelog entry

Merged changesets collect in a release PR, and merging that PR publishes to crates.io, GitHub releases and npm. `mise run release-plan` shows what the next release would bump
