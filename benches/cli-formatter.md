<!-- GENERATED FILE - update the templates in the xtask -->

# jjpwrgem CLI formatter and minifier benchmarks

Wall-clock timing against jq, prettier, dprint, and others via hyperfine. Run locally with `mise run bench-docker`
These benchmarks are run with `AMD Ryzen 5 5600X 6-Core Processor (3.70 GHz)`

The following JSON fixtures are used across benchmarks:

- [canada.json](/benches/docker/data/json-benchmark/data/canada.json) — 2.2MB, lots of lightly nested arrays, no strings

- [citm_catalog.json](/benches/docker/data/json-benchmark/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

- [twitter.json](/benches/docker/data/json-benchmark/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

## canada

[canada.json](/benches/docker/data/json-benchmark/data/canada.json) — 2.2MB, lots of lightly nested arrays, no strings

### pretty canada

![candlestick benchmark for pretty printing canada.json](/benches/docker/output/pretty-canada.png)

| Command        |     Mean [ms] | Min [ms] | Max [ms] |     Relative |
| :------------- | ------------: | -------: | -------: | -----------: |
| `jsonxf`       |    13.0 ± 0.9 |     12.0 |     18.7 |         1.00 |
| `jsonformat`   |    14.2 ± 0.6 |     13.4 |     17.6 |  1.09 ± 0.09 |
| `jsonice`      |    22.0 ± 1.3 |     20.3 |     28.5 |  1.69 ± 0.15 |
| `json-pp-rust` |    29.9 ± 2.0 |     27.4 |     36.2 |  2.30 ± 0.22 |
| `jjp`          |    34.1 ± 8.1 |     27.9 |     66.6 |  2.63 ± 0.65 |
| `gojq`         |    39.0 ± 1.7 |     36.6 |     48.0 |  3.00 ± 0.24 |
| `jaq`          |    48.6 ± 1.4 |     46.8 |     54.4 |  3.74 ± 0.27 |
| `bun`          |    54.3 ± 2.2 |     51.8 |     61.9 |  4.18 ± 0.32 |
| `node`         |    84.5 ± 1.4 |     81.6 |     87.3 |  6.51 ± 0.44 |
| `jq`           |    93.2 ± 1.4 |     91.5 |     99.0 |  7.18 ± 0.49 |
| `python`       |   313.0 ± 3.5 |    307.6 |    319.1 | 24.11 ± 1.61 |
| `sjq`          |   465.8 ± 5.2 |    459.6 |    477.5 | 35.89 ± 2.40 |
| `dprint`       |  670.6 ± 10.5 |    654.1 |    692.0 | 51.66 ± 3.50 |
| `prettier`     | 1143.1 ± 28.5 |   1108.3 |   1206.0 | 88.06 ± 6.21 |
| `oxfmt`        | 1188.2 ± 18.2 |   1171.0 |   1231.9 | 91.54 ± 6.19 |

### ugly canada

![candlestick benchmark for ugly printing canada.json](/benches/docker/output/ugly-canada.png)

| Command       |    Mean [ms] | Min [ms] | Max [ms] |     Relative |
| :------------ | -----------: | -------: | -------: | -----------: |
| `jsonxf`      |   12.6 ± 0.5 |     11.5 |     15.3 |         1.00 |
| `jjp`         |   14.3 ± 0.6 |     13.3 |     16.8 |  1.13 ± 0.06 |
| `minify`      |   20.9 ± 1.4 |     19.2 |     27.0 |  1.65 ± 0.13 |
| `jaq`         |   36.1 ± 0.9 |     34.7 |     39.2 |  2.86 ± 0.13 |
| `gojq`        |   41.2 ± 1.6 |     38.6 |     48.4 |  3.25 ± 0.18 |
| `bun`         |   42.1 ± 1.2 |     40.2 |     47.2 |  3.33 ± 0.16 |
| `node`        |   69.7 ± 1.6 |     66.5 |     75.1 |  5.51 ± 0.25 |
| `jq`          |   70.5 ± 3.3 |     65.8 |     77.9 |  5.57 ± 0.34 |
| `json-minify` |   74.5 ± 4.7 |     71.3 |     99.2 |  5.89 ± 0.44 |
| `python`      |  289.0 ± 2.9 |    286.4 |    295.8 | 22.86 ± 0.91 |
| `sjq`         | 377.5 ± 16.1 |    361.1 |    397.7 | 29.85 ± 1.72 |

## citm catalog

[citm_catalog.json](/benches/docker/data/json-benchmark/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

### pretty citm catalog

![candlestick benchmark for pretty printing citm-catalog.json](/benches/docker/output/pretty-citm_catalog.png)

| Command        |    Mean [ms] | Min [ms] | Max [ms] |       Relative |
| :------------- | -----------: | -------: | -------: | -------------: |
| `jsonxf`       |    4.9 ± 0.4 |      4.4 |      7.7 |           1.00 |
| `jsonice`      |    7.9 ± 0.4 |      7.3 |     11.4 |    1.61 ± 0.15 |
| `jsonformat`   |    8.2 ± 0.5 |      7.4 |     12.7 |    1.68 ± 0.17 |
| `jjp`          |   11.2 ± 1.0 |     10.3 |     21.4 |    2.29 ± 0.26 |
| `json-pp-rust` |   15.5 ± 1.3 |     14.0 |     23.9 |    3.18 ± 0.37 |
| `jaq`          |   19.4 ± 0.9 |     18.3 |     25.5 |    3.97 ± 0.35 |
| `gojq`         |   28.4 ± 1.4 |     26.8 |     36.4 |    5.80 ± 0.52 |
| `jq`           |   36.6 ± 1.0 |     35.0 |     39.7 |    7.48 ± 0.60 |
| `bun`          |   39.4 ± 1.6 |     37.1 |     44.1 |    8.05 ± 0.70 |
| `node`         |   53.7 ± 2.2 |     50.9 |     60.1 |   10.98 ± 0.94 |
| `sjq`          |  135.8 ± 2.6 |    133.1 |    144.3 |   27.76 ± 2.18 |
| `dprint`       |  154.2 ± 4.8 |    148.7 |    168.6 |   31.52 ± 2.59 |
| `python`       |  191.5 ± 3.1 |    187.2 |    199.2 |   39.14 ± 3.05 |
| `prettier`     | 593.5 ± 19.0 |    579.3 |    640.5 | 121.32 ± 10.02 |
| `oxfmt`        | 712.7 ± 15.1 |    696.1 |    746.2 | 145.68 ± 11.51 |

### ugly citm catalog

![candlestick benchmark for ugly printing citm-catalog.json](/benches/docker/output/ugly-citm_catalog.png)

| Command       |   Mean [ms] | Min [ms] | Max [ms] |     Relative |
| :------------ | ----------: | -------: | -------: | -----------: |
| `jsonxf`      |   4.7 ± 0.3 |      4.3 |      7.3 |         1.00 |
| `jjp`         |   6.6 ± 0.4 |      6.1 |      9.2 |  1.41 ± 0.13 |
| `minify`      |  14.9 ± 3.2 |     13.2 |     56.2 |  3.16 ± 0.72 |
| `jaq`         |  15.8 ± 0.9 |     14.8 |     21.8 |  3.35 ± 0.30 |
| `gojq`        |  27.9 ± 1.3 |     26.2 |     34.0 |  5.91 ± 0.51 |
| `jq`          |  31.4 ± 1.2 |     30.0 |     37.4 |  6.65 ± 0.53 |
| `bun`         |  33.8 ± 1.5 |     31.7 |     40.5 |  7.16 ± 0.60 |
| `node`        |  46.9 ± 2.1 |     43.2 |     52.5 |  9.94 ± 0.83 |
| `json-minify` |  50.1 ± 2.2 |     45.7 |     56.2 | 10.62 ± 0.89 |
| `sjq`         | 112.8 ± 1.5 |    110.8 |    116.6 | 23.90 ± 1.73 |
| `python`      | 186.0 ± 6.1 |    181.6 |    205.7 | 39.41 ± 3.08 |

## twitter

[twitter.json](/benches/docker/data/json-benchmark/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

### pretty twitter

![candlestick benchmark for pretty printing twitter.json](/benches/docker/output/pretty-twitter.png)

| Command        |    Mean [ms] | Min [ms] | Max [ms] |       Relative |
| :------------- | -----------: | -------: | -------: | -------------: |
| `jsonxf`       |    2.1 ± 0.3 |      1.7 |      4.6 |           1.00 |
| `jsonformat`   |    3.7 ± 0.3 |      3.3 |      6.2 |    1.70 ± 0.29 |
| `jsonice`      |    4.1 ± 0.3 |      3.6 |      6.7 |    1.91 ± 0.33 |
| `jjp`          |    5.3 ± 0.4 |      4.7 |      7.7 |    2.47 ± 0.42 |
| `json-pp-rust` |    6.4 ± 0.5 |      5.8 |      9.4 |    2.98 ± 0.50 |
| `jaq`          |   10.0 ± 0.5 |      9.3 |     13.0 |    4.68 ± 0.76 |
| `gojq`         |   15.2 ± 2.1 |     13.0 |     39.0 |    7.09 ± 1.46 |
| `jq`           |   17.2 ± 0.6 |     16.5 |     20.0 |    8.00 ± 1.26 |
| `bun`          |   26.6 ± 1.1 |     24.8 |     31.2 |   12.42 ± 1.97 |
| `node`         |   43.6 ± 2.1 |     40.8 |     51.3 |   20.33 ± 3.27 |
| `sjq`          |   45.1 ± 1.5 |     43.2 |     50.6 |   21.02 ± 3.30 |
| `dprint`       |   61.6 ± 1.5 |     59.1 |     65.0 |   28.72 ± 4.47 |
| `python`       |  162.7 ± 5.9 |    156.6 |    181.0 |  75.86 ± 11.96 |
| `prettier`     | 359.8 ± 16.8 |    333.6 |    385.7 | 167.83 ± 26.93 |
| `oxfmt`        | 444.5 ± 13.6 |    431.5 |    478.5 | 207.30 ± 32.44 |

### ugly twitter

![candlestick benchmark for ugly printing twitter.json](/benches/docker/output/ugly-twitter.png)

| Command       |   Mean [ms] | Min [ms] | Max [ms] |     Relative |
| :------------ | ----------: | -------: | -------: | -----------: |
| `jsonxf`      |   2.2 ± 0.2 |      1.9 |      3.7 |         1.00 |
| `jjp`         |   4.0 ± 0.3 |      3.6 |      6.7 |  1.84 ± 0.23 |
| `minify`      |   7.7 ± 2.1 |      6.6 |     46.2 |  3.50 ± 1.02 |
| `jaq`         |   9.0 ± 0.6 |      8.3 |     13.3 |  4.07 ± 0.46 |
| `gojq`        |  14.1 ± 0.7 |     12.8 |     16.5 |  6.42 ± 0.67 |
| `jq`          |  14.9 ± 1.1 |     13.8 |     20.7 |  6.78 ± 0.80 |
| `bun`         |  26.1 ± 1.7 |     23.6 |     33.8 | 11.86 ± 1.32 |
| `sjq`         |  39.2 ± 2.2 |     37.0 |     49.5 | 17.85 ± 1.92 |
| `node`        |  41.5 ± 2.5 |     38.9 |     54.6 | 18.87 ± 2.07 |
| `json-minify` |  43.4 ± 1.6 |     40.8 |     47.1 | 19.74 ± 1.95 |
| `python`      | 161.1 ± 4.6 |    156.7 |    174.6 | 73.26 ± 7.03 |
