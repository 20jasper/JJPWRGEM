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
