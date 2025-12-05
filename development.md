# development

## quick start

Install

- rust
- node

Install cargo-binstall

```rs
cargo install cargo-binstall --locked
```

Install just

```rs
cargo binstall just
```

Install the rest of the dev deps

```rs
just dev-install
```

## Releases

Uses `cargo-dist` to build and release binaries on multiple platforms. Triggered when a tag is made by `release-plz`
