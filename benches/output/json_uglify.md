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
