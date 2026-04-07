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
