<!-- GENERATED FILE - update the templates in the xtask -->

# bytes2chars benchmarks

Throughput benchmarks comparing `bytes2chars` against alternative UTF-8 decoders.
Run locally with `mise run bench bytes2chars`

## ascii

|                     | `utf8_decode` | `bytes2chars` |   `bstr`   |
| :------------------ | :-----------: | :-----------: | :--------: |
| time (64 KiB)       |   162.16 µs   |   136.51 µs   |  22.21 µs  |
| throughput (64 KiB) | 385.42 MiB/s  | 457.84 MiB/s  | 2.75 GiB/s |

## non_ascii

|                     | `bytes2chars` | `utf8_decode` |   `bstr`   |
| :------------------ | :-----------: | :-----------: | :--------: |
| time (64 KiB)       |   159.18 µs   |   113.22 µs   |  44.96 µs  |
| throughput (64 KiB) | 392.65 MiB/s  | 552.03 MiB/s  | 1.36 GiB/s |
