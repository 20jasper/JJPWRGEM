### mise

```bash
mise use -g github:20jasper/jjpwrgem{{AT_VERSION}}
```

### Cargo

```bash
cargo install jjpwrgem{{CARGO_VERSION}} --locked
```

### npm

The install script must run to fetch the native binary.

npm 11 and newer:

```bash
npm install -g jjpwrgem{{AT_VERSION}} --allow-scripts=jjpwrgem
```

npm 10 and older:

```bash
npm install -g jjpwrgem{{AT_VERSION}}
```

Or download a binary from [releases](https://github.com/20jasper/JJPWRGEM/releases).
