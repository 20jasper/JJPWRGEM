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
| `jjp`          |          29.6 ± 1.8 |          |                |
| `json-pp-rust` |          27.0 ± 1.5 |    -8.6% |                |
| `jsonice`      |          24.7 ± 0.3 |   -16.5% |                |
| `prettier`     |       1091.1 ± 23.9 | +3586.0% | (36.9x slower) |
| `jq`           |         102.2 ± 1.3 |  +245.4% | (3.5x slower)  |
| `gojq`         |          38.8 ± 1.4 |   +31.0% | (1.3x slower)  |
| `jaq`          |          41.7 ± 2.5 |   +41.0% | (1.4x slower)  |
| `sjq`          |         419.9 ± 2.5 | +1318.5% | (14.2x slower) |
| `dprint`       |         413.1 ± 7.3 | +1295.7% | (14.0x slower) |
| `python`       |         274.8 ± 2.6 |  +828.4% | (9.3x slower)  |
| `node`         |          67.7 ± 1.8 |  +128.7% | (2.3x slower)  |
| `bun`          |          24.1 ± 0.7 |   -18.5% |                |
| `jsonxf`       |          12.4 ± 0.3 |   -58.1% | (2.4x faster)  |
| `jsonformat`   |          15.9 ± 0.9 |   -46.2% | (1.9x faster)  |
| `oxfmt`        |          95.3 ± 5.5 |  +221.8% | (3.2x slower)  |

### ugly canada

![candlestick benchmark for ugly printing canada.json](/benches/docker/output/ugly-canada.png)

| Command       | Mean Wall Time [ms] |   Change | Factor         |
| :------------ | ------------------: | -------: | :------------- |
| `jjp`         |          16.2 ± 1.0 |          |                |
| `json-minify` |          61.8 ± 3.1 |  +282.2% | (3.8x slower)  |
| `jsonxf`      |          13.6 ± 0.9 |   -16.1% |                |
| `jq`          |          70.1 ± 2.1 |  +333.6% | (4.3x slower)  |
| `jaq`         |          32.1 ± 1.2 |   +98.7% | (2.0x slower)  |
| `gojq`        |          33.6 ± 0.7 |  +107.9% | (2.1x slower)  |
| `sjq`         |         335.1 ± 5.2 | +1971.6% | (20.7x slower) |
| `python`      |         251.3 ± 6.4 | +1453.6% | (15.5x slower) |
| `minify`      |          17.5 ± 0.6 |    +8.3% |                |
| `node`        |          55.9 ± 1.8 |  +245.3% | (3.5x slower)  |
| `bun`         |          17.1 ± 0.6 |    +5.4% |                |

## citm catalog

[citm_catalog.json](/benches/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

### pretty citm catalog

![candlestick benchmark for pretty printing citm-catalog.json](/benches/docker/output/pretty-citm_catalog.png)

| Command        | Mean Wall Time [ms] |   Change | Factor         |
| :------------- | ------------------: | -------: | :------------- |
| `jjp`          |           8.6 ± 0.2 |          |                |
| `json-pp-rust` |          11.3 ± 1.2 |   +31.4% | (1.3x slower)  |
| `jsonice`      |           6.4 ± 0.1 |   -25.5% |                |
| `prettier`     |        502.0 ± 13.9 | +5741.6% | (58.4x slower) |
| `jq`           |          31.7 ± 0.5 |  +269.1% | (3.7x slower)  |
| `gojq`         |          19.4 ± 0.6 |  +125.6% | (2.3x slower)  |
| `jaq`          |          15.0 ± 0.6 |   +74.1% | (1.7x slower)  |
| `sjq`          |         106.8 ± 1.4 | +1142.5% | (12.4x slower) |
| `dprint`       |         211.4 ± 3.3 | +2360.4% | (24.6x slower) |
| `python`       |        171.3 ± 16.5 | +1893.2% | (19.9x slower) |
| `node`         |          41.5 ± 2.7 |  +382.9% | (4.8x slower)  |
| `bun`          |          13.7 ± 0.9 |   +60.0% | (1.6x slower)  |
| `jsonxf`       |           3.4 ± 0.2 |   -60.6% | (2.5x faster)  |
| `jsonformat`   |           5.2 ± 0.2 |   -39.4% | (1.7x faster)  |
| `oxfmt`        |          49.3 ± 2.4 |  +473.7% | (5.7x slower)  |

### ugly citm catalog

![candlestick benchmark for ugly printing citm-catalog.json](/benches/docker/output/ugly-citm_catalog.png)

| Command       | Mean Wall Time [ms] |   Change | Factor         |
| :------------ | ------------------: | -------: | :------------- |
| `jjp`         |           5.0 ± 0.2 |          |                |
| `json-minify` |          40.3 ± 1.9 |  +703.0% | (8.0x slower)  |
| `jsonxf`      |           3.6 ± 0.1 |   -29.3% |                |
| `jq`          |          27.0 ± 1.2 |  +436.6% | (5.4x slower)  |
| `jaq`         |          12.8 ± 0.3 |  +154.3% | (2.5x slower)  |
| `gojq`        |          18.9 ± 0.5 |  +276.2% | (3.8x slower)  |
| `sjq`         |          88.3 ± 1.1 | +1658.0% | (17.6x slower) |
| `python`      |         150.2 ± 1.2 | +2889.7% | (29.9x slower) |
| `minify`      |           7.3 ± 0.3 |   +45.8% | (1.5x slower)  |
| `node`        |          34.7 ± 1.6 |  +591.5% | (6.9x slower)  |
| `bun`         |           9.9 ± 0.3 |   +97.5% | (2.0x slower)  |

## twitter

[twitter.json](/benches/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

### pretty twitter

![candlestick benchmark for pretty printing twitter.json](/benches/docker/output/pretty-twitter.png)

| Command        | Mean Wall Time [ms] |   Change | Factor         |
| :------------- | ------------------: | -------: | :------------- |
| `jjp`          |           4.2 ± 0.1 |          |                |
| `json-pp-rust` |           5.2 ± 0.1 |   +22.6% |                |
| `jsonice`      |           3.9 ± 0.1 |    -7.3% |                |
| `prettier`     |         275.4 ± 5.8 | +6439.0% | (65.4x slower) |
| `jq`           |          15.5 ± 0.7 |  +267.4% | (3.7x slower)  |
| `gojq`         |          12.1 ± 0.8 |  +186.2% | (2.9x slower)  |
| `jaq`          |           8.4 ± 0.6 |  +100.0% | (2.0x slower)  |
| `sjq`          |          40.5 ± 0.6 |  +862.1% | (9.6x slower)  |
| `dprint`       |          42.8 ± 1.0 |  +916.7% | (10.2x slower) |
| `python`       |         126.4 ± 1.4 | +2901.9% | (30.0x slower) |
| `node`         |          32.4 ± 1.4 |  +669.6% | (7.7x slower)  |
| `bun`          |           8.1 ± 0.3 |   +93.1% | (1.9x slower)  |
| `jsonxf`       |           1.7 ± 0.1 |   -58.6% | (2.4x faster)  |
| `jsonformat`   |           3.2 ± 0.1 |   -24.3% |                |
| `oxfmt`        |          44.1 ± 2.1 |  +946.0% | (10.5x slower) |

### ugly twitter

![candlestick benchmark for ugly printing twitter.json](/benches/docker/output/ugly-twitter.png)

| Command       | Mean Wall Time [ms] |   Change | Factor         |
| :------------ | ------------------: | -------: | :------------- |
| `jjp`         |           3.0 ± 0.1 |          |                |
| `json-minify` |          33.5 ± 1.8 | +1032.1% | (11.3x slower) |
| `jsonxf`      |           1.7 ± 0.1 |   -42.2% | (1.7x faster)  |
| `jq`          |          13.6 ± 0.4 |  +358.6% | (4.6x slower)  |
| `jaq`         |           7.5 ± 0.3 |  +152.2% | (2.5x slower)  |
| `gojq`        |          11.4 ± 0.4 |  +286.2% | (3.9x slower)  |
| `sjq`         |          34.8 ± 1.2 | +1076.4% | (11.8x slower) |
| `python`      |         124.1 ± 2.2 | +4095.8% | (42.0x slower) |
| `minify`      |           5.4 ± 0.4 |   +81.5% | (1.8x slower)  |
| `node`        |          30.2 ± 1.0 |  +921.2% | (10.2x slower) |
| `bun`         |           8.0 ± 0.7 |  +171.6% | (2.7x slower)  |
