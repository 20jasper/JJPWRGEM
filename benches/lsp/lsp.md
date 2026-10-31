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
| initialize       | 6.6ms (3.6 MB) | 137.6ms (72.7 MB) |
| diagnostic       | 0.2ms (3.8 MB) | 510.2ms (72.8 MB) |
| formatting       | 0.1ms (3.7 MB) | 0.5ms (72.5 MB)   |
| formatting edits | 1 edit         | 21 edits          |

## twitter

[twitter.json](/benches/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem        | vscode-json       |
| ---------------- | --------------- | ----------------- |
| initialize       | 10.7ms (3.7 MB) | 146.9ms (73.3 MB) |
| diagnostic       | 6.5ms (6.1 MB)  | 542.3ms (82.6 MB) |
| formatting       | 2.2ms (5.9 MB)  | 49.5ms (82.2 MB)  |
| formatting edits | 1 edit          | 15481 edits       |

## citm catalog

[citm_catalog.json](/benches/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem        | vscode-json       |
| ---------------- | --------------- | ----------------- |
| initialize       | 7.9ms (3.7 MB)  | 138.5ms (73.1 MB) |
| diagnostic       | 11.2ms (8.7 MB) | 556.4ms (97.8 MB) |
| formatting       | 3.1ms (8.4 MB)  | 200.7ms (95.8 MB) |
| formatting edits | 1 edit          | 50468 edits       |

## canada

[canada.json](/benches/data/canada.json) — 2.2MB, lots of lightly nested arrays, no strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem         | vscode-json        |
| ---------------- | ---------------- | ------------------ |
| initialize       | 6.7ms (3.6 MB)   | 136.8ms (72.7 MB)  |
| diagnostic       | 29.0ms (19.0 MB) | 603.6ms (125.0 MB) |
| formatting       | 7.5ms (18.8 MB)  | 823.5ms (122.3 MB) |
| formatting edits | 1 edit           | 223227 edits       |
