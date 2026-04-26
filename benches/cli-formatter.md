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
| `jsonxf`       |    14.0 ± 0.7 |     13.1 |     18.0 |         1.00 |
| `jsonformat`   |    15.9 ± 1.1 |     14.0 |     23.4 |  1.13 ± 0.10 |
| `jsonice`      |    21.1 ± 0.6 |     20.3 |     25.2 |  1.50 ± 0.08 |
| `json-pp-rust` |    28.5 ± 0.9 |     27.3 |     34.6 |  2.03 ± 0.12 |
| `jjp`          |    31.6 ± 2.6 |     29.2 |     44.0 |  2.25 ± 0.22 |
| `gojq`         |    56.8 ± 1.5 |     54.2 |     62.2 |  4.06 ± 0.22 |
| `jaq`          |    68.8 ± 2.3 |     65.8 |     76.1 |  4.91 ± 0.29 |
| `bun`          |    70.6 ± 2.1 |     68.1 |     77.3 |  5.04 ± 0.28 |
| `node`         |    89.1 ± 2.6 |     85.2 |     97.3 |  6.36 ± 0.35 |
| `jshon`        |   110.8 ± 5.1 |    100.2 |    122.0 |  7.91 ± 0.52 |
| `jq`           |   113.2 ± 5.7 |    109.4 |    139.2 |  8.08 ± 0.56 |
| `python`       |   276.1 ± 9.5 |    267.3 |    299.5 | 19.71 ± 1.16 |
| `jello`        |  387.9 ± 58.0 |    349.0 |    534.8 | 27.69 ± 4.35 |
| `sjq`          |  483.6 ± 21.6 |    450.7 |    518.7 | 34.53 ± 2.25 |
| `dprint`       |  670.8 ± 14.0 |    656.9 |    692.7 | 47.89 ± 2.48 |
| `prettier`     | 1141.6 ± 30.5 |   1083.6 |   1172.6 | 81.50 ± 4.43 |
| `oxfmt`        | 1230.1 ± 35.6 |   1200.9 |   1311.5 | 87.82 ± 4.88 |

### ugly canada

![candlestick benchmark for ugly printing canada.json](/xtask/bench/output/ugly-canada.png)

| Command       |    Mean [ms] | Min [ms] | Max [ms] |     Relative |
| :------------ | -----------: | -------: | -------: | -----------: |
| `jsonxf`      |   13.0 ± 1.5 |     11.7 |     21.4 |         1.00 |
| `jjp`         |   16.4 ± 2.0 |     14.9 |     27.0 |  1.27 ± 0.21 |
| `minify`      |   41.1 ± 4.6 |     37.2 |     72.6 |  3.16 ± 0.51 |
| `jaq`         |   54.3 ± 2.7 |     51.0 |     65.6 |  4.18 ± 0.53 |
| `gojq`        |   56.4 ± 3.7 |     52.1 |     69.0 |  4.34 ± 0.58 |
| `bun`         |   60.2 ± 7.6 |     57.2 |    111.4 |  4.63 ± 0.79 |
| `node`        |   73.9 ± 4.4 |     68.6 |     96.0 |  5.68 ± 0.74 |
| `jq`          |   87.1 ± 1.7 |     84.5 |     91.0 |  6.70 ± 0.79 |
| `json-minify` |   93.9 ± 2.4 |     90.8 |    100.4 |  7.22 ± 0.86 |
| `python`      | 261.2 ± 10.8 |    245.0 |    288.1 | 20.09 ± 2.48 |
| `jello`       |  348.1 ± 4.4 |    343.4 |    356.1 | 26.79 ± 3.13 |
| `sjq`         |  366.1 ± 6.6 |    359.6 |    380.2 | 28.17 ± 3.32 |

## citm catalog

[citm_catalog.json](/xtask/bench/data/json-benchmark/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

### pretty citm catalog

![candlestick benchmark for pretty printing citm-catalog.json](/xtask/bench/output/pretty-citm_catalog.png)

| Command        |     Mean [ms] | Min [ms] | Max [ms] |       Relative |
| :------------- | ------------: | -------: | -------: | -------------: |
| `jsonxf`       |     5.2 ± 0.3 |      4.8 |      7.5 |           1.00 |
| `jsonice`      |     8.6 ± 0.4 |      8.1 |     11.1 |    1.67 ± 0.12 |
| `jsonformat`   |     9.3 ± 1.7 |      8.0 |     23.8 |    1.79 ± 0.35 |
| `jjp`          |    11.9 ± 1.3 |     10.7 |     17.4 |    2.30 ± 0.28 |
| `json-pp-rust` |    17.5 ± 3.0 |     14.9 |     32.5 |    3.39 ± 0.60 |
| `jshon`        |    32.2 ± 2.3 |     30.5 |     47.0 |    6.24 ± 0.55 |
| `jaq`          |    40.8 ± 3.1 |     37.3 |     49.5 |    7.90 ± 0.74 |
| `gojq`         |    54.8 ± 6.3 |     48.2 |     88.6 |   10.61 ± 1.35 |
| `node`         |    60.2 ± 2.3 |     57.2 |     66.7 |   11.67 ± 0.77 |
| `bun`          |    61.3 ± 2.0 |     58.8 |     66.5 |   11.88 ± 0.74 |
| `jq`           |    61.3 ± 4.4 |     56.3 |     84.5 |   11.88 ± 1.05 |
| `sjq`          |  145.6 ± 13.1 |    134.5 |    181.9 |   28.21 ± 2.94 |
| `dprint`       |   159.1 ± 5.2 |    151.2 |    166.9 |   30.81 ± 1.93 |
| `python`       |   164.5 ± 2.9 |    159.6 |    171.4 |   31.86 ± 1.79 |
| `jello`        |  350.1 ± 13.7 |    338.3 |    383.1 |   67.82 ± 4.48 |
| `prettier`     | 689.6 ± 142.1 |    618.2 |   1090.0 | 133.59 ± 28.43 |
| `oxfmt`        |  805.4 ± 40.0 |    767.4 |    910.7 | 156.04 ± 11.37 |

### ugly citm catalog

![candlestick benchmark for ugly printing citm-catalog.json](/xtask/bench/output/ugly-citm_catalog.png)

| Command       |    Mean [ms] | Min [ms] | Max [ms] |     Relative |
| :------------ | -----------: | -------: | -------: | -----------: |
| `jsonxf`      |    4.7 ± 0.5 |      4.3 |      8.7 |         1.00 |
| `jjp`         |    6.7 ± 0.8 |      5.9 |     12.2 |  1.41 ± 0.22 |
| `minify`      |   33.8 ± 5.2 |     30.9 |     78.9 |  7.14 ± 1.30 |
| `jaq`         |   36.0 ± 3.4 |     33.3 |     60.2 |  7.60 ± 1.05 |
| `gojq`        |   46.1 ± 1.4 |     44.3 |     50.8 |  9.73 ± 1.01 |
| `node`        |   48.5 ± 2.2 |     45.4 |     53.5 | 10.23 ± 1.12 |
| `jq`          |   49.4 ± 1.0 |     47.8 |     53.2 | 10.42 ± 1.06 |
| `bun`         |   51.5 ± 1.9 |     49.0 |     61.7 | 10.86 ± 1.15 |
| `json-minify` |   68.5 ± 2.4 |     64.4 |     77.5 | 14.45 ± 1.53 |
| `sjq`         |  114.2 ± 3.5 |    110.2 |    128.2 | 24.11 ± 2.51 |
| `python`      | 148.4 ± 11.1 |    141.1 |    191.7 | 31.31 ± 3.90 |
| `jello`       |  346.5 ± 7.5 |    336.1 |    355.7 | 73.13 ± 7.44 |

## twitter

[twitter.json](/xtask/bench/data/json-benchmark/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

### pretty twitter

![candlestick benchmark for pretty printing twitter.json](/xtask/bench/output/pretty-twitter.png)

| Command        |    Mean [ms] | Min [ms] | Max [ms] |       Relative |
| :------------- | -----------: | -------: | -------: | -------------: |
| `jsonxf`       |    2.4 ± 0.4 |      1.8 |      5.3 |           1.00 |
| `jsonformat`   |    3.8 ± 0.3 |      3.4 |      7.7 |    1.59 ± 0.32 |
| `jsonice`      |    4.3 ± 0.3 |      4.0 |      7.1 |    1.83 ± 0.36 |
| `jjp`          |    5.8 ± 0.8 |      5.0 |      9.6 |    2.46 ± 0.56 |
| `json-pp-rust` |    6.6 ± 0.5 |      5.9 |      9.5 |    2.77 ± 0.55 |
| `jshon`        |   17.2 ± 2.8 |     13.8 |     31.6 |    7.26 ± 1.77 |
| `jaq`          |   28.6 ± 0.9 |     27.1 |     32.3 |   12.04 ± 2.24 |
| `gojq`         |   32.7 ± 1.6 |     30.4 |     40.7 |   13.78 ± 2.62 |
| `jq`           |   34.4 ± 1.6 |     32.7 |     42.6 |   14.53 ± 2.74 |
| `bun`          |   44.8 ± 5.7 |     41.8 |     83.1 |   18.89 ± 4.22 |
| `node`         |   44.8 ± 2.3 |     41.8 |     55.0 |   18.89 ± 3.60 |
| `sjq`          |   46.1 ± 2.8 |     43.7 |     60.9 |   19.44 ± 3.75 |
| `dprint`       |   60.7 ± 1.8 |     58.7 |     67.9 |   25.60 ± 4.74 |
| `python`       |  120.3 ± 3.3 |    117.3 |    129.0 |   50.72 ± 9.39 |
| `jello`        |  291.6 ± 7.2 |    280.0 |    306.2 | 123.01 ± 22.72 |
| `prettier`     | 376.0 ± 24.3 |    352.9 |    440.0 | 158.58 ± 30.79 |
| `oxfmt`        |  465.4 ± 7.2 |    453.1 |    474.3 | 196.31 ± 36.07 |

### ugly twitter

![candlestick benchmark for ugly printing twitter.json](/xtask/bench/output/ugly-twitter.png)

| Command       |    Mean [ms] | Min [ms] | Max [ms] |       Relative |
| :------------ | -----------: | -------: | -------: | -------------: |
| `jsonxf`      |    2.1 ± 0.2 |      1.8 |      3.5 |           1.00 |
| `jjp`         |    4.1 ± 0.6 |      3.6 |      8.9 |    2.00 ± 0.33 |
| `minify`      |   25.8 ± 1.4 |     24.0 |     35.7 |   12.53 ± 1.21 |
| `jaq`         |   27.3 ± 1.1 |     26.1 |     34.2 |   13.25 ± 1.18 |
| `gojq`        |   32.5 ± 1.7 |     30.7 |     45.3 |   15.80 ± 1.52 |
| `jq`          |   32.8 ± 1.2 |     31.1 |     36.7 |   15.95 ± 1.41 |
| `sjq`         |   38.7 ± 1.3 |     37.5 |     44.6 |   18.81 ± 1.63 |
| `node`        |   42.2 ± 2.0 |     39.7 |     49.6 |   20.54 ± 1.90 |
| `bun`         |   42.6 ± 1.9 |     40.5 |     53.5 |   20.73 ± 1.91 |
| `json-minify` |   63.1 ± 3.7 |     59.6 |     84.2 |   30.67 ± 3.04 |
| `python`      |  119.1 ± 3.0 |    115.2 |    127.2 |   57.89 ± 4.88 |
| `jello`       | 288.2 ± 12.8 |    276.5 |    319.2 | 140.13 ± 12.86 |
