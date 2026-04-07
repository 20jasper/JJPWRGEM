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

|                           |  `jjpwrgem`  | `serde_json` | `simd_json`  |  `sonic_rs`  |
| :------------------------ | :----------: | :----------: | :----------: | :----------: |
| time (canada)             |   21.39 ms   |   13.66 ms   |   9.31 ms    |   2.69 ms    |
| throughput (canada)       | 100.36 MiB/s | 157.21 MiB/s | 230.51 MiB/s | 799.29 MiB/s |
| time (citm_catalog)       |   5.20 ms    |   3.69 ms    |   1.95 ms    |   1.08 ms    |
| throughput (citm_catalog) | 316.97 MiB/s | 446.12 MiB/s | 843.10 MiB/s |  1.50 GiB/s  |
| time (twitter)            |   2.69 ms    |   2.02 ms    |  864.46 µs   |  413.18 µs   |
| throughput (twitter)      | 224.08 MiB/s | 297.73 MiB/s | 696.68 MiB/s |  1.42 GiB/s  |

## prettify_ast

|                           |  `sonic_rs`  | `serde_json` | `simd_json`  | `jjpwrgem` |
| :------------------------ | :----------: | :----------: | :----------: | :--------: |
| time (canada)             |   7.04 ms    |   6.30 ms    |   5.53 ms    |  1.75 ms   |
| throughput (canada)       | 305.07 MiB/s | 340.65 MiB/s | 387.95 MiB/s | 1.20 GiB/s |
| time (citm_catalog)       |   1.48 ms    |   1.32 ms    |   1.02 ms    |  1.01 ms   |
| throughput (citm_catalog) |  1.09 GiB/s  |  1.22 GiB/s  |  1.58 GiB/s  | 1.59 GiB/s |
| time (twitter)            |  505.86 µs   |  612.46 µs   |  492.70 µs   | 320.85 µs  |
| throughput (twitter)      |  1.16 GiB/s  | 983.35 MiB/s |  1.19 GiB/s  | 1.83 GiB/s |

## uglify_ast

|                           |  `sonic_rs`  | `simd_json`  | `serde_json` | `jjpwrgem` |
| :------------------------ | :----------: | :----------: | :----------: | :--------: |
| time (canada)             |   3.86 ms    |   3.28 ms    |   2.99 ms    | 778.93 µs  |
| throughput (canada)       | 555.72 MiB/s | 653.98 MiB/s | 718.31 MiB/s | 2.69 GiB/s |
| time (citm_catalog)       |  616.20 µs   |  503.55 µs   |  515.18 µs   | 280.56 µs  |
| throughput (citm_catalog) |  2.61 GiB/s  |  3.19 GiB/s  |  3.12 GiB/s  | 5.73 GiB/s |
| time (twitter)            |  284.97 µs   |  394.73 µs   |  381.14 µs   | 133.70 µs  |
| throughput (twitter)      |  2.06 GiB/s  |  1.49 GiB/s  |  1.54 GiB/s  | 4.40 GiB/s |

## uglify_tokens

|                           |  `jjpwrgem`  |
| :------------------------ | :----------: |
| time (canada)             |   12.17 ms   |
| throughput (canada)       | 176.36 MiB/s |
| time (citm_catalog)       |   4.44 ms    |
| throughput (citm_catalog) | 371.13 MiB/s |
| time (twitter)            |   2.38 ms    |
| throughput (twitter)      | 253.27 MiB/s |
