<!-- GENERATED FILE - update the templates in the xtask -->

# LSP benchmarks

Wall-clock timing and memory usage against VSCode's JSON LSP via `lsp-bench`. Run locally with `mise run bench-lsp-all`

These benchmarks are run with `AMD Ryzen 5 5600X 6-Core Processor (3.70 GHz)`

## small

[small.json](/benches/data/small.json) — 163B, small nested objects and arrays, mixed scalar values

Baseline performance for minimal parsing work. VSCode's diagnostics calculation has a 500ms delay

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem        | vscode-json       |
| ---------------- | --------------- | ----------------- |
| initialize       | 12.5ms (4.6 MB) | 149.3ms (71.0 MB) |
| diagnostic       | 0.2ms (4.6 MB)  | 509.6ms (71.3 MB) |
| formatting       | 0.2ms (4.5 MB)  | 0.5ms (70.2 MB)   |
| formatting edits | 1 edit          | 21 edits          |

## twitter

[twitter.json](/benches/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem        | vscode-json       |
| ---------------- | --------------- | ----------------- |
| initialize       | 12.9ms (4.6 MB) | 165.1ms (71.1 MB) |
| diagnostic       | 10.0ms (7.4 MB) | 541.8ms (80.9 MB) |
| formatting       | 2.9ms (7.4 MB)  | 58.3ms (80.1 MB)  |
| formatting edits | 1 edit          | 15481 edits       |

## citm catalog

[citm_catalog.json](/benches/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem         | vscode-json       |
| ---------------- | ---------------- | ----------------- |
| initialize       | 12.6ms (4.6 MB)  | 152.3ms (71.2 MB) |
| diagnostic       | 17.1ms (11.0 MB) | 573.3ms (96.0 MB) |
| formatting       | 4.4ms (10.8 MB)  | 116.7ms (93.9 MB) |
| formatting edits | 1 edit           | 50468 edits       |

## canada

[canada.json](/benches/data/canada.json) — 2.2MB, lots of lightly nested arrays, no strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem         | vscode-json        |
| ---------------- | ---------------- | ------------------ |
| initialize       | 12.5ms (4.6 MB)  | 147.7ms (71.7 MB)  |
| diagnostic       | 45.6ms (24.8 MB) | 618.6ms (123.1 MB) |
| formatting       | 9.8ms (24.7 MB)  | 477.2ms (120.4 MB) |
| formatting edits | 1 edit           | 223227 edits       |
