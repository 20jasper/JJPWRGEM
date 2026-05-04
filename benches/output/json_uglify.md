## uglify_ast

|                           |  `sonic_rs`  | `simd_json`  | `serde_json` | `jjpwrgem` |
| :------------------------ | :----------: | :----------: | :----------: | :--------: |
| time (canada)             |   3.81 ms    |   3.30 ms    |   2.94 ms    | 744.76 µs  |
| throughput (canada)       | 563.22 MiB/s | 650.28 MiB/s | 729.10 MiB/s | 2.81 GiB/s |
| time (citm_catalog)       |  633.39 µs   |  488.54 µs   |  507.50 µs   | 295.85 µs  |
| throughput (citm_catalog) |  2.54 GiB/s  |  3.29 GiB/s  |  3.17 GiB/s  | 5.44 GiB/s |
| time (twitter)            |  289.03 µs   |  372.55 µs   |  365.44 µs   | 144.80 µs  |
| throughput (twitter)      |  2.03 GiB/s  |  1.58 GiB/s  |  1.61 GiB/s  | 4.06 GiB/s |

## uglify_tokens

|                           |  `jjpwrgem`  |
| :------------------------ | :----------: |
| time (canada)             |   8.91 ms    |
| throughput (canada)       | 241.00 MiB/s |
| time (citm_catalog)       |   2.64 ms    |
| throughput (citm_catalog) | 622.78 MiB/s |
| time (twitter)            |   1.44 ms    |
| throughput (twitter)      | 419.28 MiB/s |
