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
| `jjp`          |          28.3 ± 0.5 |          |                |
| `json-pp-rust` |          24.1 ± 0.3 |   -14.9% |                |
| `jsonice`      |          22.6 ± 0.2 |   -20.1% |                |
| `prettier`     |       1015.5 ± 44.4 | +3491.4% | (35.9x slower) |
| `jq`           |          96.6 ± 2.1 |  +241.6% | (3.4x slower)  |
| `gojq`         |          35.3 ± 0.7 |   +24.7% |                |
| `jaq`          |          39.7 ± 1.4 |   +40.5% | (1.4x slower)  |
| `sjq`          |         423.9 ± 4.2 | +1399.1% | (15.0x slower) |
| `dprint`       |        416.5 ± 11.4 | +1373.1% | (14.7x slower) |
| `python`       |         276.0 ± 7.7 |  +876.3% | (9.8x slower)  |
| `node`         |          68.0 ± 2.2 |  +140.6% | (2.4x slower)  |
| `bun`          |          24.0 ± 0.8 |   -15.0% |                |
| `jsonxf`       |          12.4 ± 0.2 |   -56.2% | (2.3x faster)  |
| `jsonformat`   |          15.9 ± 0.2 |   -43.9% | (1.8x faster)  |
| `oxfmt`        |          96.0 ± 5.5 |  +239.5% | (3.4x slower)  |

### ugly canada

![candlestick benchmark for ugly printing canada.json](/benches/docker/output/ugly-canada.png)

| Command       | Mean Wall Time [ms] |   Change | Factor         |
| :------------ | ------------------: | -------: | :------------- |
| `jjp`         |          16.1 ± 0.4 |          |                |
| `json-minify` |          59.8 ± 1.3 |  +270.9% | (3.7x slower)  |
| `jsonxf`      |          12.0 ± 0.2 |   -25.5% |                |
| `jq`          |          70.3 ± 1.9 |  +336.0% | (4.4x slower)  |
| `jaq`         |          32.0 ± 1.1 |   +98.6% | (2.0x slower)  |
| `gojq`        |          33.5 ± 0.8 |  +108.2% | (2.1x slower)  |
| `sjq`         |         338.9 ± 1.8 | +2003.5% | (21.0x slower) |
| `python`      |        252.9 ± 14.3 | +1469.4% | (15.7x slower) |
| `minify`      |          17.3 ± 0.5 |    +7.5% |                |
| `node`        |          55.4 ± 1.9 |  +243.6% | (3.4x slower)  |
| `bun`         |          17.2 ± 0.6 |    +6.7% |                |

## citm catalog

[citm_catalog.json](/benches/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

### pretty citm catalog

![candlestick benchmark for pretty printing citm-catalog.json](/benches/docker/output/pretty-citm_catalog.png)

| Command        | Mean Wall Time [ms] |   Change | Factor         |
| :------------- | ------------------: | -------: | :------------- |
| `jjp`          |           8.7 ± 0.2 |          |                |
| `json-pp-rust` |          12.1 ± 1.1 |   +39.8% | (1.4x slower)  |
| `jsonice`      |           6.5 ± 0.2 |   -25.5% |                |
| `prettier`     |         501.7 ± 9.8 | +5690.3% | (57.9x slower) |
| `jq`           |          31.4 ± 0.6 |  +262.8% | (3.6x slower)  |
| `gojq`         |          19.1 ± 0.3 |  +120.9% | (2.2x slower)  |
| `jaq`          |          14.8 ± 0.5 |   +70.6% | (1.7x slower)  |
| `sjq`          |         111.6 ± 4.2 | +1187.7% | (12.9x slower) |
| `dprint`       |         212.5 ± 2.7 | +2353.1% | (24.5x slower) |
| `python`       |         155.8 ± 1.7 | +1697.7% | (18.0x slower) |
| `node`         |          40.3 ± 1.8 |  +364.8% | (4.6x slower)  |
| `bun`          |          13.6 ± 0.4 |   +56.7% | (1.6x slower)  |
| `jsonxf`       |           3.4 ± 0.1 |   -61.0% | (2.6x faster)  |
| `jsonformat`   |           5.2 ± 0.2 |   -39.8% | (1.7x faster)  |
| `oxfmt`        |          49.8 ± 2.1 |  +474.9% | (5.7x slower)  |

### ugly citm catalog

![candlestick benchmark for ugly printing citm-catalog.json](/benches/docker/output/ugly-citm_catalog.png)

| Command       | Mean Wall Time [ms] |   Change | Factor         |
| :------------ | ------------------: | -------: | :------------- |
| `jjp`         |           4.7 ± 0.2 |          |                |
| `json-minify` |          37.4 ± 1.7 |  +691.0% | (7.9x slower)  |
| `jsonxf`      |           3.3 ± 0.1 |   -30.9% | (1.4x faster)  |
| `jq`          |          26.0 ± 0.4 |  +450.3% | (5.5x slower)  |
| `jaq`         |          12.7 ± 0.4 |  +167.6% | (2.7x slower)  |
| `gojq`        |          19.2 ± 0.8 |  +306.0% | (4.1x slower)  |
| `sjq`         |          88.9 ± 4.5 | +1777.9% | (18.8x slower) |
| `python`      |        156.7 ± 12.4 | +3210.9% | (33.1x slower) |
| `minify`      |           7.4 ± 0.4 |   +56.1% | (1.6x slower)  |
| `node`        |          34.9 ± 1.7 |  +637.9% | (7.4x slower)  |
| `bun`         |          10.2 ± 0.3 |  +115.9% | (2.2x slower)  |

## twitter

[twitter.json](/benches/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

### pretty twitter

![candlestick benchmark for pretty printing twitter.json](/benches/docker/output/pretty-twitter.png)

| Command        | Mean Wall Time [ms] |   Change | Factor         |
| :------------- | ------------------: | -------: | :------------- |
| `jjp`          |           4.2 ± 0.1 |          |                |
| `json-pp-rust` |           5.1 ± 0.2 |   +21.4% |                |
| `jsonice`      |           3.9 ± 0.1 |    -7.3% |                |
| `prettier`     |         274.6 ± 3.7 | +6418.9% | (65.2x slower) |
| `jq`           |          15.0 ± 0.3 |  +255.6% | (3.6x slower)  |
| `gojq`         |          11.5 ± 0.4 |  +172.9% | (2.7x slower)  |
| `jaq`          |           8.1 ± 0.3 |   +91.9% | (1.9x slower)  |
| `sjq`          |          40.2 ± 0.5 |  +855.2% | (9.6x slower)  |
| `dprint`       |          43.3 ± 0.9 |  +927.2% | (10.3x slower) |
| `python`       |         127.7 ± 1.5 | +2930.8% | (30.3x slower) |
| `node`         |          32.7 ± 1.5 |  +676.4% | (7.8x slower)  |
| `bun`          |           8.3 ± 0.3 |   +96.5% | (2.0x slower)  |
| `jsonxf`       |           1.8 ± 0.1 |   -58.4% | (2.4x faster)  |
| `jsonformat`   |           3.2 ± 0.1 |   -24.2% |                |
| `oxfmt`        |          44.1 ± 2.6 |  +946.4% | (10.5x slower) |

### ugly twitter

![candlestick benchmark for ugly printing twitter.json](/benches/docker/output/ugly-twitter.png)

| Command       | Mean Wall Time [ms] |   Change | Factor         |
| :------------ | ------------------: | -------: | :------------- |
| `jjp`         |           3.0 ± 0.1 |          |                |
| `json-minify` |          33.8 ± 2.3 | +1019.5% | (11.2x slower) |
| `jsonxf`      |           1.8 ± 0.2 |   -39.7% | (1.7x faster)  |
| `jq`          |          13.5 ± 0.2 |  +346.9% | (4.5x slower)  |
| `jaq`         |           7.5 ± 0.2 |  +146.9% | (2.5x slower)  |
| `gojq`        |          11.3 ± 0.4 |  +275.8% | (3.8x slower)  |
| `sjq`         |          35.0 ± 0.4 | +1058.8% | (11.6x slower) |
| `python`      |         126.2 ± 1.9 | +4081.9% | (41.8x slower) |
| `minify`      |           5.3 ± 0.4 |   +74.9% | (1.7x slower)  |
| `node`        |          30.4 ± 1.1 |  +906.1% | (10.1x slower) |
| `bun`         |           7.4 ± 0.2 |  +145.7% | (2.5x slower)  |
