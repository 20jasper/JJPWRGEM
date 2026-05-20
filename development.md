# development

## quick start

Install [`mise`](https://mise.jdx.dev/)

```sh
curl https://mise.run | sh
mise trust
```

`mise` is used for development tasks and versioning. `mise tasks` will show everything available and install proper versions on your system

## Releases

Uses `cargo-dist` to build and release binaries on multiple platforms. Triggered when a tag is made by `release-plz`
