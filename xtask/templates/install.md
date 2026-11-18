### mise

```bash
mise use -g github:20jasper/jjpwrgem{{AT_VERSION}}
```

### Cargo

```bash
cargo install jjpwrgem{{CARGO_VERSION}} --locked
```

### npm

```bash
# script points jjp directly at a native binary, avoiding 30ms of node process startup
npm install -g jjpwrgem{{AT_VERSION}} --allow-scripts=jjpwrgem
pnpm add -g jjpwrgem{{AT_VERSION}} --allow-build=jjpwrgem
```

Or download a binary from [releases](https://github.com/20jasper/JJPWRGEM/releases).
