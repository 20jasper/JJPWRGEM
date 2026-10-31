<!-- GENERATED FILE - update the templates in the xtask -->

# jjpwrgem CLI formatter and minifier benchmarks

Wall-clock timing against jq, prettier, dprint, and others via hyperfine. Run locally with `mise run bench-docker`
These benchmarks are run with `AMD Ryzen 5 5600X 6-Core Processor (3.70 GHz)`

The following JSON fixtures are used across benchmarks:

- [canada.json](/benches/data/canada.json) — 2.2MB, lots of lightly nested arrays, no strings

- [citm_catalog.json](/benches/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

- [twitter.json](/benches/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

## canada

[canada.json](/benches/data/canada.json) — 2.2MB, lots of lightly nested arrays, no strings

### pretty canada

![candlestick benchmark for pretty printing canada.json](/benches/docker/output/pretty-canada.png)

| Command        | Mean Wall Time [ms] |   Change | Factor         |
| :------------- | ------------------: | -------: | :------------- |
| `jjp`          |          19.9 ± 0.4 |          |                |
| `json-pp-rust` |          26.3 ± 1.2 |   +32.2% | (1.3x slower)  |
| `jsonice`      |          24.4 ± 1.0 |   +22.4% |                |
| `prettier`     |       1106.4 ± 27.6 | +5453.6% | (55.5x slower) |
| `jq`           |        109.7 ± 10.4 |  +450.6% | (5.5x slower)  |
| `gojq`         |          40.5 ± 3.4 |  +103.5% | (2.0x slower)  |
| `jaq`          |          44.0 ± 3.8 |  +120.7% | (2.2x slower)  |
| `sjq`          |         455.6 ± 7.7 | +2186.9% | (22.9x slower) |
| `dprint`       |        449.1 ± 20.1 | +2154.1% | (22.5x slower) |
| `python`       |         299.3 ± 2.7 | +1402.6% | (15.0x slower) |
| `node`         |          76.8 ± 1.7 |  +285.5% | (3.9x slower)  |
| `bun`          |          26.1 ± 0.7 |   +30.9% | (1.3x slower)  |
| `jsonxf`       |          13.5 ± 0.3 |   -32.3% | (1.5x faster)  |
| `jsonformat`   |          17.4 ± 0.7 |   -12.9% |                |
| `oxfmt`        |         105.5 ± 3.0 |  +429.7% | (5.3x slower)  |

### ugly canada

![candlestick benchmark for ugly printing canada.json](/benches/docker/output/ugly-canada.png)

| Command       | Mean Wall Time [ms] |   Change | Factor         |
| :------------ | ------------------: | -------: | :------------- |
| `jjp`         |           9.6 ± 0.4 |          |                |
| `json-minify` |          67.0 ± 1.6 |  +598.2% | (7.0x slower)  |
| `jsonxf`      |          13.2 ± 0.4 |   +37.1% | (1.4x slower)  |
| `jq`          |          77.5 ± 0.5 |  +707.4% | (8.1x slower)  |
| `jaq`         |          35.0 ± 1.0 |  +264.0% | (3.6x slower)  |
| `gojq`        |          37.0 ± 0.7 |  +285.7% | (3.9x slower)  |
| `sjq`         |         370.4 ± 6.4 | +3758.0% | (38.6x slower) |
| `python`      |         277.8 ± 2.2 | +2793.4% | (28.9x slower) |
| `minify`      |          19.2 ± 0.6 |  +100.4% | (2.0x slower)  |
| `node`        |          65.4 ± 2.6 |  +580.9% | (6.8x slower)  |
| `bun`         |          19.0 ± 1.0 |   +97.7% | (2.0x slower)  |

## citm catalog

[citm_catalog.json](/benches/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

### pretty citm catalog

![candlestick benchmark for pretty printing citm-catalog.json](/benches/docker/output/pretty-citm_catalog.png)

| Command        | Mean Wall Time [ms] |   Change | Factor         |
| :------------- | ------------------: | -------: | :------------- |
| `jjp`          |           7.1 ± 0.7 |          |                |
| `json-pp-rust` |          12.0 ± 0.4 |   +68.6% | (1.7x slower)  |
| `jsonice`      |           6.9 ± 0.1 |    -2.7% |                |
| `prettier`     |        560.8 ± 12.7 | +7810.1% | (79.1x slower) |
| `jq`           |          34.2 ± 1.5 |  +381.9% | (4.8x slower)  |
| `gojq`         |          21.3 ± 0.6 |  +201.1% | (3.0x slower)  |
| `jaq`          |          16.0 ± 0.7 |  +125.5% | (2.3x slower)  |
| `sjq`          |         114.4 ± 1.5 | +1513.0% | (16.1x slower) |
| `dprint`       |         233.7 ± 4.3 | +3196.3% | (33.0x slower) |
| `python`       |         168.3 ± 2.3 | +2273.4% | (23.7x slower) |
| `node`         |          44.7 ± 2.3 |  +530.2% | (6.3x slower)  |
| `bun`          |          14.8 ± 0.4 |  +109.2% | (2.1x slower)  |
| `jsonxf`       |           3.6 ± 0.1 |   -49.8% | (2.0x faster)  |
| `jsonformat`   |           5.6 ± 0.2 |   -21.5% |                |
| `oxfmt`        |          55.3 ± 3.1 |  +679.7% | (7.8x slower)  |

### ugly citm catalog

![candlestick benchmark for ugly printing citm-catalog.json](/benches/docker/output/ugly-citm_catalog.png)

| Command       | Mean Wall Time [ms] |   Change | Factor         |
| :------------ | ------------------: | -------: | :------------- |
| `jjp`         |           4.0 ± 0.2 |          |                |
| `json-minify` |          41.0 ± 2.1 |  +933.8% | (10.3x slower) |
| `jsonxf`      |           3.4 ± 0.1 |   -13.2% |                |
| `jq`          |          28.6 ± 0.6 |  +622.7% | (7.2x slower)  |
| `jaq`         |          14.0 ± 0.4 |  +251.9% | (3.5x slower)  |
| `gojq`        |          21.6 ± 1.0 |  +446.0% | (5.5x slower)  |
| `sjq`         |          98.5 ± 3.8 | +2386.0% | (24.9x slower) |
| `python`      |         163.1 ± 3.9 | +4014.7% | (41.1x slower) |
| `minify`      |           8.1 ± 0.4 |  +103.3% | (2.0x slower)  |
| `node`        |          38.5 ± 2.1 |  +870.4% | (9.7x slower)  |
| `bun`         |          11.1 ± 0.3 |  +179.4% | (2.8x slower)  |

## twitter

[twitter.json](/benches/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

### pretty twitter

![candlestick benchmark for pretty printing twitter.json](/benches/docker/output/pretty-twitter.png)

| Command        | Mean Wall Time [ms] |   Change | Factor         |
| :------------- | ------------------: | -------: | :------------- |
| `jjp`          |           3.4 ± 0.1 |          |                |
| `json-pp-rust` |           5.5 ± 0.1 |   +62.7% | (1.6x slower)  |
| `jsonice`      |           4.1 ± 0.1 |   +21.9% |                |
| `prettier`     |         309.5 ± 6.5 | +9002.0% | (91.0x slower) |
| `jq`           |          16.2 ± 0.2 |  +376.8% | (4.8x slower)  |
| `gojq`         |          12.7 ± 0.8 |  +274.4% | (3.7x slower)  |
| `jaq`          |           8.8 ± 0.2 |  +157.8% | (2.6x slower)  |
| `sjq`          |          44.4 ± 0.6 | +1206.3% | (13.1x slower) |
| `dprint`       |          47.4 ± 2.3 | +1293.7% | (13.9x slower) |
| `python`       |         139.8 ± 1.9 | +4011.0% | (41.1x slower) |
| `node`         |          36.1 ± 1.7 |  +960.9% | (10.6x slower) |
| `bun`          |           8.9 ± 0.3 |  +163.2% | (2.6x slower)  |
| `jsonxf`       |           1.8 ± 0.1 |   -46.8% | (1.9x faster)  |
| `jsonformat`   |           3.4 ± 0.1 |    -1.2% |                |
| `oxfmt`        |          49.0 ± 2.5 | +1341.2% | (14.4x slower) |

### ugly twitter

![candlestick benchmark for ugly printing twitter.json](/benches/docker/output/ugly-twitter.png)

| Command       | Mean Wall Time [ms] |   Change | Factor         |
| :------------ | ------------------: | -------: | :------------- |
| `jjp`         |           2.5 ± 0.1 |          |                |
| `json-minify` |          36.9 ± 2.2 | +1394.5% | (14.9x slower) |
| `jsonxf`      |           1.8 ± 0.2 |   -25.3% |                |
| `jq`          |          15.1 ± 1.3 |  +510.5% | (6.1x slower)  |
| `jaq`         |           8.1 ± 0.4 |  +228.7% | (3.3x slower)  |
| `gojq`        |          12.4 ± 0.5 |  +403.2% | (5.0x slower)  |
| `sjq`         |          38.1 ± 0.7 | +1442.8% | (15.4x slower) |
| `python`      |         135.4 ± 1.0 | +5381.9% | (54.8x slower) |
| `minify`      |           5.9 ± 0.5 |  +137.6% | (2.4x slower)  |
| `node`        |          33.7 ± 1.1 | +1263.4% | (13.6x slower) |
| `bun`         |           8.1 ± 0.3 |  +227.8% | (3.3x slower)  |
