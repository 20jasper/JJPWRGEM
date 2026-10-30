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
