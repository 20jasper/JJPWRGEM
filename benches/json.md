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

|                           | `serde_json` |  `jjpwrgem`  | `simd_json`  |  `sonic_rs`   |
| :------------------------ | :----------: | :----------: | :----------: | :-----------: |
| time (canada)             |   9.44 ms    |   7.48 ms    |   4.39 ms    |    2.85 ms    |
| throughput (canada)       | 307.08 MiB/s | 387.59 MiB/s | 659.92 MiB/s | 1016.92 MiB/s |
| time (citm_catalog)       |   2.73 ms    |   2.24 ms    |   1.53 ms    |   872.68 µs   |
| throughput (citm_catalog) | 398.10 MiB/s | 486.20 MiB/s | 709.44 MiB/s |  1.22 GiB/s   |
| time (twitter)            |   1.78 ms    |  871.99 µs   |  735.07 µs   |   420.20 µs   |
| throughput (twitter)      | 334.01 MiB/s | 682.99 MiB/s | 810.21 MiB/s |  1.38 GiB/s   |

## prettify_ast

|                           |  `sonic_rs`  | `serde_json`  | `simd_json`  | `jjpwrgem` |
| :------------------------ | :----------: | :-----------: | :----------: | :--------: |
| time (canada)             |   6.50 ms    |    6.33 ms    |   5.69 ms    |  1.94 ms   |
| throughput (canada)       | 446.05 MiB/s | 457.62 MiB/s  | 509.88 MiB/s | 1.46 GiB/s |
| time (citm_catalog)       |   1.43 ms    |    1.35 ms    |   1.11 ms    | 880.57 µs  |
| throughput (citm_catalog) | 761.17 MiB/s | 803.32 MiB/s  | 976.63 MiB/s | 1.21 GiB/s |
| time (twitter)            |  491.61 µs   |   593.40 µs   |  522.03 µs   | 348.73 µs  |
| throughput (twitter)      |  1.18 GiB/s  | 1003.64 MiB/s |  1.11 GiB/s  | 1.67 GiB/s |

## uglify_ast

|                           | `simd_json`  | `serde_json` | `sonic_rs` | `jjpwrgem` |
| :------------------------ | :----------: | :----------: | :--------: | :--------: |
| time (canada)             |   3.53 ms    |   2.74 ms    |  2.67 ms   | 861.71 µs  |
| throughput (canada)       | 821.25 MiB/s |  1.03 GiB/s  | 1.06 GiB/s | 3.29 GiB/s |
| time (citm_catalog)       |  479.56 µs   |  541.78 µs   | 558.52 µs  | 320.77 µs  |
| throughput (citm_catalog) |  2.21 GiB/s  |  1.96 GiB/s  | 1.90 GiB/s | 3.31 GiB/s |
| time (twitter)            |  395.87 µs   |  434.36 µs   | 274.82 µs  | 147.64 µs  |
| throughput (twitter)      |  1.47 GiB/s  |  1.34 GiB/s  | 2.12 GiB/s | 3.94 GiB/s |

## uglify_tokens

|                           |  `jjpwrgem`  |
| :------------------------ | :----------: |
| time (canada)             |   5.25 ms    |
| throughput (canada)       | 552.62 MiB/s |
| time (citm_catalog)       |   1.91 ms    |
| throughput (citm_catalog) | 567.82 MiB/s |
| time (twitter)            |  854.77 µs   |
| throughput (twitter)      | 696.75 MiB/s |

## serialize_struct

|                           | `simd_json`  | `jjpwrgem` | `sonic_rs` | `serde_json` |
| :------------------------ | :----------: | :--------: | :--------: | :----------: |
| time (canada)             |   3.17 ms    |  2.28 ms   |  2.26 ms   |   2.24 ms    |
| throughput (canada)       | 915.81 MiB/s | 1.24 GiB/s | 1.25 GiB/s |  1.26 GiB/s  |
| time (citm_catalog)       |  424.75 µs   | 457.40 µs  | 499.32 µs  |  421.05 µs   |
| throughput (citm_catalog) |  2.50 GiB/s  | 2.32 GiB/s | 2.13 GiB/s |  2.52 GiB/s  |
| time (twitter)            |  333.65 µs   | 324.14 µs  | 234.71 µs  |  318.01 µs   |
| throughput (twitter)      |  1.74 GiB/s  | 1.79 GiB/s | 2.48 GiB/s |  1.83 GiB/s  |
