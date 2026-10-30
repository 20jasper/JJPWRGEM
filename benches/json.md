<!-- GENERATED FILE - update the templates in the xtask -->

# jjpwrgem JSON benchmarks

jjpwrgem is optimized for readonly operations on cached syntax trees. Notably pretty serialization from a syntax tree is 3 times faster than `sonic_rs` while also supporting width aware expansion. Deserialization is not yet fully optimized

Throughput benchmarks for deserializing into a syntax tree, serializing the syntax tree, and streaming serialization and deserialization

Run locally with `mise run bench-json` or individual `mise run bench json_deser`, `mise run bench json_prettify`, `mise run bench json_uglify`, and `mise run bench json_serialize`

Throughput is normalized by input and output bytes and benchmarks do not measure initial buffer allocation

The following JSON fixtures are used across benchmarks:

- [canada.json](/benches/data/canada.json) — 2.2MB, lots of lightly nested arrays, no strings

- [citm_catalog.json](/benches/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

- [twitter.json](/benches/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

## deser

|                           | `serde_json` |  `jjpwrgem`  | `simd_json`  | `sonic_rs` |
| :------------------------ | :----------: | :----------: | :----------: | :--------: |
| time (canada)             |   9.51 ms    |   7.24 ms    |   4.54 ms    |  2.76 ms   |
| throughput (canada)       | 304.88 MiB/s | 400.17 MiB/s | 639.13 MiB/s | 1.03 GiB/s |
| time (citm_catalog)       |   2.74 ms    |   2.21 ms    |   1.56 ms    | 896.74 µs  |
| throughput (citm_catalog) | 397.37 MiB/s | 492.73 MiB/s | 698.96 MiB/s | 1.18 GiB/s |
| time (twitter)            |   1.81 ms    |  889.30 µs   |  780.56 µs   | 420.20 µs  |
| throughput (twitter)      | 329.76 MiB/s | 669.70 MiB/s | 762.99 MiB/s | 1.38 GiB/s |

## prettify_ast

|                           |  `sonic_rs`  | `serde_json` | `simd_json`  | `jjpwrgem` |
| :------------------------ | :----------: | :----------: | :----------: | :--------: |
| time (canada)             |   6.60 ms    |   6.06 ms    |   5.75 ms    |  1.91 ms   |
| throughput (canada)       | 439.49 MiB/s | 478.29 MiB/s | 504.37 MiB/s | 1.48 GiB/s |
| time (citm_catalog)       |   1.42 ms    |   1.41 ms    |   1.09 ms    | 880.04 µs  |
| throughput (citm_catalog) | 763.80 MiB/s | 768.41 MiB/s | 996.85 MiB/s | 1.21 GiB/s |
| time (twitter)            |  497.83 µs   |  656.85 µs   |  514.74 µs   | 341.54 µs  |
| throughput (twitter)      |  1.17 GiB/s  | 906.69 MiB/s |  1.13 GiB/s  | 1.70 GiB/s |

## uglify_ast

|                           | `simd_json`  | `serde_json` | `sonic_rs` | `jjpwrgem` |
| :------------------------ | :----------: | :----------: | :--------: | :--------: |
| time (canada)             |   3.51 ms    |   2.73 ms    |  2.66 ms   | 811.73 µs  |
| throughput (canada)       | 826.86 MiB/s |  1.04 GiB/s  | 1.06 GiB/s | 3.49 GiB/s |
| time (citm_catalog)       |  495.32 µs   |  517.18 µs   | 546.28 µs  | 324.89 µs  |
| throughput (citm_catalog) |  2.14 GiB/s  |  2.05 GiB/s  | 1.94 GiB/s | 3.27 GiB/s |
| time (twitter)            |  405.71 µs   |  367.35 µs   | 262.05 µs  | 144.45 µs  |
| throughput (twitter)      |  1.43 GiB/s  |  1.58 GiB/s  | 2.22 GiB/s | 4.03 GiB/s |

## uglify_tokens

|                           |  `jjpwrgem`  |
| :------------------------ | :----------: |
| time (canada)             |   5.25 ms    |
| throughput (canada)       | 552.52 MiB/s |
| time (citm_catalog)       |   1.88 ms    |
| throughput (citm_catalog) | 577.06 MiB/s |
| time (twitter)            |  842.17 µs   |
| throughput (twitter)      | 707.17 MiB/s |

## serialize

|                           |  `jjpwrgem`  | `simd_json`  | `sonic_rs` | `serde_json` |
| :------------------------ | :----------: | :----------: | :--------: | :----------: |
| time (canada)             |   8.02 ms    |   3.21 ms    |  2.32 ms   |   2.30 ms    |
| throughput (canada)       | 361.43 MiB/s | 903.48 MiB/s | 1.22 GiB/s |  1.23 GiB/s  |
| time (citm_catalog)       |  755.88 µs   |  409.35 µs   | 486.50 µs  |  419.11 µs   |
| throughput (citm_catalog) |  1.40 GiB/s  |  2.59 GiB/s  | 2.18 GiB/s |  2.53 GiB/s  |
| time (twitter)            |  641.04 µs   |  309.92 µs   | 233.40 µs  |  321.56 µs   |
| throughput (twitter)      | 929.05 MiB/s |  1.88 GiB/s  | 2.49 GiB/s |  1.81 GiB/s  |
