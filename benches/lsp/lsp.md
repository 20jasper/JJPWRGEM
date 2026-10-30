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
| initialize       | 6.2ms (3.7 MB) | 123.2ms (72.6 MB) |
| diagnostic       | 0.2ms (3.8 MB) | 505.4ms (72.7 MB) |
| formatting       | 0.1ms (3.7 MB) | 0.4ms (72.8 MB)   |
| formatting edits | 1 edit         | 21 edits          |

## twitter

[twitter.json](/benches/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem       | vscode-json       |
| ---------------- | -------------- | ----------------- |
| initialize       | 8.3ms (3.7 MB) | 129.3ms (73.5 MB) |
| diagnostic       | 7.0ms (6.6 MB) | 525.8ms (82.8 MB) |
| formatting       | 2.1ms (6.5 MB) | 44.5ms (81.9 MB)  |
| formatting edits | 1 edit         | 15481 edits       |

## citm catalog

[citm_catalog.json](/benches/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem         | vscode-json       |
| ---------------- | ---------------- | ----------------- |
| initialize       | 8.4ms (3.7 MB)   | 125.2ms (73.2 MB) |
| diagnostic       | 10.9ms (10.2 MB) | 545.6ms (97.8 MB) |
| formatting       | 3.0ms (10.0 MB)  | 175.1ms (97.6 MB) |
| formatting edits | 1 edit           | 50468 edits       |

## canada

[canada.json](/benches/data/canada.json) — 2.2MB, lots of lightly nested arrays, no strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem         | vscode-json        |
| ---------------- | ---------------- | ------------------ |
| initialize       | 6.4ms (3.7 MB)   | 124.7ms (73.1 MB)  |
| diagnostic       | 28.8ms (24.0 MB) | 585.8ms (125.3 MB) |
| formatting       | 7.9ms (23.9 MB)  | 736.1ms (122.0 MB) |
| formatting edits | 1 edit           | 223227 edits       |
