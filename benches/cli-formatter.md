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
| `jjp`          |          21.5 ± 0.4 |          |                |
| `json-pp-rust` |          24.9 ± 0.9 |   +15.8% |                |
| `jsonice`      |          22.5 ± 1.0 |    +4.8% |                |
| `prettier`     |       1000.4 ± 14.1 | +4554.0% | (46.5x slower) |
| `jq`           |          95.4 ± 3.2 |  +343.6% | (4.4x slower)  |
| `gojq`         |          35.6 ± 0.7 |   +65.5% | (1.7x slower)  |
| `jaq`          |          39.2 ± 1.8 |   +82.3% | (1.8x slower)  |
| `sjq`          |         415.5 ± 6.4 | +1832.8% | (19.3x slower) |
| `dprint`       |         416.7 ± 4.0 | +1838.5% | (19.4x slower) |
| `python`       |         271.9 ± 5.1 | +1165.0% | (12.6x slower) |
| `node`         |          69.4 ± 1.9 |  +223.1% | (3.2x slower)  |
| `bun`          |          24.0 ± 0.7 |   +11.9% |                |
| `jsonxf`       |          12.2 ± 0.2 |   -43.1% | (1.8x faster)  |
| `jsonformat`   |          15.7 ± 0.2 |   -26.8% |                |
| `oxfmt`        |          93.3 ± 3.0 |  +333.9% | (4.3x slower)  |

### ugly canada

![candlestick benchmark for ugly printing canada.json](/benches/docker/output/ugly-canada.png)

| Command       | Mean Wall Time [ms] |   Change | Factor         |
| :------------ | ------------------: | -------: | :------------- |
| `jjp`         |           8.9 ± 0.3 |          |                |
| `json-minify` |          60.2 ± 1.5 |  +578.5% | (6.8x slower)  |
| `jsonxf`      |          12.1 ± 0.7 |   +36.7% | (1.4x slower)  |
| `jq`          |          72.6 ± 6.6 |  +718.9% | (8.2x slower)  |
| `jaq`         |          32.2 ± 2.4 |  +262.7% | (3.6x slower)  |
| `gojq`        |          33.6 ± 1.4 |  +278.8% | (3.8x slower)  |
| `sjq`         |         333.9 ± 6.1 | +3663.9% | (37.6x slower) |
| `python`      |         245.2 ± 2.9 | +2663.9% | (27.6x slower) |
| `minify`      |          17.6 ± 0.7 |   +98.4% | (2.0x slower)  |
| `node`        |          56.2 ± 1.9 |  +533.5% | (6.3x slower)  |
| `bun`         |          16.7 ± 0.5 |   +88.2% | (1.9x slower)  |

## citm catalog

[citm_catalog.json](/benches/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

### pretty citm catalog

![candlestick benchmark for pretty printing citm-catalog.json](/benches/docker/output/pretty-citm_catalog.png)

| Command        | Mean Wall Time [ms] |   Change | Factor         |
| :------------- | ------------------: | -------: | :------------- |
| `jjp`          |           7.1 ± 0.4 |          |                |
| `json-pp-rust` |          10.9 ± 0.6 |   +52.6% | (1.5x slower)  |
| `jsonice`      |           6.2 ± 0.1 |   -12.6% |                |
| `prettier`     |         510.2 ± 7.7 | +7040.1% | (71.4x slower) |
| `jq`           |          30.9 ± 1.0 |  +332.5% | (4.3x slower)  |
| `gojq`         |          19.4 ± 0.6 |  +171.5% | (2.7x slower)  |
| `jaq`          |          14.6 ± 0.8 |  +105.0% | (2.1x slower)  |
| `sjq`          |         106.3 ± 1.3 | +1387.7% | (14.9x slower) |
| `dprint`       |         212.2 ± 2.9 | +2869.9% | (29.7x slower) |
| `python`       |         151.9 ± 2.4 | +2025.4% | (21.3x slower) |
| `node`         |          41.9 ± 3.1 |  +486.1% | (5.9x slower)  |
| `bun`          |          13.6 ± 0.7 |   +90.1% | (1.9x slower)  |
| `jsonxf`       |           3.3 ± 0.2 |   -54.5% | (2.2x faster)  |
| `jsonformat`   |           5.0 ± 0.2 |   -29.5% |                |
| `oxfmt`        |          48.9 ± 1.4 |  +584.5% | (6.8x slower)  |

### ugly citm catalog

![candlestick benchmark for ugly printing citm-catalog.json](/benches/docker/output/ugly-citm_catalog.png)

| Command       | Mean Wall Time [ms] |   Change | Factor         |
| :------------ | ------------------: | -------: | :------------- |
| `jjp`         |           3.6 ± 0.1 |          |                |
| `json-minify` |          36.8 ± 1.9 |  +920.3% | (10.2x slower) |
| `jsonxf`      |           3.1 ± 0.1 |   -13.6% |                |
| `jq`          |          25.9 ± 0.5 |  +619.2% | (7.2x slower)  |
| `jaq`         |          12.5 ± 0.5 |  +247.2% | (3.5x slower)  |
| `gojq`        |          18.7 ± 0.4 |  +419.3% | (5.2x slower)  |
| `sjq`         |          88.4 ± 1.9 | +2349.3% | (24.5x slower) |
| `python`      |         147.7 ± 4.1 | +3993.0% | (40.9x slower) |
| `minify`      |           7.3 ± 0.4 |  +102.3% | (2.0x slower)  |
| `node`        |          34.7 ± 2.0 |  +862.8% | (9.6x slower)  |
| `bun`         |           9.8 ± 0.3 |  +171.4% | (2.7x slower)  |

## twitter

[twitter.json](/benches/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

### pretty twitter

![candlestick benchmark for pretty printing twitter.json](/benches/docker/output/pretty-twitter.png)

| Command        | Mean Wall Time [ms] |   Change | Factor         |
| :------------- | ------------------: | -------: | :------------- |
| `jjp`          |           3.6 ± 0.1 |          |                |
| `json-pp-rust` |           5.0 ± 0.2 |   +39.3% | (1.4x slower)  |
| `jsonice`      |           3.7 ± 0.2 |    +4.8% |                |
| `prettier`     |        280.6 ± 11.0 | +7796.1% | (79.0x slower) |
| `jq`           |          14.7 ± 0.2 |  +314.1% | (4.1x slower)  |
| `gojq`         |          11.9 ± 0.9 |  +235.9% | (3.4x slower)  |
| `jaq`          |           8.1 ± 0.5 |  +126.7% | (2.3x slower)  |
| `sjq`          |          40.7 ± 0.9 | +1044.5% | (11.4x slower) |
| `dprint`       |          42.8 ± 1.2 | +1103.1% | (12.0x slower) |
| `python`       |         123.6 ± 1.5 | +3376.9% | (34.8x slower) |
| `node`         |          32.6 ± 1.6 |  +817.3% | (9.2x slower)  |
| `bun`          |           7.9 ± 0.2 |  +123.5% | (2.2x slower)  |
| `jsonxf`       |           1.6 ± 0.1 |   -54.0% | (2.2x faster)  |
| `jsonformat`   |           3.0 ± 0.1 |   -14.3% |                |
| `oxfmt`        |          43.5 ± 1.7 | +1124.8% | (12.2x slower) |

### ugly twitter

![candlestick benchmark for ugly printing twitter.json](/benches/docker/output/ugly-twitter.png)

| Command       | Mean Wall Time [ms] |   Change | Factor         |
| :------------ | ------------------: | -------: | :------------- |
| `jjp`         |           2.4 ± 0.1 |          |                |
| `json-minify` |          33.5 ± 2.0 | +1321.9% | (14.2x slower) |
| `jsonxf`      |           1.6 ± 0.1 |   -31.8% | (1.5x faster)  |
| `jq`          |          13.4 ± 0.4 |  +468.4% | (5.7x slower)  |
| `jaq`         |           7.3 ± 0.3 |  +211.7% | (3.1x slower)  |
| `gojq`        |          11.2 ± 0.4 |  +377.3% | (4.8x slower)  |
| `sjq`         |          34.7 ± 0.9 | +1374.8% | (14.7x slower) |
| `python`      |         123.4 ± 1.8 | +5142.4% | (52.4x slower) |
| `minify`      |           5.2 ± 0.4 |  +120.2% | (2.2x slower)  |
| `node`        |          30.5 ± 1.4 | +1196.8% | (13.0x slower) |
| `bun`         |           7.1 ± 0.2 |  +200.0% | (3.0x slower)  |
