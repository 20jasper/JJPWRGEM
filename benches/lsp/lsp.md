<!-- GENERATED FILE - update the templates in the xtask -->

# LSP benchmarks

Wall-clock timing and memory usage against VSCode's JSON LSP via `lsp-bench`. Run locally with `mise run bench-lsp-all`

These benchmarks are run with `AMD Ryzen 5 5600X 6-Core Processor (3.70 GHz)`

VSCode's diagnostics calculation has a 500ms delay for push only diagnostics. The LSP client can pull as of LSP 3.17 (2022), so most modern editors request diagnostics earlier than 500ms

JJPWRGEM does not delay anything

## small

[small.json](/benches/data/small.json) — 163B, small nested objects and arrays, mixed scalar values

Baseline performance for minimal parsing work

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem       | vscode-json       |
| ---------------- | -------------- | ----------------- |
| initialize       | 7.0ms (3.7 MB) | 135.7ms (73.6 MB) |
| diagnostic       | 0.2ms (3.8 MB) | 508.6ms (73.4 MB) |
| formatting       | 0.2ms (3.8 MB) | 0.4ms (72.5 MB)   |
| formatting edits | 1 edit         | 21 edits          |

## twitter

[twitter.json](/benches/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem       | vscode-json       |
| ---------------- | -------------- | ----------------- |
| initialize       | 8.5ms (3.7 MB) | 161.3ms (73.3 MB) |
| diagnostic       | 7.5ms (6.1 MB) | 543.5ms (82.5 MB) |
| formatting       | 2.4ms (5.9 MB) | 53.0ms (82.4 MB)  |
| formatting edits | 1 edit         | 15481 edits       |

## citm catalog

[citm_catalog.json](/benches/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem        | vscode-json       |
| ---------------- | --------------- | ----------------- |
| initialize       | 7.1ms (3.7 MB)  | 142.8ms (72.9 MB) |
| diagnostic       | 11.2ms (8.6 MB) | 557.9ms (97.8 MB) |
| formatting       | 3.2ms (8.6 MB)  | 202.1ms (96.4 MB) |
| formatting edits | 1 edit          | 50468 edits       |

## canada

[canada.json](/benches/data/canada.json) — 2.2MB, lots of lightly nested arrays, no strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem         | vscode-json        |
| ---------------- | ---------------- | ------------------ |
| initialize       | 7.0ms (3.7 MB)   | 136.7ms (73.1 MB)  |
| diagnostic       | 28.8ms (19.0 MB) | 603.1ms (124.9 MB) |
| formatting       | 8.1ms (18.8 MB)  | 838.8ms (124.8 MB) |
| formatting edits | 1 edit           | 223227 edits       |
