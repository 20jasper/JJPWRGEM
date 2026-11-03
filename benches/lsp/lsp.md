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

| method           | jjpwrgem        | vscode-json       |
| ---------------- | --------------- | ----------------- |
| initialize       | 18.5ms (4.0 MB) | 114.1ms (73.7 MB) |
| diagnostic       | 0.1ms (3.9 MB)  | 504.4ms (73.9 MB) |
| formatting       | 0.0ms (3.9 MB)  | 0.2ms (73.4 MB)   |
| formatting edits | 1 edit          | 21 edits          |

## twitter

[twitter.json](/benches/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem        | vscode-json       |
| ---------------- | --------------- | ----------------- |
| initialize       | 20.0ms (4.1 MB) | 113.2ms (74.1 MB) |
| diagnostic       | 4.6ms (6.4 MB)  | 521.6ms (84.5 MB) |
| formatting       | 1.6ms (6.2 MB)  | 24.4ms (84.0 MB)  |
| formatting edits | 1 edit          | 15481 edits       |

## citm catalog

[citm_catalog.json](/benches/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem        | vscode-json       |
| ---------------- | --------------- | ----------------- |
| initialize       | 20.1ms (4.1 MB) | 115.1ms (73.9 MB) |
| diagnostic       | 7.9ms (8.8 MB)  | 541.5ms (99.2 MB) |
| formatting       | 2.3ms (8.7 MB)  | 88.5ms (98.9 MB)  |
| formatting edits | 1 edit          | 50468 edits       |

## canada

[canada.json](/benches/data/canada.json) — 2.2MB, lots of lightly nested arrays, no strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem         | vscode-json        |
| ---------------- | ---------------- | ------------------ |
| initialize       | 19.5ms (3.9 MB)  | 116.1ms (73.9 MB)  |
| diagnostic       | 17.9ms (19.2 MB) | 576.2ms (128.0 MB) |
| formatting       | 6.2ms (19.0 MB)  | 316.7ms (123.4 MB) |
| formatting edits | 1 edit           | 223227 edits       |
