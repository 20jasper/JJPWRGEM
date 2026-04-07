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
