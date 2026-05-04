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

| Command        |     Mean [ms] | Min [ms] | Max [ms] |     Relative |
| :------------- | ------------: | -------: | -------: | -----------: |
| `jsonxf`       |    12.3 ± 0.4 |     11.7 |     17.1 |         1.00 |
| `jsonformat`   |    14.0 ± 0.3 |     13.4 |     16.4 |  1.14 ± 0.04 |
| `jsonice`      |    20.4 ± 0.4 |     20.0 |     24.5 |  1.66 ± 0.06 |
| `json-pp-rust` |    26.7 ± 0.4 |     25.9 |     28.3 |  2.18 ± 0.08 |
| `jjp`          |    27.8 ± 0.8 |     26.9 |     31.6 |  2.27 ± 0.09 |
| `gojq`         |    54.5 ± 5.3 |     52.2 |     92.9 |  4.45 ± 0.46 |
| `jaq`          |    65.3 ± 0.6 |     64.1 |     67.0 |  5.33 ± 0.17 |
| `bun`          |    68.5 ± 3.3 |     65.9 |     87.7 |  5.59 ± 0.32 |
| `node`         |    84.7 ± 1.8 |     82.5 |     91.2 |  6.91 ± 0.26 |
| `jshon`        |    98.5 ± 0.7 |     97.5 |    100.1 |  8.04 ± 0.25 |
| `jq`           |   108.4 ± 3.1 |    106.5 |    123.7 |  8.85 ± 0.37 |
| `python`       |   260.6 ± 3.5 |    257.1 |    268.5 | 21.26 ± 0.71 |
| `jello`        |   336.1 ± 4.0 |    328.3 |    342.0 | 27.42 ± 0.90 |
| `sjq`          |   451.7 ± 4.2 |    446.9 |    461.7 | 36.86 ± 1.17 |
| `dprint`       |  660.5 ± 24.7 |    642.9 |    707.9 | 53.90 ± 2.60 |
| `prettier`     | 1107.3 ± 12.6 |   1095.3 |   1137.4 | 90.36 ± 2.94 |
| `oxfmt`        | 1171.1 ± 15.1 |   1150.4 |   1199.5 | 95.56 ± 3.16 |

### ugly canada

![candlestick benchmark for ugly printing canada.json](/xtask/bench/output/ugly-canada.png)

| Command       |    Mean [ms] | Min [ms] | Max [ms] |     Relative |
| :------------ | -----------: | -------: | -------: | -----------: |
| `jsonxf`      |   12.2 ± 0.4 |     11.4 |     16.9 |         1.00 |
| `jjp`         |   14.0 ± 0.5 |     13.2 |     17.0 |  1.14 ± 0.06 |
| `minify`      |   36.8 ± 0.8 |     35.8 |     40.3 |  3.01 ± 0.12 |
| `jaq`         |   50.6 ± 1.3 |     49.1 |     55.3 |  4.15 ± 0.18 |
| `gojq`        |   52.0 ± 0.8 |     50.2 |     53.5 |  4.26 ± 0.16 |
| `bun`         |   56.4 ± 1.0 |     55.2 |     61.5 |  4.62 ± 0.18 |
| `node`        |   68.3 ± 1.0 |     66.2 |     71.2 |  5.60 ± 0.22 |
| `jq`          |   82.4 ± 0.7 |     81.3 |     84.7 |  6.75 ± 0.25 |
| `json-minify` |   88.5 ± 1.0 |     86.9 |     92.0 |  7.25 ± 0.27 |
| `python`      |  238.4 ± 1.3 |    236.2 |    241.0 | 19.53 ± 0.70 |
| `jello`       |  327.2 ± 2.2 |    324.8 |    332.5 | 26.80 ± 0.97 |
| `sjq`         | 360.8 ± 14.3 |    350.3 |    400.4 | 29.55 ± 1.57 |

## citm catalog

[citm_catalog.json](/xtask/bench/data/json-benchmark/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

### pretty citm catalog

![candlestick benchmark for pretty printing citm-catalog.json](/xtask/bench/output/pretty-citm_catalog.png)

| Command        |   Mean [ms] | Min [ms] | Max [ms] |      Relative |
| :------------- | ----------: | -------: | -------: | ------------: |
| `jsonxf`       |   4.5 ± 0.2 |      4.3 |      7.4 |          1.00 |
| `jsonformat`   |   7.6 ± 0.2 |      7.1 |      9.0 |   1.69 ± 0.10 |
| `jsonice`      |   8.3 ± 0.2 |      8.0 |      9.1 |   1.84 ± 0.10 |
| `jjp`          |  10.6 ± 0.3 |     10.1 |     11.6 |   2.35 ± 0.13 |
| `json-pp-rust` |  14.4 ± 0.3 |     13.9 |     16.3 |   3.19 ± 0.18 |
| `jshon`        |  27.5 ± 0.4 |     27.0 |     28.9 |   6.10 ± 0.32 |
| `jaq`          |  39.3 ± 1.0 |     37.9 |     44.0 |   8.72 ± 0.50 |
| `gojq`         |  46.2 ± 1.9 |     42.9 |     48.8 |  10.25 ± 0.67 |
| `node`         |  51.7 ± 1.7 |     49.4 |     55.8 |  11.47 ± 0.69 |
| `jq`           |  52.4 ± 0.6 |     51.2 |     53.9 |  11.61 ± 0.60 |
| `bun`          |  52.4 ± 0.7 |     51.4 |     55.5 |  11.62 ± 0.61 |
| `sjq`          | 141.1 ± 0.8 |    139.6 |    142.4 |  31.27 ± 1.60 |
| `python`       | 154.5 ± 3.3 |    143.5 |    161.3 |  34.26 ± 1.89 |
| `dprint`       | 159.4 ± 1.6 |    157.2 |    162.2 |  35.33 ± 1.83 |
| `jello`        | 319.3 ± 1.6 |    316.8 |    321.9 |  70.81 ± 3.63 |
| `prettier`     | 584.6 ± 3.6 |    579.8 |    590.1 | 129.61 ± 6.65 |
| `oxfmt`        | 701.2 ± 3.6 |    697.2 |    707.8 | 155.48 ± 7.96 |

### ugly citm catalog

![candlestick benchmark for ugly printing citm-catalog.json](/xtask/bench/output/ugly-citm_catalog.png)

| Command       |   Mean [ms] | Min [ms] | Max [ms] |     Relative |
| :------------ | ----------: | -------: | -------: | -----------: |
| `jsonxf`      |   4.5 ± 0.2 |      4.3 |      6.7 |         1.00 |
| `jjp`         |   5.8 ± 0.2 |      5.5 |      6.9 |  1.29 ± 0.06 |
| `minify`      |  32.1 ± 1.6 |     29.7 |     36.8 |  7.17 ± 0.44 |
| `jaq`         |  32.7 ± 0.7 |     31.8 |     36.3 |  7.30 ± 0.29 |
| `gojq`        |  43.9 ± 2.6 |     42.4 |     64.7 |  9.79 ± 0.67 |
| `node`        |  45.5 ± 1.7 |     42.9 |     49.3 | 10.16 ± 0.51 |
| `jq`          |  47.1 ± 0.5 |     46.3 |     49.8 | 10.51 ± 0.37 |
| `bun`         |  47.8 ± 0.6 |     46.8 |     50.0 | 10.68 ± 0.38 |
| `json-minify` |  64.2 ± 1.6 |     62.2 |     67.9 | 14.32 ± 0.60 |
| `sjq`         | 110.6 ± 3.5 |    108.2 |    120.3 | 24.69 ± 1.13 |
| `python`      | 148.7 ± 1.4 |    146.5 |    151.9 | 33.18 ± 1.15 |
| `jello`       | 315.7 ± 2.7 |    311.5 |    319.5 | 70.46 ± 2.44 |

## twitter

[twitter.json](/xtask/bench/data/json-benchmark/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

### pretty twitter

![candlestick benchmark for pretty printing twitter.json](/xtask/bench/output/pretty-twitter.png)

| Command        |   Mean [ms] | Min [ms] | Max [ms] |       Relative |
| :------------- | ----------: | -------: | -------: | -------------: |
| `jsonxf`       |   2.0 ± 0.1 |      1.8 |      2.4 |           1.00 |
| `jsonformat`   |   3.6 ± 0.2 |      3.3 |      5.1 |    1.82 ± 0.12 |
| `jsonice`      |   4.2 ± 0.1 |      4.0 |      5.2 |    2.11 ± 0.11 |
| `jjp`          |   5.1 ± 0.1 |      4.8 |      6.1 |    2.57 ± 0.14 |
| `json-pp-rust` |   6.1 ± 0.2 |      5.7 |      8.7 |    3.06 ± 0.17 |
| `jshon`        |  14.0 ± 0.5 |     13.6 |     18.3 |    7.08 ± 0.41 |
| `jaq`          |  27.1 ± 2.3 |     26.1 |     50.3 |   13.68 ± 1.32 |
| `gojq`         |  30.6 ± 0.6 |     29.3 |     32.4 |   15.46 ± 0.77 |
| `jq`           |  32.4 ± 0.8 |     31.5 |     37.0 |   16.37 ± 0.86 |
| `bun`          |  40.7 ± 0.5 |     39.7 |     42.1 |   20.53 ± 0.98 |
| `node`         |  41.6 ± 1.2 |     39.9 |     47.2 |   20.98 ± 1.16 |
| `sjq`          |  43.8 ± 3.5 |     42.5 |     64.9 |   22.13 ± 2.06 |
| `dprint`       |  57.5 ± 0.7 |     56.3 |     59.0 |   29.00 ± 1.39 |
| `python`       | 114.3 ± 2.6 |    112.1 |    125.6 |   57.70 ± 2.98 |
| `jello`        | 268.3 ± 2.8 |    264.9 |    273.0 |  135.41 ± 6.43 |
| `prettier`     | 341.0 ± 2.8 |    336.9 |    345.8 |  172.12 ± 8.08 |
| `oxfmt`        | 467.4 ± 1.9 |    465.6 |    471.4 | 235.97 ± 10.96 |

### ugly twitter

![candlestick benchmark for ugly printing twitter.json](/xtask/bench/output/ugly-twitter.png)

| Command       |   Mean [ms] | Min [ms] | Max [ms] |      Relative |
| :------------ | ----------: | -------: | -------: | ------------: |
| `jsonxf`      |   1.9 ± 0.1 |      1.7 |      3.0 |          1.00 |
| `jjp`         |   3.6 ± 0.1 |      3.4 |      4.2 |   1.89 ± 0.12 |
| `minify`      |  24.3 ± 0.4 |     23.5 |     25.5 |  12.71 ± 0.69 |
| `jaq`         |  25.8 ± 2.2 |     24.7 |     48.7 |  13.53 ± 1.35 |
| `gojq`        |  30.3 ± 0.6 |     29.2 |     32.4 |  15.84 ± 0.87 |
| `jq`          |  30.9 ± 0.4 |     30.1 |     33.4 |  16.19 ± 0.86 |
| `sjq`         |  37.2 ± 0.4 |     36.6 |     38.7 |  19.47 ± 1.02 |
| `node`        |  39.3 ± 1.1 |     37.4 |     42.5 |  20.55 ± 1.19 |
| `bun`         |  40.3 ± 6.5 |     38.1 |     85.4 |  21.12 ± 3.58 |
| `json-minify` |  59.0 ± 1.5 |     57.0 |     62.5 |  30.88 ± 1.76 |
| `python`      | 112.9 ± 0.9 |    111.1 |    115.3 |  59.09 ± 3.08 |
| `jello`       | 265.7 ± 1.7 |    263.3 |    268.5 | 139.08 ± 7.21 |
