<!-- GENERATED FILE - update the templates in the xtask -->

# jjpwrgem benchmarks

Is jjpwrgem blazingly fast? Yes. Is it the blazingly fastest? Depends on your definition

It can parse and pretty print a 1.7MB JSON file in around ~11ms[^prettyCitm] and the average package.json in ~500 microseconds—the fastest I could find for CLIs that validate syntax. jjpwrgem is best in its class for stringification speed, with Gigabytes of througput for minification[^throughputBench]. There's faster out there for parsing speed, but it does the job!

[^prettyCitm]: #pretty-citm-catalog

[^throughputBench]: #parsing-and-stringification-throughput

Please show me any other tools I should benchmark against!

## comparison with other CLI tools

These benchmarks are available to run locally via `just bench`

note: the goal of many tools is not to format—`jq` is far better at filtering JSON than `jjp`!

note: `jsonxf` and `jsonformat` don't validate syntax

### canada

2.2MB JSON file with lots of lightly nested arrays. See [canada.json](https://github.com/serde-rs/json-benchmark/blob/master/data/canada.json)

#### pretty canada

![candlestick benchmark for pretty printing canada.json](/xtask/bench/output/pretty-canada.png)

| Command      |    Mean [ms] | Min [ms] | Max [ms] |     Relative |
| :----------- | -----------: | -------: | -------: | -----------: |
| `jsonxf`     |   11.5 ± 0.4 |     10.8 |     14.2 |         1.00 |
| `jsonformat` |   13.3 ± 0.4 |     12.9 |     16.7 |  1.16 ± 0.06 |
| `jjp`        |   26.6 ± 1.6 |     25.0 |     36.1 |  2.32 ± 0.17 |
| `gojq`       |   38.3 ± 1.8 |     35.5 |     46.0 |  3.34 ± 0.20 |
| `jaq`        |   46.2 ± 1.3 |     44.9 |     53.9 |  4.03 ± 0.19 |
| `bun`        |   46.8 ± 2.8 |     44.3 |     63.2 |  4.08 ± 0.29 |
| `node`       |   60.6 ± 3.2 |     56.5 |     71.0 |  5.29 ± 0.34 |
| `jshon`      |   87.3 ± 1.1 |     85.7 |     90.1 |  7.61 ± 0.30 |
| `jq`         |   89.1 ± 5.5 |     85.3 |    106.4 |  7.77 ± 0.56 |
| `python`     |  234.9 ± 2.6 |    230.3 |    237.9 | 20.49 ± 0.80 |
| `jello`      | 304.9 ± 14.3 |    287.9 |    325.2 | 26.60 ± 1.60 |
| `sjq`        |  339.0 ± 2.0 |    336.9 |    343.2 | 29.57 ± 1.13 |
| `dprint`     | 439.9 ± 12.0 |    420.9 |    460.3 | 38.37 ± 1.78 |
| `prettier`   | 977.4 ± 33.3 |    937.6 |   1043.1 | 85.26 ± 4.33 |

#### ugly canada

![candlestick benchmark for ugly printing canada.json](/xtask/bench/output/ugly-canada.png)

| Command       |    Mean [ms] | Min [ms] | Max [ms] |     Relative |
| :------------ | -----------: | -------: | -------: | -----------: |
| `jsonxf`      |   11.6 ± 0.3 |     10.8 |     12.4 |         1.00 |
| `jjp`         |   22.0 ± 0.7 |     20.8 |     26.2 |  1.90 ± 0.08 |
| `minify`      |   23.2 ± 0.8 |     21.7 |     25.7 |  2.00 ± 0.08 |
| `jaq`         |   31.9 ± 0.9 |     31.0 |     37.1 |  2.75 ± 0.11 |
| `gojq`        |   36.3 ± 3.1 |     33.5 |     52.7 |  3.13 ± 0.28 |
| `bun`         |   37.8 ± 2.9 |     35.5 |     51.6 |  3.26 ± 0.26 |
| `node`        |   51.2 ± 2.0 |     47.8 |     56.5 |  4.42 ± 0.21 |
| `json-minify` |   61.4 ± 2.1 |     58.9 |     70.2 |  5.29 ± 0.23 |
| `jq`          |   62.2 ± 2.1 |     60.6 |     72.8 |  5.36 ± 0.23 |
| `python`      | 221.1 ± 11.7 |    209.2 |    246.5 | 19.06 ± 1.12 |
| `sjq`         |  276.0 ± 8.3 |    266.4 |    291.7 | 23.79 ± 0.95 |
| `jello`       |  286.9 ± 5.9 |    282.9 |    301.8 | 24.73 ± 0.82 |

### citm catalog

1.7MB JSON file with lots of lightly nested, long objects. See [citm_catalog.json](https://github.com/serde-rs/json-benchmark/blob/master/data/citm_catalog.json)

#### pretty citm catalog

![candlestick benchmark for pretty printing citm-catalog.json](/xtask/bench/output/pretty-citm_catalog.png)

| Command      |    Mean [ms] | Min [ms] | Max [ms] |      Relative |
| :----------- | -----------: | -------: | -------: | ------------: |
| `jsonxf`     |    4.0 ± 0.1 |      3.9 |      5.4 |          1.00 |
| `jsonformat` |    6.9 ± 0.4 |      6.5 |     10.6 |   1.72 ± 0.11 |
| `jjp`        |   11.1 ± 0.6 |     10.2 |     14.7 |   2.73 ± 0.18 |
| `jaq`        |   22.9 ± 0.7 |     22.1 |     27.0 |   5.65 ± 0.26 |
| `jshon`      |   23.2 ± 1.1 |     22.3 |     30.0 |   5.73 ± 0.33 |
| `gojq`       |   29.3 ± 1.2 |     27.8 |     34.9 |   7.23 ± 0.39 |
| `bun`        |   35.0 ± 1.8 |     33.4 |     44.9 |   8.66 ± 0.55 |
| `node`       |   36.0 ± 1.4 |     34.2 |     40.1 |   8.90 ± 0.47 |
| `jq`         |   40.1 ± 1.2 |     39.1 |     48.3 |   9.92 ± 0.46 |
| `dprint`     |  110.7 ± 3.6 |    106.8 |    123.7 |  27.36 ± 1.32 |
| `sjq`        |  115.2 ± 1.3 |    112.8 |    118.3 |  28.47 ± 1.07 |
| `python`     |  126.4 ± 2.7 |    122.8 |    133.4 |  31.24 ± 1.29 |
| `jello`      | 284.0 ± 11.2 |    276.0 |    313.2 |  70.19 ± 3.73 |
| `prettier`   | 506.2 ± 16.4 |    488.3 |    543.3 | 125.10 ± 6.03 |

#### ugly citm catalog

![candlestick benchmark for ugly printing citm-catalog.json](/xtask/bench/output/ugly-citm_catalog.png)

| Command       |   Mean [ms] | Min [ms] | Max [ms] |     Relative |
| :------------ | ----------: | -------: | -------: | -----------: |
| `jsonxf`      |   4.1 ± 0.3 |      3.8 |      7.2 |         1.00 |
| `jjp`         |  10.2 ± 0.8 |      9.5 |     17.4 |  2.50 ± 0.26 |
| `minify`      |  18.5 ± 2.5 |     16.5 |     44.0 |  4.55 ± 0.68 |
| `jaq`         |  19.7 ± 0.6 |     18.8 |     24.4 |  4.83 ± 0.35 |
| `gojq`        |  29.3 ± 1.3 |     27.3 |     33.7 |  7.19 ± 0.57 |
| `jq`          |  34.8 ± 0.8 |     34.0 |     40.6 |  8.53 ± 0.59 |
| `node`        |  35.1 ± 3.0 |     31.0 |     49.2 |  8.62 ± 0.93 |
| `bun`         |  35.2 ± 3.9 |     30.2 |     53.4 |  8.63 ± 1.11 |
| `json-minify` |  40.7 ± 1.6 |     38.7 |     46.5 |  9.98 ± 0.75 |
| `sjq`         | 104.9 ± 4.8 |    100.0 |    120.3 | 25.72 ± 2.05 |
| `python`      | 127.0 ± 9.2 |    118.9 |    160.5 | 31.14 ± 3.04 |
| `jello`       | 284.9 ± 8.3 |    278.5 |    305.5 | 69.87 ± 4.99 |

### twitter

.6MB JSON file with lots of lightly nested, short objects. See [twitter.json](https://github.com/serde-rs/json-benchmark/blob/master/data/twitter.json)

#### pretty twitter

![candlestick benchmark for pretty printing twitter.json](/xtask/bench/output/pretty-twitter.png)

| Command      |    Mean [ms] | Min [ms] | Max [ms] |       Relative |
| :----------- | -----------: | -------: | -------: | -------------: |
| `jsonxf`     |    1.6 ± 0.1 |      1.4 |      2.9 |           1.00 |
| `jsonformat` |    3.1 ± 0.3 |      2.9 |      5.4 |    1.98 ± 0.24 |
| `jjp`        |    4.8 ± 0.4 |      4.4 |      7.2 |    3.09 ± 0.38 |
| `jshon`      |   12.0 ± 0.6 |     11.4 |     16.0 |    7.69 ± 0.77 |
| `jaq`        |   17.2 ± 0.6 |     16.4 |     19.9 |   10.99 ± 1.00 |
| `gojq`       |   19.8 ± 0.7 |     18.2 |     23.0 |   12.64 ± 1.15 |
| `jq`         |   23.6 ± 1.2 |     22.7 |     32.3 |   15.10 ± 1.48 |
| `bun`        |   26.9 ± 1.6 |     25.5 |     37.9 |   17.16 ± 1.78 |
| `node`       |   29.4 ± 1.6 |     27.9 |     41.0 |   18.78 ± 1.90 |
| `dprint`     |   43.1 ± 1.4 |     41.7 |     51.6 |   27.55 ± 2.49 |
| `sjq`        |   43.8 ± 3.4 |     41.0 |     56.8 |   27.99 ± 3.22 |
| `python`     |   99.4 ± 1.3 |     97.5 |    103.4 |   63.50 ± 5.42 |
| `jello`      |  242.9 ± 3.5 |    238.8 |    248.4 | 155.10 ± 13.27 |
| `prettier`   | 307.3 ± 13.5 |    296.1 |    336.6 | 196.25 ± 18.67 |

#### ugly twitter

![candlestick benchmark for ugly printing twitter.json](/xtask/bench/output/ugly-twitter.png)

| Command       |   Mean [ms] | Min [ms] | Max [ms] |      Relative |
| :------------ | ----------: | -------: | -------: | ------------: |
| `jsonxf`      |   1.6 ± 0.1 |      1.5 |      2.8 |          1.00 |
| `jjp`         |   4.4 ± 0.5 |      4.1 |      9.9 |   2.76 ± 0.33 |
| `minify`      |  13.5 ± 1.0 |     12.5 |     20.2 |   8.42 ± 0.80 |
| `jaq`         |  16.2 ± 0.8 |     15.1 |     20.4 |  10.06 ± 0.81 |
| `gojq`        |  19.4 ± 0.4 |     18.3 |     21.0 |  12.07 ± 0.80 |
| `jq`          |  22.1 ± 0.4 |     21.3 |     23.5 |  13.74 ± 0.89 |
| `bun`         |  25.8 ± 1.3 |     24.4 |     33.8 |  16.06 ± 1.30 |
| `node`        |  29.1 ± 4.2 |     26.4 |     60.5 |  18.09 ± 2.83 |
| `sjq`         |  38.0 ± 1.1 |     36.8 |     42.6 |  23.65 ± 1.62 |
| `json-minify` |  38.1 ± 1.3 |     36.5 |     42.2 |  23.67 ± 1.67 |
| `python`      | 101.9 ± 6.4 |     97.0 |    122.8 |  63.34 ± 5.59 |
| `jello`       | 241.3 ± 5.1 |    237.4 |    256.0 | 150.02 ± 9.88 |

## parsing and stringification throughput

jjpwrgem consistently had the highest stringification speed for twitter.json and citg catalog and canada.json, but this was the most pronounced difference

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
