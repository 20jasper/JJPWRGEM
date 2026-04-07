<!-- GENERATED FILE - update the templates in the xtask -->

# jjpwrgem JSON benchmarks

jjpwrgem is optimized for readonly operations on cached syntax trees. Notably pretty serialization from a syntax tree is 4 times faster than `sonic_rs` while also supporting width aware expansion. Deserialization is not yet fully optimized

Throughput benchmarks for deserializing into a syntax tree, serializing the syntax tree, and streaming serialization and deserialization

Run locally with `just bench-json` or individual `just bench json_deser`, `just bench json_prettify`, and `just bench json_uglify`

Throughput is normalized by input and output bytes and benchmarks do not measure initial buffer allocation

The following JSON fixtures are used across benchmarks:

- [canada.json](/xtask/bench/data/json-benchmark/data/canada.json) — 2.2MB, lots of lightly nested arrays, no strings

- [citm_catalog.json](/xtask/bench/data/json-benchmark/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

- [twitter.json](/xtask/bench/data/json-benchmark/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

## deser

|                           |  `jjpwrgem`  | `serde_json` | `simd_json`  |  `sonic_rs`   |
| :------------------------ | :----------: | :----------: | :----------: | :-----------: |
| time (canada)             |   21.28 ms   |   12.75 ms   |   4.44 ms    |    2.15 ms    |
| throughput (canada)       | 100.87 MiB/s | 168.36 MiB/s | 483.03 MiB/s | 1000.02 MiB/s |
| time (citm_catalog)       |   4.76 ms    |   3.13 ms    |   1.73 ms    |   983.79 µs   |
| throughput (citm_catalog) | 345.82 MiB/s | 525.71 MiB/s | 952.66 MiB/s |  1.64 GiB/s   |
| time (twitter)            |   2.47 ms    |   1.81 ms    |  769.15 µs   |   374.93 µs   |
| throughput (twitter)      | 243.45 MiB/s | 331.87 MiB/s | 783.02 MiB/s |  1.57 GiB/s   |

## prettify_ast

|                           |  `sonic_rs`  | `serde_json` | `simd_json`  | `jjpwrgem` |
| :------------------------ | :----------: | :----------: | :----------: | :--------: |
| time (canada)             |   6.96 ms    |   5.77 ms    |   5.65 ms    |  1.79 ms   |
| throughput (canada)       | 308.48 MiB/s | 372.34 MiB/s | 379.86 MiB/s | 1.17 GiB/s |
| time (citm_catalog)       |   1.43 ms    |   1.38 ms    |   1.05 ms    |  1.00 ms   |
| throughput (citm_catalog) |  1.12 GiB/s  |  1.16 GiB/s  |  1.53 GiB/s  | 1.60 GiB/s |
| time (twitter)            |  499.98 µs   |  602.77 µs   |  477.83 µs   | 323.02 µs  |
| throughput (twitter)      |  1.18 GiB/s  | 999.15 MiB/s |  1.23 GiB/s  | 1.82 GiB/s |

## uglify_ast

|                           |  `sonic_rs`  | `simd_json`  | `serde_json` | `jjpwrgem` |
| :------------------------ | :----------: | :----------: | :----------: | :--------: |
| time (canada)             |   3.59 ms    |   3.25 ms    |   2.76 ms    | 771.60 µs  |
| throughput (canada)       | 597.63 MiB/s | 661.04 MiB/s | 777.91 MiB/s | 2.72 GiB/s |
| time (citm_catalog)       |  652.34 µs   |  477.33 µs   |  530.58 µs   | 283.04 µs  |
| throughput (citm_catalog) |  2.47 GiB/s  |  3.37 GiB/s  |  3.03 GiB/s  | 5.68 GiB/s |
| time (twitter)            |  286.59 µs   |  355.21 µs   |  383.72 µs   | 133.60 µs  |
| throughput (twitter)      |  2.05 GiB/s  |  1.66 GiB/s  |  1.53 GiB/s  | 4.40 GiB/s |

## uglify_tokens

|                           |  `jjpwrgem`  |
| :------------------------ | :----------: |
| time (canada)             |   12.38 ms   |
| throughput (canada)       | 173.43 MiB/s |
| time (citm_catalog)       |   4.01 ms    |
| throughput (citm_catalog) | 410.33 MiB/s |
| time (twitter)            |   2.19 ms    |
| throughput (twitter)      | 275.31 MiB/s |
