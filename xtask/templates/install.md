**mise**

```bash
mise use -g github:20jasper/jjpwrgem{{AT_VERSION}}
```

**Cargo**

```bash
cargo install jjpwrgem{{CARGO_VERSION}} --locked
```

**npm**

Not recommended on Windows because of its overhead: on macOS and Linux npm runs the native binary directly, but on Windows every call goes through Node.

npm 11 and newer:

```bash
npm install -g jjpwrgem{{AT_VERSION}} --allow-scripts=jjpwrgem
```

npm 10 and older:

```bash
npm install -g jjpwrgem{{AT_VERSION}}
```

Or download a binary from [releases](https://github.com/20jasper/JJPWRGEM/releases).
