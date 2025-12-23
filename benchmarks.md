<!-- GENERATED FILE - update the templates in the xtask -->

# jjpwrgem benchmarks

Is jjpwrgem blazingly fast? Yes. Is it the blazingly fastest? Depends on your definition

It can parse and pretty print a 1.7MB JSON file in around ~11ms[^prettyCitm] and the average package.json in ~500 microseconds. Also see [the throughput benchmarks](^throughputBench)

[^prettyCitm]: #pretty-citm-catalog

[^throughputBench]: #parsing-and-stringification-throughput

These benchmarks are run with `AMD Ryzen 5 5600X 6-Core Processor (3.70 GHz)`

Please show me any other tools I should benchmark against!

## comparison with other CLI tools

These benchmarks are available to run locally via `just bench`

note: the goal of many tools is not to format—`jq` is far better at filtering JSON than `jjp`!

note: `jsonxf` and `jsonformat` don't validate syntax

### canada

2.2MB JSON file with lots of lightly nested arrays. See [canada.json](https://github.com/serde-rs/json-benchmark/blob/master/data/canada.json)

#### pretty canada

![candlestick benchmark for pretty printing canada.json](/xtask/bench/output/pretty-canada.png)

| Command        |     Mean [ms] | Min [ms] | Max [ms] |     Relative |
| :------------- | ------------: | -------: | -------: | -----------: |
| `jsonxf`       |    12.0 ± 0.3 |     11.6 |     15.4 |         1.00 |
| `jsonformat`   |    14.0 ± 0.2 |     13.6 |     15.0 |  1.16 ± 0.03 |
| `jsonice`      |    21.1 ± 2.3 |     19.2 |     29.8 |  1.76 ± 0.20 |
| `json-pp-rust` |    28.6 ± 1.5 |     27.4 |     39.0 |  2.38 ± 0.13 |
| `jjp`          |    30.1 ± 0.4 |     29.4 |     32.4 |  2.51 ± 0.07 |
| `gojq`         |    52.2 ± 0.9 |     50.9 |     55.6 |  4.35 ± 0.12 |
| `jaq`          |    56.0 ± 4.4 |     52.0 |     72.0 |  4.66 ± 0.38 |
| `bun`          |    70.2 ± 1.5 |     66.7 |     75.4 |  5.85 ± 0.18 |
| `node`         |    89.4 ± 5.1 |     81.4 |    102.9 |  7.45 ± 0.46 |
| `jshon`        |   104.7 ± 7.9 |     96.9 |    124.2 |  8.72 ± 0.68 |
| `jq`           |   107.3 ± 1.2 |    105.6 |    109.9 |  8.93 ± 0.22 |
| `python`       |  286.9 ± 24.0 |    247.3 |    327.6 | 23.89 ± 2.06 |
| `jello`        |  343.2 ± 30.7 |    317.1 |    407.5 | 28.58 ± 2.63 |
| `sjq`          |  480.9 ± 31.3 |    448.4 |    532.6 | 40.04 ± 2.75 |
| `dprint`       |  667.8 ± 22.8 |    642.4 |    712.3 | 55.61 ± 2.25 |
| `prettier`     | 1154.8 ± 41.3 |   1108.1 |   1227.5 | 96.17 ± 4.02 |

#### ugly canada

![candlestick benchmark for ugly printing canada.json](/xtask/bench/output/ugly-canada.png)

| Command       |   Mean [ms] | Min [ms] | Max [ms] |     Relative |
| :------------ | ----------: | -------: | -------: | -----------: |
| `jsonxf`      |  12.1 ± 0.2 |     11.4 |     12.8 |         1.00 |
| `jjp`         |  18.5 ± 0.4 |     18.1 |     21.1 |  1.53 ± 0.04 |
| `jaq`         |  38.7 ± 0.4 |     37.8 |     40.0 |  3.20 ± 0.06 |
| `minify`      |  40.0 ± 1.3 |     37.7 |     44.2 |  3.31 ± 0.12 |
| `gojq`        |  49.4 ± 0.8 |     48.1 |     53.1 |  4.09 ± 0.09 |
| `bun`         |  54.5 ± 0.5 |     53.5 |     55.8 |  4.51 ± 0.08 |
| `node`        |  67.3 ± 1.5 |     65.1 |     70.1 |  5.57 ± 0.15 |
| `jq`          |  80.6 ± 0.4 |     79.5 |     81.7 |  6.68 ± 0.11 |
| `json-minify` |  85.4 ± 0.6 |     84.3 |     86.7 |  7.07 ± 0.12 |
| `python`      | 229.2 ± 1.4 |    227.6 |    232.6 | 18.97 ± 0.31 |
| `jello`       | 308.7 ± 1.5 |    307.6 |    312.4 | 25.55 ± 0.41 |
| `sjq`         | 357.6 ± 1.5 |    355.1 |    359.7 | 29.60 ± 0.47 |

### citm catalog

1.7MB JSON file with lots of lightly nested, long objects. See [citm_catalog.json](https://github.com/serde-rs/json-benchmark/blob/master/data/citm_catalog.json)

#### pretty citm catalog

![candlestick benchmark for pretty printing citm-catalog.json](/xtask/bench/output/pretty-citm_catalog.png)

| Command        |   Mean [ms] | Min [ms] | Max [ms] |      Relative |
| :------------- | ----------: | -------: | -------: | ------------: |
| `jsonxf`       |   4.5 ± 0.1 |      4.3 |      5.0 |          1.00 |
| `jsonice`      |   7.2 ± 0.1 |      6.9 |      7.9 |   1.59 ± 0.04 |
| `jsonformat`   |   7.4 ± 0.2 |      7.0 |      9.2 |   1.64 ± 0.06 |
| `jjp`          |  13.7 ± 0.2 |     13.3 |     15.1 |   3.02 ± 0.08 |
| `json-pp-rust` |  14.3 ± 0.4 |     13.9 |     16.2 |   3.16 ± 0.11 |
| `jshon`        |  27.4 ± 0.3 |     26.9 |     28.2 |   6.06 ± 0.15 |
| `jaq`          |  29.1 ± 0.5 |     28.1 |     31.0 |   6.43 ± 0.18 |
| `gojq`         |  41.1 ± 0.4 |     40.2 |     42.5 |   9.10 ± 0.22 |
| `jq`           |  50.1 ± 0.5 |     49.2 |     52.4 |  11.09 ± 0.27 |
| `node`         |  51.5 ± 1.3 |     49.5 |     55.3 |  11.39 ± 0.39 |
| `bun`          |  53.2 ± 0.6 |     51.9 |     54.6 |  11.77 ± 0.29 |
| `sjq`          | 132.9 ± 0.7 |    131.9 |    133.9 |  29.40 ± 0.66 |
| `python`       | 140.9 ± 1.1 |    139.3 |    143.1 |  31.17 ± 0.73 |
| `dprint`       | 144.7 ± 1.4 |    142.3 |    148.2 |  32.01 ± 0.77 |
| `jello`        | 309.6 ± 1.1 |    308.0 |    311.7 |  68.48 ± 1.53 |
| `prettier`     | 554.8 ± 3.9 |    549.3 |    562.3 | 122.72 ± 2.83 |

#### ugly citm catalog

![candlestick benchmark for ugly printing citm-catalog.json](/xtask/bench/output/ugly-citm_catalog.png)

| Command       |   Mean [ms] | Min [ms] | Max [ms] |     Relative |
| :------------ | ----------: | -------: | -------: | -----------: |
| `jsonxf`      |   4.5 ± 0.1 |      4.3 |      5.2 |         1.00 |
| `jjp`         |   9.7 ± 0.3 |      9.4 |     12.5 |  2.15 ± 0.09 |
| `jaq`         |  25.9 ± 0.7 |     25.0 |     31.3 |  5.73 ± 0.22 |
| `minify`      |  30.1 ± 1.0 |     28.5 |     33.8 |  6.65 ± 0.27 |
| `gojq`        |  40.7 ± 0.5 |     39.9 |     42.6 |  9.01 ± 0.26 |
| `node`        |  45.0 ± 1.6 |     43.0 |     48.3 |  9.95 ± 0.43 |
| `jq`          |  45.1 ± 1.3 |     44.1 |     54.8 |  9.98 ± 0.39 |
| `bun`         |  47.8 ± 0.6 |     46.5 |     50.0 | 10.57 ± 0.30 |
| `json-minify` |  60.2 ± 1.5 |     58.4 |     64.7 | 13.31 ± 0.47 |
| `sjq`         | 113.0 ± 0.8 |    111.5 |    115.0 | 24.98 ± 0.66 |
| `python`      | 135.1 ± 1.0 |    133.8 |    136.8 | 29.88 ± 0.79 |
| `jello`       | 306.8 ± 1.2 |    304.5 |    308.6 | 67.83 ± 1.74 |

### twitter

.6MB JSON file with lots of lightly nested, short objects. See [twitter.json](https://github.com/serde-rs/json-benchmark/blob/master/data/twitter.json)

#### pretty twitter

![candlestick benchmark for pretty printing twitter.json](/xtask/bench/output/pretty-twitter.png)

| Command        |   Mean [ms] | Min [ms] | Max [ms] |      Relative |
| :------------- | ----------: | -------: | -------: | ------------: |
| `jsonxf`       |   2.1 ± 0.1 |      1.9 |      2.7 |          1.00 |
| `jsonformat`   |   3.6 ± 0.1 |      3.4 |      4.1 |   1.73 ± 0.09 |
| `jsonice`      |   3.8 ± 0.1 |      3.6 |      5.4 |   1.85 ± 0.10 |
| `jjp`          |   6.0 ± 0.1 |      5.7 |      6.8 |   2.91 ± 0.14 |
| `json-pp-rust` |   6.0 ± 0.2 |      5.7 |      7.1 |   2.92 ± 0.16 |
| `jshon`        |  13.9 ± 0.2 |     13.6 |     14.6 |   6.76 ± 0.31 |
| `jaq`          |  23.0 ± 0.5 |     22.3 |     25.9 |  11.15 ± 0.54 |
| `gojq`         |  28.4 ± 0.5 |     27.4 |     30.1 |  13.79 ± 0.65 |
| `jq`           |  30.3 ± 0.4 |     29.5 |     32.9 |  14.69 ± 0.68 |
| `bun`          |  39.9 ± 0.5 |     38.8 |     42.1 |  19.34 ± 0.89 |
| `node`         |  40.0 ± 0.9 |     39.0 |     44.4 |  19.40 ± 0.96 |
| `sjq`          |  46.2 ± 0.4 |     45.2 |     47.2 |  22.39 ± 1.02 |
| `dprint`       |  57.5 ± 0.9 |     56.3 |     60.7 |  27.88 ± 1.31 |
| `python`       | 112.2 ± 1.2 |    110.3 |    114.8 |  54.42 ± 2.48 |
| `jello`        | 257.6 ± 0.8 |    256.8 |    259.4 | 124.98 ± 5.55 |
| `prettier`     | 336.4 ± 2.3 |    333.1 |    339.2 | 163.20 ± 7.32 |

#### ugly twitter

![candlestick benchmark for ugly printing twitter.json](/xtask/bench/output/ugly-twitter.png)

| Command       |   Mean [ms] | Min [ms] | Max [ms] |      Relative |
| :------------ | ----------: | -------: | -------: | ------------: |
| `jsonxf`      |   2.1 ± 0.1 |      1.9 |      2.7 |          1.00 |
| `jjp`         |   4.9 ± 0.1 |      4.7 |      5.5 |   2.34 ± 0.11 |
| `jaq`         |  22.0 ± 0.4 |     21.3 |     23.9 |  10.59 ± 0.47 |
| `minify`      |  22.7 ± 0.4 |     22.0 |     24.0 |  10.93 ± 0.49 |
| `gojq`        |  28.3 ± 0.5 |     27.5 |     30.1 |  13.63 ± 0.60 |
| `jq`          |  28.9 ± 0.4 |     28.2 |     30.7 |  13.92 ± 0.59 |
| `node`        |  37.6 ± 0.8 |     36.7 |     40.8 |  18.08 ± 0.83 |
| `bun`         |  38.7 ± 0.6 |     37.3 |     40.5 |  18.63 ± 0.82 |
| `sjq`         |  40.3 ± 0.4 |     39.6 |     41.9 |  19.41 ± 0.82 |
| `json-minify` |  55.6 ± 1.5 |     53.9 |     59.8 |  26.76 ± 1.30 |
| `python`      | 111.5 ± 0.9 |    110.5 |    114.2 |  53.67 ± 2.23 |
| `jello`       | 256.5 ± 1.0 |    255.1 |    258.6 | 123.50 ± 5.06 |

## parsing and stringification throughput

Note that these are benches for String to AST and AST to String. As of 0.5.4, jjpwrgem skips AST creation for many operations for a ~50% speedup

jjpwrgem consistently had the highest stringification speed for twitter.json and citg catalog and canada.json, but of course unless you are going from AST to String, the parsing speed holds it back

### ugly canada throughput

See [canada.json](https://github.com/serde-rs/json-benchmark/blob/master/data/canada.json)

![Bar chart comparing the average parse and stringify speeds (in MB/s) of four JSON libraries for canada.json: serde_json1.0.228, rustc_serialize0.3.25, simd-json0.17.0, and jjpwrgem0.3.3. The blue bars represent the average parse speed, while the red bars represent the average stringify speed. jjpwrgem0.3.3 has the fastest stringify speed but the lowest parse speed](/xtask/bench/output/ugly-json-canada-throughput.png)

| name                  | Parse Average MB/s | Stringify Average MB/s |
| --------------------- | -----------------: | ---------------------: |
| serde_json1.0.228     |             435.56 |                 601.11 |
| rustc_serialize0.3.25 |             230.00 |                 128.89 |
| simd-json0.17.0       |             483.33 |                 635.56 |
| jjpwrgem0.3.3         |             158.89 |               2,678.89 |

### ugly twitter throughput

See [twitter.json](https://github.com/serde-rs/json-benchmark/blob/master/data/twitter.json)

![Bar chart comparing the average parse and stringify speeds (in MB/s) of four JSON libraries for twitter.json: serde_json1.0.228, rustc_serialize0.3.25, simd-json0.17.0, and jjpwrgem0.3.3. The blue bars represent the average parse speed, while the red bars represent the average stringify speed. jjpwrgem0.3.3 has the fastest stringify speed but the second lowest parse speed](/xtask/bench/output/ugly-json-twitter-throughput.png)

| name                  | Parse Average MB/s | Stringify Average MB/s |
| --------------------- | -----------------: | ---------------------: |
| serde_json1.0.228     |             520.00 |               1,425.00 |
| rustc_serialize0.3.25 |             190.00 |                 511.00 |
| simd-json0.17.0       |           1,236.00 |               1,383.00 |
| jjpwrgem0.3.3         |             271.00 |               3,542.00 |

### ugly citm throughput

See [citm_catalog.json](https://github.com/serde-rs/json-benchmark/blob/master/data/citm_catalog.json)

![Bar chart comparing the average parse and stringify speeds (in MB/s) of four JSON libraries for citm.json: serde_json1.0.228, rustc_serialize0.3.25, simd-json0.17.0, and jjpwrgem0.3.3. The blue bars represent the average parse speed, while the red bars represent the average stringify speed. jjpwrgem0.3.3 has the fastest stringify speed but a close lowest parse speed](/xtask/bench/output/ugly-json-citm-throughput.png)

| name                  | Parse Average MB/s | Stringify Average MB/s |
| --------------------- | -----------------: | ---------------------: |
| serde_json1.0.228     |             683.33 |               1,056.67 |
| rustc_serialize0.3.25 |             383.33 |                 288.89 |
| simd-json0.17.0       |           1,272.22 |               1,143.33 |
| jjpwrgem0.3.3         |             280.00 |               2,007.78 |
