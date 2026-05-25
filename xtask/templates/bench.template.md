# jjpwrgem benchmarks

[![See codspeed dashboard](https://img.shields.io/endpoint?url=https://codspeed.io/badge.json)](https://codspeed.io/20jasper/JJPWRGEM?utm_source=badge)

{{BENCH_SUMMARY}}

- [LSP memory and wall-clock timing](lsp/README.md): against vscode-json
- [CLI formatter and minifier speed](cli-formatter.md): wall-clock timing against jq, prettier, dprint, and others
- [JSON syntax tree serialization and deserialization throughput](json.md): throughput and wall clock benchmarks vs serde_json, simd_json, and sonic-rs
- [lazy UTF-8 decode throughput](utf8.md): `bytes2chars` vs `utf8-decode` and `bstr`

## Running

Instruction count benchmarks require [Valgrind](https://valgrind.org) 3.20+ and `gungraun-runner`
