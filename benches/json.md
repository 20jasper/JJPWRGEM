<!-- GENERATED FILE - update the templates in the xtask -->

# jjpwrgem JSON benchmarks

jjpwrgem is optimized for readonly operations on cached syntax trees. Notably pretty serialization from a syntax tree is 4 times faster than `sonic_rs` while also supporting width aware expansion. Deserialization is not yet fully optimized

Throughput benchmarks for deserializing into a syntax tree, serializing the syntax tree, and streaming serialization and deserialization

Run locally with `mise run bench-json` or individual `mise run bench json_deser`, `mise run bench json_prettify`, and `mise run bench json_uglify`

Throughput is normalized by input and output bytes and benchmarks do not measure initial buffer allocation

The following JSON fixtures are used across benchmarks:

- [canada.json](/benches/docker/data/json-benchmark/data/canada.json) — 2.2MB, lots of lightly nested arrays, no strings

- [citm_catalog.json](/benches/docker/data/json-benchmark/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

- [twitter.json](/benches/docker/data/json-benchmark/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

## deser

|                           |  `jjpwrgem`  | `serde_json` | `simd_json`  |  `sonic_rs`  |
| :------------------------ | :----------: | :----------: | :----------: | :----------: |
| time (canada)             |   19.62 ms   |   13.11 ms   |   7.82 ms    |   2.23 ms    |
| throughput (canada)       | 109.44 MiB/s | 163.73 MiB/s | 274.58 MiB/s | 960.94 MiB/s |
| time (citm_catalog)       |   3.47 ms    |   3.13 ms    |   1.68 ms    |   1.04 ms    |
| throughput (citm_catalog) | 474.52 MiB/s | 526.57 MiB/s | 980.45 MiB/s |  1.54 GiB/s  |
| time (twitter)            |   1.71 ms    |   1.80 ms    |  774.89 µs   |  370.42 µs   |
| throughput (twitter)      | 353.04 MiB/s | 334.23 MiB/s | 777.22 MiB/s |  1.59 GiB/s  |

## prettify_ast

|                           |  `sonic_rs`  | `serde_json` | `simd_json`  | `jjpwrgem` |
| :------------------------ | :----------: | :----------: | :----------: | :--------: |
| time (canada)             |   7.35 ms    |   6.83 ms    |   5.77 ms    |  1.80 ms   |
| throughput (canada)       | 292.02 MiB/s | 314.45 MiB/s | 372.12 MiB/s | 1.16 GiB/s |
| time (citm_catalog)       |   1.51 ms    |   1.44 ms    |   1.08 ms    |  1.01 ms   |
| throughput (citm_catalog) |  1.07 GiB/s  |  1.12 GiB/s  |  1.48 GiB/s  | 1.60 GiB/s |
| time (twitter)            |  514.96 µs   |  632.21 µs   |  497.80 µs   | 342.36 µs  |
| throughput (twitter)      |  1.14 GiB/s  | 952.63 MiB/s |  1.18 GiB/s  | 1.72 GiB/s |

## uglify_ast

|                           |  `sonic_rs`  | `simd_json`  | `serde_json` | `jjpwrgem` |
| :------------------------ | :----------: | :----------: | :----------: | :--------: |
| time (canada)             |   3.81 ms    |   3.30 ms    |   2.94 ms    | 744.76 µs  |
| throughput (canada)       | 563.22 MiB/s | 650.28 MiB/s | 729.10 MiB/s | 2.81 GiB/s |
| time (citm_catalog)       |  633.39 µs   |  488.54 µs   |  507.50 µs   | 295.85 µs  |
| throughput (citm_catalog) |  2.54 GiB/s  |  3.29 GiB/s  |  3.17 GiB/s  | 5.44 GiB/s |
| time (twitter)            |  289.03 µs   |  372.55 µs   |  365.44 µs   | 144.80 µs  |
| throughput (twitter)      |  2.03 GiB/s  |  1.58 GiB/s  |  1.61 GiB/s  | 4.06 GiB/s |

## uglify_tokens

|                           |  `jjpwrgem`  |
| :------------------------ | :----------: |
| time (canada)             |   8.91 ms    |
| throughput (canada)       | 241.00 MiB/s |
| time (citm_catalog)       |   2.64 ms    |
| throughput (citm_catalog) | 622.78 MiB/s |
| time (twitter)            |   1.44 ms    |
| throughput (twitter)      | 419.28 MiB/s |
