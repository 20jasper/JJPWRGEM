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
| `jjp`          |          19.7 ± 0.8 |          |                |
| `json-pp-rust` |          25.8 ± 0.6 |   +31.4% | (1.3x slower)  |
| `jsonice`      |          24.4 ± 0.8 |   +24.1% |                |
| `prettier`     |       1106.0 ± 23.7 | +5523.6% | (56.2x slower) |
| `jq`           |         105.8 ± 1.9 |  +438.0% | (5.4x slower)  |
| `gojq`         |          39.1 ± 1.3 |   +98.9% | (2.0x slower)  |
| `jaq`          |          43.9 ± 1.3 |  +123.3% | (2.2x slower)  |
| `sjq`          |         462.4 ± 5.0 | +2251.2% | (23.5x slower) |
| `dprint`       |        461.5 ± 32.0 | +2246.5% | (23.5x slower) |
| `python`       |         295.2 ± 6.4 | +1401.0% | (15.0x slower) |
| `node`         |          76.2 ± 2.1 |  +287.3% | (3.9x slower)  |
| `bun`          |          26.0 ± 0.7 |   +32.1% | (1.3x slower)  |
| `jsonxf`       |          13.5 ± 0.3 |   -31.2% | (1.5x faster)  |
| `jsonformat`   |          17.2 ± 0.5 |   -12.6% |                |
| `oxfmt`        |         104.4 ± 2.9 |  +430.8% | (5.3x slower)  |

### ugly canada

![candlestick benchmark for ugly printing canada.json](/benches/docker/output/ugly-canada.png)

| Command       | Mean Wall Time [ms] |   Change | Factor         |
| :------------ | ------------------: | -------: | :------------- |
| `jjp`         |           9.6 ± 0.3 |          |                |
| `json-minify` |          66.4 ± 1.7 |  +595.0% | (6.9x slower)  |
| `jsonxf`      |          13.1 ± 0.3 |   +37.2% | (1.4x slower)  |
| `jq`          |          75.4 ± 1.3 |  +689.4% | (7.9x slower)  |
| `jaq`         |          34.4 ± 1.1 |  +260.6% | (3.6x slower)  |
| `gojq`        |          37.2 ± 2.5 |  +289.8% | (3.9x slower)  |
| `sjq`         |         367.8 ± 6.0 | +3751.2% | (38.5x slower) |
| `python`      |         272.9 ± 5.1 | +2757.6% | (28.6x slower) |
| `minify`      |          19.6 ± 1.4 |  +105.1% | (2.1x slower)  |
| `node`        |          62.8 ± 1.9 |  +557.6% | (6.6x slower)  |
| `bun`         |          18.7 ± 0.6 |   +95.4% | (2.0x slower)  |

## citm catalog

[citm_catalog.json](/benches/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

### pretty citm catalog

![candlestick benchmark for pretty printing citm-catalog.json](/benches/docker/output/pretty-citm_catalog.png)

| Command        | Mean Wall Time [ms] |   Change | Factor         |
| :------------- | ------------------: | -------: | :------------- |
| `jjp`          |           6.7 ± 0.2 |          |                |
| `json-pp-rust` |          11.9 ± 0.5 |   +78.1% | (1.8x slower)  |
| `jsonice`      |           6.9 ± 0.2 |    +3.9% |                |
| `prettier`     |        570.2 ± 19.5 | +8467.0% | (85.7x slower) |
| `jq`           |          34.9 ± 1.5 |  +424.6% | (5.2x slower)  |
| `gojq`         |          21.1 ± 0.5 |  +217.7% | (3.2x slower)  |
| `jaq`          |          15.9 ± 0.7 |  +139.4% | (2.4x slower)  |
| `sjq`          |         114.5 ± 1.9 | +1621.0% | (17.2x slower) |
| `dprint`       |         231.7 ± 4.3 | +3380.8% | (34.8x slower) |
| `python`       |         170.7 ± 3.5 | +2464.0% | (25.6x slower) |
| `node`         |          44.5 ± 2.1 |  +568.7% | (6.7x slower)  |
| `bun`          |          14.7 ± 0.4 |  +121.3% | (2.2x slower)  |
| `jsonxf`       |           3.5 ± 0.1 |   -46.7% | (1.9x faster)  |
| `jsonformat`   |           5.6 ± 0.2 |   -16.1% |                |
| `oxfmt`        |          55.7 ± 2.6 |  +737.3% | (8.4x slower)  |

### ugly citm catalog

![candlestick benchmark for ugly printing citm-catalog.json](/benches/docker/output/ugly-citm_catalog.png)

| Command       | Mean Wall Time [ms] |   Change | Factor         |
| :------------ | ------------------: | -------: | :------------- |
| `jjp`         |           3.9 ± 0.1 |          |                |
| `json-minify` |          41.2 ± 2.1 |  +949.4% | (10.5x slower) |
| `jsonxf`      |           3.4 ± 0.1 |   -12.5% |                |
| `jq`          |          27.9 ± 0.7 |  +611.6% | (7.1x slower)  |
| `jaq`         |          13.7 ± 0.8 |  +249.5% | (3.5x slower)  |
| `gojq`        |          20.8 ± 0.6 |  +431.0% | (5.3x slower)  |
| `sjq`         |          95.4 ± 1.8 | +2332.3% | (24.3x slower) |
| `python`      |         163.4 ± 6.1 | +4065.2% | (41.7x slower) |
| `minify`      |           7.9 ± 0.3 |  +101.7% | (2.0x slower)  |
| `node`        |          38.7 ± 1.9 |  +886.3% | (9.9x slower)  |
| `bun`         |          11.8 ± 0.9 |  +200.7% | (3.0x slower)  |

## twitter

[twitter.json](/benches/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

### pretty twitter

![candlestick benchmark for pretty printing twitter.json](/benches/docker/output/pretty-twitter.png)

| Command        | Mean Wall Time [ms] |   Change | Factor         |
| :------------- | ------------------: | -------: | :------------- |
| `jjp`          |           3.4 ± 0.1 |          |                |
| `json-pp-rust` |           5.5 ± 0.3 |   +61.0% | (1.6x slower)  |
| `jsonice`      |           4.2 ± 0.3 |   +22.6% |                |
| `prettier`     |         306.7 ± 3.4 | +8809.5% | (89.1x slower) |
| `jq`           |          16.2 ± 0.2 |  +370.6% | (4.7x slower)  |
| `gojq`         |          12.6 ± 0.5 |  +264.7% | (3.6x slower)  |
| `jaq`          |           8.8 ± 0.3 |  +155.1% | (2.6x slower)  |
| `sjq`          |          44.2 ± 0.7 | +1185.1% | (12.9x slower) |
| `dprint`       |          46.8 ± 1.0 | +1258.1% | (13.6x slower) |
| `python`       |         139.2 ± 1.4 | +3943.7% | (40.4x slower) |
| `node`         |          35.6 ± 1.8 |  +934.7% | (10.3x slower) |
| `bun`          |           8.9 ± 0.3 |  +160.0% | (2.6x slower)  |
| `jsonxf`       |           1.8 ± 0.1 |   -47.7% | (1.9x faster)  |
| `jsonformat`   |           3.4 ± 0.1 |    -2.0% |                |
| `oxfmt`        |          49.3 ± 2.1 | +1332.1% | (14.3x slower) |

### ugly twitter

![candlestick benchmark for ugly printing twitter.json](/benches/docker/output/ugly-twitter.png)

| Command       | Mean Wall Time [ms] |   Change | Factor         |
| :------------ | ------------------: | -------: | :------------- |
| `jjp`         |           2.5 ± 0.1 |          |                |
| `json-minify` |          37.0 ± 2.3 | +1400.9% | (15.0x slower) |
| `jsonxf`      |           1.8 ± 0.1 |   -28.1% |                |
| `jq`          |          14.7 ± 0.3 |  +498.2% | (6.0x slower)  |
| `jaq`         |           8.1 ± 0.2 |  +227.3% | (3.3x slower)  |
| `gojq`        |          12.4 ± 0.5 |  +402.0% | (5.0x slower)  |
| `sjq`         |          38.1 ± 0.9 | +1445.8% | (15.5x slower) |
| `python`      |         137.9 ± 3.1 | +5499.3% | (56.0x slower) |
| `minify`      |           5.9 ± 0.4 |  +141.0% | (2.4x slower)  |
| `node`        |          35.5 ± 2.7 | +1343.6% | (14.4x slower) |
| `bun`         |           8.9 ± 0.9 |  +261.1% | (3.6x slower)  |
