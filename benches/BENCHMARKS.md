<!-- GENERATED FILE - update the templates in the xtask -->

# jjpwrgem JSON benchmarks

Throughput benchmarks for `jjpwrgem`'s parse and formatting operations.
Run locally with `just bench-json` or individual `just bench json_deser`, `just bench json_prettify`, and `just bench json_uglify`.
Throughput is normalized by input bytes for all tables so implementations stay comparable even when output formatting differs.

## deser

|                           |  `jjpwrgem`  | `serde_json` | `simd_json`  |  `sonic_rs`  |
| :------------------------ | :----------: | :----------: | :----------: | :----------: |
| time (canada)             |   20.18 ms   |   12.64 ms   |   5.17 ms    |   2.89 ms    |
| throughput (canada)       | 106.38 MiB/s | 169.85 MiB/s | 415.01 MiB/s | 743.51 MiB/s |
| time (citm_catalog)       |   4.81 ms    |   3.22 ms    |   1.81 ms    |   1.06 ms    |
| throughput (citm_catalog) | 342.62 MiB/s | 512.34 MiB/s | 909.85 MiB/s |  1.52 GiB/s  |
| time (twitter)            |   2.24 ms    |   1.82 ms    |  863.12 µs   |  416.82 µs   |
| throughput (twitter)      | 269.41 MiB/s | 331.09 MiB/s | 697.77 MiB/s |  1.41 GiB/s  |

## prettify_ast

|                           |  `sonic_rs`  | `serde_json` | `simd_json`  | `jjpwrgem` |
| :------------------------ | :----------: | :----------: | :----------: | :--------: |
| time (canada)             |   6.94 ms    |   6.03 ms    |   5.36 ms    |  1.67 ms   |
| throughput (canada)       | 309.36 MiB/s | 355.90 MiB/s | 400.35 MiB/s | 1.26 GiB/s |
| time (citm_catalog)       |   1.40 ms    |   1.19 ms    |  972.12 µs   | 933.11 µs  |
| throughput (citm_catalog) |  1.15 GiB/s  |  1.35 GiB/s  |  1.65 GiB/s  | 1.72 GiB/s |
| time (twitter)            |  472.09 µs   |  539.06 µs   |  473.95 µs   | 298.64 µs  |
| throughput (twitter)      |  1.25 GiB/s  |  1.09 GiB/s  |  1.24 GiB/s  | 1.97 GiB/s |

## uglify_ast

|                           |  `sonic_rs`  | `simd_json`  | `serde_json` | `jjpwrgem` |
| :------------------------ | :----------: | :----------: | :----------: | :--------: |
| time (canada)             |   3.58 ms    |   3.26 ms    |   2.87 ms    | 734.19 µs  |
| throughput (canada)       | 598.85 MiB/s | 659.30 MiB/s | 747.90 MiB/s | 2.86 GiB/s |
| time (citm_catalog)       |  583.26 µs   |  439.36 µs   |  520.63 µs   | 262.31 µs  |
| throughput (citm_catalog) |  2.76 GiB/s  |  3.66 GiB/s  |  3.09 GiB/s  | 6.13 GiB/s |
| time (twitter)            |  274.44 µs   |  352.39 µs   |  369.98 µs   | 124.35 µs  |
| throughput (twitter)      |  2.14 GiB/s  |  1.67 GiB/s  |  1.59 GiB/s  | 4.73 GiB/s |

## uglify_tokens

|                           |  `jjpwrgem`  |
| :------------------------ | :----------: |
| time (canada)             |   11.07 ms   |
| throughput (canada)       | 193.94 MiB/s |
| time (citm_catalog)       |   4.14 ms    |
| throughput (citm_catalog) | 397.89 MiB/s |
| time (twitter)            |   1.96 ms    |
| throughput (twitter)      | 307.12 MiB/s |
