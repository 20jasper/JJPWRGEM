<!-- GENERATED FILE - update the templates in the xtask -->

# jjpwrgem JSON benchmarks

jjpwrgem is optimized for readonly operations on cached syntax trees. Notably pretty serialization from a syntax tree is 3 times faster than `sonic_rs` while also supporting width aware expansion. Deserialization is not yet fully optimized

Throughput benchmarks for deserializing into a syntax tree, serializing the syntax tree, and streaming serialization and deserialization

Run locally with `mise run bench-json` or individual `mise run bench json_deser`, `mise run bench json_prettify`, and `mise run bench json_uglify`

Throughput is normalized by input and output bytes and benchmarks do not measure initial buffer allocation

The following JSON fixtures are used across benchmarks:

- [canada.json](/benches/data/canada.json) — 2.2MB, lots of lightly nested arrays, no strings

- [citm_catalog.json](/benches/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

- [twitter.json](/benches/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

## deser

|                           | `serde_json` |  `jjpwrgem`  | `simd_json`  | `sonic_rs` |
| :------------------------ | :----------: | :----------: | :----------: | :--------: |
| time (canada)             |   8.47 ms    |   9.29 ms    |   4.03 ms    |  2.58 ms   |
| throughput (canada)       | 342.31 MiB/s | 312.13 MiB/s | 719.81 MiB/s | 1.10 GiB/s |
| time (citm_catalog)       |   2.49 ms    |   2.27 ms    |   1.41 ms    | 795.61 µs  |
| throughput (citm_catalog) | 437.37 MiB/s | 478.21 MiB/s | 770.73 MiB/s | 1.33 GiB/s |
| time (twitter)            |   1.62 ms    |  884.36 µs   |  688.02 µs   | 376.94 µs  |
| throughput (twitter)      | 366.73 MiB/s | 673.44 MiB/s | 865.62 MiB/s | 1.54 GiB/s |

## prettify_ast

|                           |  `sonic_rs`  | `serde_json` | `simd_json`  | `jjpwrgem` |
| :------------------------ | :----------: | :----------: | :----------: | :--------: |
| time (canada)             |   5.83 ms    |   5.53 ms    |   5.17 ms    |  2.00 ms   |
| throughput (canada)       | 496.85 MiB/s | 524.11 MiB/s | 560.69 MiB/s | 1.42 GiB/s |
| time (citm_catalog)       |   1.31 ms    |   1.31 ms    |   1.01 ms    | 894.83 µs  |
| throughput (citm_catalog) | 831.80 MiB/s | 828.46 MiB/s |  1.05 GiB/s  | 1.19 GiB/s |
| time (twitter)            |  442.70 µs   |  598.65 µs   |  466.76 µs   | 340.87 µs  |
| throughput (twitter)      |  1.31 GiB/s  | 994.83 MiB/s |  1.25 GiB/s  | 1.71 GiB/s |

## uglify_ast

|                           | `simd_json`  | `serde_json` | `sonic_rs` | `jjpwrgem` |
| :------------------------ | :----------: | :----------: | :--------: | :--------: |
| time (canada)             |   3.13 ms    |   2.49 ms    |  2.42 ms   | 804.26 µs  |
| throughput (canada)       | 927.36 MiB/s |  1.14 GiB/s  | 1.17 GiB/s | 3.52 GiB/s |
| time (citm_catalog)       |  463.02 µs   |  470.42 µs   | 505.90 µs  | 307.33 µs  |
| throughput (citm_catalog) |  2.29 GiB/s  |  2.26 GiB/s  | 2.10 GiB/s | 3.45 GiB/s |
| time (twitter)            |  398.96 µs   |  338.31 µs   | 245.14 µs  | 134.32 µs  |
| throughput (twitter)      |  1.46 GiB/s  |  1.72 GiB/s  | 2.37 GiB/s | 4.33 GiB/s |

## uglify_tokens

|                           |  `jjpwrgem`  |
| :------------------------ | :----------: |
| time (canada)             |   4.85 ms    |
| throughput (canada)       | 597.75 MiB/s |
| time (citm_catalog)       |   1.69 ms    |
| throughput (citm_catalog) | 643.51 MiB/s |
| time (twitter)            |  749.12 µs   |
| throughput (twitter)      | 795.01 MiB/s |
