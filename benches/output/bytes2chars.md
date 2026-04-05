## ascii

|                     | `bytes2chars` | `utf8_decode` |   `bstr`   |
| :------------------ | :-----------: | :-----------: | :--------: |
| time (64 KiB)       |   146.69 µs   |   45.05 µs    |  21.99 µs  |
| throughput (64 KiB) | 426.08 MiB/s  |  1.35 GiB/s   | 2.78 GiB/s |

## non_ascii

|                     | `bytes2chars` |   `bstr`   | `utf8_decode` |
| :------------------ | :-----------: | :--------: | :-----------: |
| time (64 KiB)       |   150.80 µs   |  51.73 µs  |   41.85 µs    |
| throughput (64 KiB) | 414.46 MiB/s  | 1.18 GiB/s |  1.46 GiB/s   |
