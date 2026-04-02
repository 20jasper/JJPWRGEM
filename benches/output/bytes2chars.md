## ascii

|                     | `bytes2chars` | `utf8_decode` |   `bstr`   |
| :------------------ | :-----------: | :-----------: | :--------: |
| time (64 KiB)       |   146.31 µs   |   51.08 µs    |  27.32 µs  |
| throughput (64 KiB) | 427.17 MiB/s  |  1.19 GiB/s   | 2.23 GiB/s |

## non_ascii

|                     | `bytes2chars` |   `bstr`   | `utf8_decode` |
| :------------------ | :-----------: | :--------: | :-----------: |
| time (64 KiB)       |   147.82 µs   |  43.60 µs  |   42.46 µs    |
| throughput (64 KiB) | 422.81 MiB/s  | 1.40 GiB/s |  1.44 GiB/s   |
