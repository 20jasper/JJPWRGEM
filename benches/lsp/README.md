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
| initialize       | 14.5ms (4.6 MB) | 143.9ms (71.5 MB) |
| diagnostic       | 0.2ms (4.8 MB)  | 506.0ms (71.6 MB) |
| formatting       | 0.2ms (4.5 MB)  | 0.5ms (71.0 MB)   |
| formatting edits | 1 edit          | 21 edits          |

## twitter

[twitter.json](/benches/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem        | vscode-json       |
| ---------------- | --------------- | ----------------- |
| initialize       | 15.1ms (4.6 MB) | 143.5ms (71.6 MB) |
| diagnostic       | 9.2ms (7.2 MB)  | 528.2ms (81.1 MB) |
| formatting       | 2.5ms (7.2 MB)  | 55.6ms (80.4 MB)  |
| formatting edits | 1 edit          | 15480 edits       |

## citm catalog

[citm_catalog.json](/benches/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem         | vscode-json        |
| ---------------- | ---------------- | ------------------ |
| initialize       | 12.3ms (4.6 MB)  | 139.6ms (71.4 MB)  |
| diagnostic       | 19.5ms (11.6 MB) | 563.9ms (105.5 MB) |
| formatting       | 3.5ms (11.7 MB)  | 17.5ms (101.9 MB)  |
| formatting edits | 1 edit           | 0 edits            |

## canada

[canada.json](/benches/data/canada.json) — 2.2MB, lots of lightly nested arrays, no strings

Note: memory values are RSS (resident set size), shown in megabytes.

| method           | jjpwrgem         | vscode-json        |
| ---------------- | ---------------- | ------------------ |
| initialize       | 12.5ms (4.6 MB)  | 140.5ms (71.2 MB)  |
| diagnostic       | 34.9ms (21.0 MB) | 579.8ms (124.7 MB) |
| formatting       | 9.1ms (20.8 MB)  | 350.7ms (122.4 MB) |
| formatting edits | 1 edit           | 223229 edits       |
