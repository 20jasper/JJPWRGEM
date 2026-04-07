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
| `jsonxf`       |    12.6 ± 1.1 |     11.7 |     20.2 |         1.00 |
| `jsonformat`   |    14.0 ± 0.4 |     13.3 |     17.0 |  1.12 ± 0.10 |
| `jsonice`      |    20.9 ± 1.3 |     19.9 |     27.5 |  1.66 ± 0.18 |
| `json-pp-rust` |    27.2 ± 1.0 |     25.6 |     34.9 |  2.17 ± 0.20 |
| `jjp`          |    31.1 ± 2.4 |     28.3 |     41.7 |  2.47 ± 0.29 |
| `gojq`         |    55.6 ± 1.1 |     54.0 |     58.8 |  4.43 ± 0.39 |
| `jaq`          |    73.1 ± 3.1 |     66.6 |     85.2 |  5.82 ± 0.55 |
| `bun`          |    73.9 ± 2.5 |     72.5 |     87.4 |  5.88 ± 0.54 |
| `node`         |    93.6 ± 4.7 |     87.2 |    107.1 |  7.45 ± 0.73 |
| `jshon`        |    99.9 ± 0.9 |     98.8 |    103.0 |  7.95 ± 0.68 |
| `jq`           |   112.3 ± 3.3 |    108.5 |    121.8 |  8.94 ± 0.80 |
| `python`       |   278.9 ± 2.2 |    275.9 |    282.7 | 22.19 ± 1.89 |
| `jello`        |   332.2 ± 3.3 |    326.3 |    335.4 | 26.43 ± 2.26 |
| `sjq`          |  491.5 ± 12.4 |    476.5 |    521.9 | 39.11 ± 3.46 |
| `dprint`       |  728.7 ± 19.3 |    718.7 |    781.7 | 57.99 ± 5.16 |
| `prettier`     | 1134.7 ± 31.3 |   1083.5 |   1202.6 | 90.29 ± 8.06 |
| `oxfmt`        | 1200.7 ± 28.7 |   1174.8 |   1261.3 | 95.54 ± 8.42 |

### ugly canada

![candlestick benchmark for ugly printing canada.json](/xtask/bench/output/ugly-canada.png)

| Command       |    Mean [ms] | Min [ms] | Max [ms] |     Relative |
| :------------ | -----------: | -------: | -------: | -----------: |
| `jsonxf`      |   12.2 ± 0.6 |     11.3 |     17.0 |         1.00 |
| `jjp`         |   17.6 ± 0.7 |     16.8 |     23.7 |  1.44 ± 0.09 |
| `minify`      |   38.1 ± 1.3 |     36.6 |     42.6 |  3.11 ± 0.19 |
| `jaq`         |   52.4 ± 2.8 |     50.5 |     67.4 |  4.28 ± 0.31 |
| `gojq`        |   54.3 ± 1.5 |     52.0 |     58.5 |  4.44 ± 0.25 |
| `bun`         |   60.0 ± 7.5 |     56.6 |    106.8 |  4.90 ± 0.66 |
| `node`        |   66.9 ± 1.7 |     64.5 |     70.9 |  5.47 ± 0.31 |
| `jq`          |   86.5 ± 4.5 |     84.2 |    110.9 |  7.06 ± 0.51 |
| `json-minify` |   91.1 ± 1.6 |     86.8 |     96.4 |  7.44 ± 0.39 |
| `python`      | 242.1 ± 10.9 |    231.7 |    273.3 | 19.77 ± 1.33 |
| `jello`       | 328.8 ± 25.9 |    311.4 |    398.7 | 26.85 ± 2.51 |
| `sjq`         |  355.5 ± 5.6 |    348.0 |    361.2 | 29.03 ± 1.52 |

## citm catalog

[citm_catalog.json](/xtask/bench/data/json-benchmark/data/citm_catalog.json) — 1.7MB, lots of lightly nested long objects, ASCII strings

### pretty citm catalog

![candlestick benchmark for pretty printing citm-catalog.json](/xtask/bench/output/pretty-citm_catalog.png)

| Command        |    Mean [ms] | Min [ms] | Max [ms] |       Relative |
| :------------- | -----------: | -------: | -------: | -------------: |
| `jsonxf`       |    4.6 ± 0.3 |      4.3 |      7.9 |           1.00 |
| `jsonformat`   |    7.7 ± 0.2 |      7.3 |      9.4 |    1.68 ± 0.10 |
| `jsonice`      |    8.5 ± 0.5 |      8.0 |     14.3 |    1.85 ± 0.15 |
| `jjp`          |   11.9 ± 0.9 |     11.3 |     21.1 |    2.61 ± 0.23 |
| `json-pp-rust` |   14.7 ± 0.6 |     14.0 |     17.3 |    3.22 ± 0.22 |
| `jshon`        |   28.5 ± 1.7 |     26.8 |     37.7 |    6.23 ± 0.50 |
| `jaq`          |   39.3 ± 6.5 |     36.7 |     84.5 |    8.58 ± 1.50 |
| `gojq`         |   45.2 ± 0.6 |     44.1 |     47.6 |    9.87 ± 0.56 |
| `node`         |   53.3 ± 1.9 |     50.7 |     58.5 |   11.64 ± 0.76 |
| `jq`           |   54.2 ± 1.3 |     52.2 |     61.3 |   11.83 ± 0.71 |
| `bun`          |   55.5 ± 1.0 |     54.4 |     60.7 |   12.13 ± 0.71 |
| `sjq`          |  131.6 ± 1.2 |    128.9 |    134.0 |   28.76 ± 1.60 |
| `python`       | 149.6 ± 11.4 |    142.5 |    188.9 |   32.69 ± 3.07 |
| `dprint`       |  150.2 ± 1.7 |    147.6 |    153.1 |   32.82 ± 1.84 |
| `jello`        |  325.4 ± 4.2 |    320.5 |    333.3 |   71.10 ± 4.02 |
| `prettier`     | 609.7 ± 15.9 |    595.8 |    640.8 |  133.21 ± 8.11 |
| `oxfmt`        | 730.9 ± 29.9 |    713.3 |    814.2 | 159.71 ± 10.94 |

### ugly citm catalog

![candlestick benchmark for ugly printing citm-catalog.json](/xtask/bench/output/ugly-citm_catalog.png)

| Command       |   Mean [ms] | Min [ms] | Max [ms] |     Relative |
| :------------ | ----------: | -------: | -------: | -----------: |
| `jsonxf`      |   4.6 ± 0.4 |      4.3 |      9.0 |         1.00 |
| `jjp`         |   7.3 ± 0.5 |      7.0 |     13.2 |  1.59 ± 0.17 |
| `minify`      |  32.1 ± 0.6 |     31.0 |     34.2 |  6.96 ± 0.62 |
| `jaq`         |  33.7 ± 0.7 |     32.5 |     36.7 |  7.31 ± 0.65 |
| `gojq`        |  44.6 ± 0.6 |     43.7 |     46.2 |  9.67 ± 0.85 |
| `node`        |  47.0 ± 3.0 |     43.1 |     55.8 | 10.18 ± 1.09 |
| `jq`          |  49.3 ± 1.8 |     47.6 |     56.3 | 10.69 ± 1.00 |
| `bun`         |  49.7 ± 1.0 |     48.5 |     55.2 | 10.78 ± 0.96 |
| `json-minify` |  66.7 ± 2.3 |     63.7 |     72.5 | 14.47 ± 1.34 |
| `sjq`         | 112.6 ± 5.4 |    108.4 |    133.5 | 24.42 ± 2.41 |
| `python`      | 140.3 ± 2.8 |    136.1 |    149.1 | 30.40 ± 2.70 |
| `jello`       | 321.2 ± 0.9 |    319.4 |    322.3 | 69.63 ± 6.02 |

## twitter

[twitter.json](/xtask/bench/data/json-benchmark/data/twitter.json) — 0.6MB, lots of lightly nested short objects, multibyte strings

### pretty twitter

![candlestick benchmark for pretty printing twitter.json](/xtask/bench/output/pretty-twitter.png)

| Command        |    Mean [ms] | Min [ms] | Max [ms] |       Relative |
| :------------- | -----------: | -------: | -------: | -------------: |
| `jsonxf`       |    2.0 ± 0.1 |      1.8 |      3.2 |           1.00 |
| `jsonformat`   |    3.6 ± 0.2 |      3.4 |      5.8 |    1.79 ± 0.15 |
| `jsonice`      |    4.2 ± 0.2 |      4.0 |      5.8 |    2.09 ± 0.17 |
| `jjp`          |    6.0 ± 0.3 |      5.7 |      9.3 |    2.95 ± 0.23 |
| `json-pp-rust` |    6.3 ± 0.5 |      5.8 |     11.0 |    3.10 ± 0.31 |
| `jshon`        |   14.4 ± 1.1 |     13.7 |     24.3 |    7.10 ± 0.73 |
| `jaq`          |   29.2 ± 5.2 |     27.2 |     79.6 |   14.36 ± 2.72 |
| `gojq`         |   32.0 ± 0.7 |     30.6 |     34.4 |   15.78 ± 1.12 |
| `jq`           |   34.1 ± 0.7 |     33.0 |     37.5 |   16.80 ± 1.19 |
| `bun`          |   41.7 ± 0.6 |     40.8 |     43.5 |   20.55 ± 1.42 |
| `node`         |   42.7 ± 2.5 |     40.3 |     56.4 |   21.05 ± 1.87 |
| `sjq`          |   43.9 ± 0.6 |     43.3 |     47.1 |   21.62 ± 1.48 |
| `dprint`       |   58.9 ± 0.8 |     57.7 |     61.0 |   29.00 ± 2.00 |
| `python`       |  119.8 ± 7.3 |    114.1 |    143.2 |   59.00 ± 5.37 |
| `jello`        |  271.0 ± 3.5 |    267.3 |    279.1 |  133.45 ± 9.16 |
| `prettier`     | 361.7 ± 19.2 |    346.7 |    411.6 | 178.15 ± 15.28 |
| `oxfmt`        | 454.4 ± 15.4 |    440.4 |    494.6 | 223.79 ± 16.88 |

### ugly twitter

![candlestick benchmark for ugly printing twitter.json](/xtask/bench/output/ugly-twitter.png)

| Command       |    Mean [ms] | Min [ms] | Max [ms] |      Relative |
| :------------ | -----------: | -------: | -------: | ------------: |
| `jsonxf`      |    2.0 ± 0.1 |      1.8 |      2.8 |          1.00 |
| `jjp`         |    5.0 ± 0.2 |      4.4 |      6.1 |   2.48 ± 0.17 |
| `minify`      |   26.0 ± 0.9 |     24.7 |     30.0 |  12.83 ± 0.81 |
| `jaq`         |   27.9 ± 4.9 |     26.1 |     72.4 |  13.80 ± 2.52 |
| `gojq`        |   31.9 ± 1.0 |     30.5 |     37.2 |  15.74 ± 0.97 |
| `jq`          |   32.5 ± 0.6 |     31.8 |     35.1 |  16.07 ± 0.90 |
| `sjq`         |   37.9 ± 0.5 |     37.1 |     39.4 |  18.69 ± 1.03 |
| `node`        |   40.2 ± 1.7 |     38.0 |     47.8 |  19.86 ± 1.35 |
| `bun`         |   41.5 ± 4.4 |     39.4 |     74.8 |  20.50 ± 2.43 |
| `json-minify` |   65.0 ± 1.3 |     62.5 |     67.9 |  32.12 ± 1.82 |
| `python`      |  118.2 ± 5.9 |    113.9 |    139.5 |  58.35 ± 4.26 |
| `jello`       | 287.6 ± 10.5 |    267.2 |    299.1 | 141.99 ± 9.17 |
