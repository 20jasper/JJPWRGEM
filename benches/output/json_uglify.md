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
