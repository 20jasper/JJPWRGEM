<!-- GENERATED FILE - update the templates in the xtask -->

# jjpwrgem CLI formatter and minifier benchmarks

Wall-clock timing against jq, prettier, dprint, and others via hyperfine. Run locally with `just bench-docker`
These benchmarks are run with `AMD Ryzen 5 5600X 6-Core Processor (3.70 GHz)`

The following JSON fixtures are used across benchmarks:

- [canada.json](/xtask/bench/data/json-benchmark/data/canada.json) — 2.2MB, lots of lightly nested arrays, no strings

- [citm_catalog.json](/xtask/bench/data/json-benchmark/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

- [twitter.json](/xtask/bench/data/json-benchmark/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

## canada

[canada.json](/xtask/bench/data/json-benchmark/data/canada.json) — 2.2MB, lots of lightly nested arrays, no strings

### pretty canada

![candlestick benchmark for pretty printing canada.json](/xtask/bench/output/pretty-canada.png)

| Command        |      Mean [ms] | Min [ms] | Max [ms] |       Relative |
| :------------- | -------------: | -------: | -------: | -------------: |
| `jsonxf`       |     13.4 ± 1.7 |     11.1 |     20.5 |           1.00 |
| `jsonformat`   |     17.7 ± 3.7 |     13.4 |     34.5 |    1.33 ± 0.32 |
| `jsonice`      |     23.5 ± 4.8 |     19.2 |     46.7 |    1.76 ± 0.43 |
| `json-pp-rust` |     27.8 ± 2.3 |     25.2 |     35.4 |    2.08 ± 0.32 |
| `jjp`          |     33.1 ± 4.1 |     28.0 |     46.2 |    2.48 ± 0.44 |
| `gojq`         |     60.2 ± 5.5 |     52.3 |     76.0 |    4.50 ± 0.71 |
| `jaq`          |     71.6 ± 8.1 |     63.6 |    100.0 |    5.36 ± 0.92 |
| `bun`          |     81.7 ± 6.7 |     71.8 |     96.7 |    6.12 ± 0.94 |
| `node`         |    95.0 ± 18.0 |     80.7 |    179.2 |    7.11 ± 1.63 |
| `jshon`        |    109.1 ± 5.0 |     99.0 |    118.3 |    8.16 ± 1.12 |
| `jq`           |   126.9 ± 13.7 |    108.8 |    160.8 |    9.49 ± 1.60 |
| `python`       |   275.1 ± 21.8 |    257.1 |    334.5 |   20.59 ± 3.12 |
| `jello`        |   358.2 ± 17.4 |    332.7 |    381.5 |   26.80 ± 3.70 |
| `sjq`          |   478.1 ± 30.1 |    452.2 |    544.7 |   35.78 ± 5.14 |
| `dprint`       |   739.9 ± 25.9 |    716.2 |    798.9 |   55.37 ± 7.41 |
| `prettier`     | 1253.7 ± 125.6 |   1113.2 |   1572.5 |  93.82 ± 15.33 |
| `oxfmt`        | 1411.2 ± 142.7 |   1265.6 |   1602.3 | 105.61 ± 17.32 |

### ugly canada

![candlestick benchmark for ugly printing canada.json](/xtask/bench/output/ugly-canada.png)

| Command       |    Mean [ms] | Min [ms] | Max [ms] |     Relative |
| :------------ | -----------: | -------: | -------: | -----------: |
| `jsonxf`      |   13.7 ± 2.3 |     10.7 |     20.7 |         1.00 |
| `jjp`         |   18.4 ± 2.6 |     15.6 |     38.2 |  1.35 ± 0.29 |
| `minify`      |  43.4 ± 14.4 |     33.9 |    141.5 |  3.17 ± 1.18 |
| `jaq`         |   55.9 ± 2.9 |     51.6 |     64.1 |  4.09 ± 0.71 |
| `bun`         |   61.7 ± 6.5 |     54.9 |     83.3 |  4.51 ± 0.89 |
| `gojq`        |   64.7 ± 5.7 |     56.9 |     80.2 |  4.73 ± 0.89 |
| `node`        |   75.9 ± 4.9 |     66.4 |     86.8 |  5.55 ± 0.99 |
| `jq`          |  100.5 ± 8.0 |     88.6 |    119.1 |  7.34 ± 1.36 |
| `json-minify` | 101.4 ± 15.3 |     84.3 |    150.6 |  7.41 ± 1.67 |
| `python`      | 263.6 ± 17.9 |    234.5 |    299.4 | 19.26 ± 3.48 |
| `jello`       | 341.4 ± 10.7 |    325.8 |    357.3 | 24.95 ± 4.24 |
| `sjq`         | 415.0 ± 21.5 |    387.9 |    462.7 | 30.33 ± 5.31 |

## citm catalog

[citm_catalog.json](/xtask/bench/data/json-benchmark/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

### pretty citm catalog

![candlestick benchmark for pretty printing citm-catalog.json](/xtask/bench/output/pretty-citm_catalog.png)

| Command        |    Mean [ms] | Min [ms] | Max [ms] |       Relative |
| :------------- | -----------: | -------: | -------: | -------------: |
| `jsonxf`       |    4.9 ± 0.7 |      4.0 |      8.4 |           1.00 |
| `jsonformat`   |    7.5 ± 0.6 |      6.8 |     12.1 |    1.51 ± 0.25 |
| `jsonice`      |    9.7 ± 1.9 |      8.2 |     22.1 |    1.96 ± 0.49 |
| `jjp`          |   16.4 ± 2.4 |     13.4 |     25.9 |    3.31 ± 0.68 |
| `json-pp-rust` |   18.0 ± 2.0 |     15.3 |     23.5 |    3.63 ± 0.66 |
| `jshon`        |   29.6 ± 2.3 |     27.5 |     42.2 |    5.98 ± 0.99 |
| `jaq`          |  41.9 ± 12.3 |     34.3 |    109.7 |    8.48 ± 2.77 |
| `gojq`         |   44.3 ± 4.7 |     40.8 |     73.6 |    8.97 ± 1.61 |
| `jq`           |   54.8 ± 8.0 |     49.1 |     84.7 |   11.07 ± 2.29 |
| `bun`          |   59.0 ± 3.2 |     55.5 |     68.6 |   11.93 ± 1.86 |
| `node`         |   60.4 ± 5.7 |     53.9 |     83.2 |   12.21 ± 2.13 |
| `sjq`          |  144.2 ± 7.6 |    136.9 |    166.9 |   29.16 ± 4.53 |
| `python`       |  153.9 ± 8.5 |    146.9 |    175.1 |   31.13 ± 4.87 |
| `dprint`       |  160.6 ± 9.6 |    153.7 |    188.6 |   32.48 ± 5.13 |
| `jello`        | 399.2 ± 57.5 |    351.6 |    550.6 |  80.73 ± 16.57 |
| `prettier`     | 673.3 ± 29.8 |    620.5 |    733.5 | 136.14 ± 20.80 |
| `oxfmt`        | 850.6 ± 58.2 |    762.0 |    988.9 | 172.01 ± 27.78 |

### ugly citm catalog

![candlestick benchmark for ugly printing citm-catalog.json](/xtask/bench/output/ugly-citm_catalog.png)

| Command       |    Mean [ms] | Min [ms] | Max [ms] |      Relative |
| :------------ | -----------: | -------: | -------: | ------------: | --- | ------- | --------- | -------- | -------- | -------- |
| `jsonxf`      |    4.9 ± 0.7 |      4.3 |     10.0 |          1.00 |
| `jjp`         |   10.1 ± 0.8 |      9.2 |     16.4 |   2.06 ± 0.34 |
| `jaq`         |   33.7 ± 2.2 |     30.4 |     39.0 |   6.88 ± 1.07 |
| `minify`      |   34.0 ± 2.9 |     30.1 |     42.7 |   6.94 ± 1.15 |
| `gojq`        |  51.1 ± 11.3 |     41.7 |    130.5 |  10.44 ± 2.74 |
| `node`        |   52.6 ± 7.3 |     43.1 |     82.7 |  10.74 ± 2.13 |
| `bun`         |   52.8 ± 6.2 |     46.6 |     83.9 |  10.79 ± 1.98 |
| `jq`          |   56.2 ± 4.8 |     49.7 |     69.5 |  11.47 ± 1.90 |
| `json-minify` |   72.0 ± 7.0 |     65.5 |     99.0 |  14.72 ± 2.53 |
| `sjq`         |  115.9 ± 9.5 |    105.9 |    135.4 |  23.67 ± 3.88 |
| `python`      |  140.7 ± 6.8 |    133.8 |    162.5 |  28.75 ± 4.30 |
| `jello`       | 381.7 ± 20.3 |    353.1 |    428.6 | 77.99 ± 11.79 |
| 64.7          | 13.31 ± 0.47 |
| `sjq`         |  113.0 ± 0.8 |    111.5 |    115.0 |  24.98 ± 0.66 |
| `python`      |  135.1 ± 1.0 |    133.8 |    136.8 |  29.88 ± 0.79 |
| `jello`       |  306.8 ± 1.2 |    304.5 |    308.6 |  67.83 ± 1.74 |     | Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
| :---          |         ---: |     ---: |     ---: |          ---: |
| `jsonxf`      |    4.9 ± 0.7 |      4.3 |     10.0 |          1.00 |
| `jjp`         |   10.1 ± 0.8 |      9.2 |     16.4 |   2.06 ± 0.34 |
| `json-minify` |   72.0 ± 7.0 |     65.5 |     99.0 |  14.72 ± 2.53 |
| `jello`       | 381.7 ± 20.3 |    353.1 |    428.6 | 77.99 ± 11.79 |

## twitter

[twitter.json](/xtask/bench/data/json-benchmark/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

### pretty twitter

![candlestick benchmark for pretty printing twitter.json](/xtask/bench/output/pretty-twitter.png)

| Command        |    Mean [ms] | Min [ms] | Max [ms] |       Relative |
| :------------- | -----------: | -------: | -------: | -------------: |
| `jsonxf`       |    2.0 ± 0.3 |      1.6 |      3.9 |           1.00 |
| `jsonformat`   |    4.2 ± 1.1 |      3.1 |     14.0 |    2.14 ± 0.65 |
| `jsonice`      |    4.5 ± 0.8 |      3.7 |     12.1 |    2.27 ± 0.54 |
| `json-pp-rust` |    6.5 ± 0.7 |      5.5 |      9.8 |    3.28 ± 0.66 |
| `jjp`          |    6.9 ± 1.0 |      5.6 |     12.3 |    3.50 ± 0.77 |
| `jshon`        |   14.0 ± 1.4 |     12.7 |     20.8 |    7.11 ± 1.38 |
| `jaq`          |   27.9 ± 3.0 |     25.0 |     52.5 |   14.20 ± 2.80 |
| `gojq`         |   33.4 ± 2.0 |     30.5 |     40.5 |   17.01 ± 3.00 |
| `jq`           |   34.8 ± 2.4 |     31.7 |     44.8 |   17.70 ± 3.18 |
| `node`         |   42.4 ± 2.0 |     39.1 |     48.6 |   21.60 ± 3.73 |
| `bun`          |   42.7 ± 3.1 |     38.4 |     51.3 |   21.71 ± 3.93 |
| `sjq`          |   48.1 ± 5.9 |     41.7 |     74.3 |   24.46 ± 5.06 |
| `dprint`       |  70.2 ± 19.7 |     57.8 |    175.7 |  35.75 ± 11.63 |
| `python`       |  123.2 ± 9.9 |    109.9 |    146.9 |  62.72 ± 11.54 |
| `jello`        | 277.9 ± 17.4 |    261.2 |    314.1 | 141.44 ± 25.04 |
| `prettier`     | 397.9 ± 25.8 |    342.6 |    424.2 | 202.50 ± 36.03 |
| `oxfmt`        | 519.2 ± 30.2 |    473.7 |    566.1 | 264.27 ± 46.41 |

### ugly twitter

![candlestick benchmark for ugly printing twitter.json](/xtask/bench/output/ugly-twitter.png)

| Command       |    Mean [ms] | Min [ms] | Max [ms] |       Relative |
| :------------ | -----------: | -------: | -------: | -------------: |
| `jsonxf`      |    1.8 ± 0.2 |      1.6 |      3.2 |           1.00 |
| `jjp`         |    5.3 ± 0.8 |      4.3 |      8.1 |    2.86 ± 0.54 |
| `minify`      |   25.4 ± 4.8 |     22.6 |     58.3 |   13.75 ± 2.97 |
| `jaq`         |   30.8 ± 6.5 |     25.2 |     64.7 |   16.64 ± 3.92 |
| `jq`          |   34.7 ± 2.8 |     30.2 |     44.5 |   18.79 ± 2.48 |
| `gojq`        |   35.0 ± 5.4 |     30.3 |     63.0 |   18.96 ± 3.53 |
| `sjq`         |   38.2 ± 3.6 |     34.7 |     51.2 |   20.65 ± 2.93 |
| `bun`         |   39.8 ± 2.9 |     36.7 |     49.5 |   21.51 ± 2.75 |
| `node`        |   40.2 ± 3.0 |     36.2 |     53.4 |   21.74 ± 2.80 |
| `json-minify` |   66.6 ± 6.1 |     54.9 |     86.1 |   36.05 ± 5.01 |
| `python`      |  118.6 ± 9.2 |    107.2 |    138.1 |   64.16 ± 8.36 |
| `jello`       | 304.6 ± 22.7 |    270.2 |    351.1 | 164.74 ± 21.21 |
