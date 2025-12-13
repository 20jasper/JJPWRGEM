<!-- GENERATED FILE - update the templates in the xtask -->

# jjpwrgem benchmarks

Is jjpwrgem blazingly fast? Yes. Is it the blazingly fastest? Depends on your definition

It can parse and pretty print a 1.7MB JSON file in around ~11ms[^prettyCitm] and the average package.json in ~500 microseconds—the fastest I could find for CLIs that validate syntax. jjpwrgem is best in its class for stringification speed, with Gigabytes of througput for minification[^throughputBench]. There's faster out there for parsing speed, but it does the job!

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

| Command      |     Mean [ms] | Min [ms] | Max [ms] |     Relative |
| :----------- | ------------: | -------: | -------: | -----------: |
| `jsonxf`     |    11.8 ± 0.6 |     11.0 |     16.8 |         1.00 |
| `jsonformat` |    14.1 ± 1.1 |     13.0 |     20.9 |  1.19 ± 0.11 |
| `jjp`        |    25.0 ± 3.2 |     22.8 |     44.1 |  2.12 ± 0.30 |
| `gojq`       |    40.3 ± 4.3 |     37.0 |     72.5 |  3.42 ± 0.41 |
| `jaq`        |    49.8 ± 4.7 |     47.0 |     70.6 |  4.23 ± 0.46 |
| `bun`        |    52.0 ± 3.0 |     49.4 |     69.4 |  4.41 ± 0.35 |
| `node`       |    62.9 ± 2.3 |     59.9 |     70.9 |  5.34 ± 0.35 |
| `jshon`      |    91.1 ± 2.4 |     88.0 |    100.0 |  7.72 ± 0.47 |
| `jq`         |  101.6 ± 12.7 |     89.6 |    135.9 |  8.61 ± 1.18 |
| `python`     |   241.6 ± 5.4 |    235.9 |    256.3 | 20.49 ± 1.21 |
| `jello`      |   307.7 ± 5.5 |    300.1 |    315.9 | 26.10 ± 1.50 |
| `sjq`        |   377.2 ± 7.5 |    371.0 |    397.6 | 31.99 ± 1.86 |
| `dprint`     |  452.8 ± 11.0 |    441.0 |    476.6 | 38.40 ± 2.30 |
| `prettier`   | 1040.5 ± 57.6 |    961.1 |   1150.2 | 88.24 ± 6.86 |

#### ugly canada

![candlestick benchmark for ugly printing canada.json](/xtask/bench/output/ugly-canada.png)

| Command       |    Mean [ms] | Min [ms] | Max [ms] |     Relative |
| :------------ | -----------: | -------: | -------: | -----------: |
| `jsonxf`      |   14.4 ± 3.9 |     11.0 |     35.4 |         1.00 |
| `jjp`         |   23.8 ± 2.7 |     22.0 |     47.3 |  1.66 ± 0.48 |
| `minify`      |   25.3 ± 1.2 |     23.4 |     31.7 |  1.76 ± 0.48 |
| `jaq`         |   36.5 ± 5.6 |     32.5 |     64.8 |  2.54 ± 0.79 |
| `gojq`        |   38.0 ± 2.1 |     35.2 |     44.2 |  2.65 ± 0.73 |
| `bun`         |   41.3 ± 1.4 |     39.8 |     49.9 |  2.87 ± 0.78 |
| `node`        |   54.5 ± 1.8 |     51.6 |     59.8 |  3.80 ± 1.03 |
| `json-minify` |   67.3 ± 2.3 |     64.1 |     73.0 |  4.69 ± 1.27 |
| `jq`          |  78.1 ± 12.9 |     66.1 |    105.4 |  5.44 ± 1.72 |
| `python`      |  228.4 ± 9.5 |    217.2 |    250.3 | 15.91 ± 4.33 |
| `sjq`         |  304.3 ± 3.8 |    299.8 |    310.3 | 21.20 ± 5.71 |
| `jello`       | 335.2 ± 37.2 |    307.6 |    425.2 | 23.35 ± 6.80 |

### citm catalog

1.7MB JSON file with lots of lightly nested, long objects. See [citm_catalog.json](https://github.com/serde-rs/json-benchmark/blob/master/data/citm_catalog.json)

#### pretty citm catalog

![candlestick benchmark for pretty printing citm-catalog.json](/xtask/bench/output/pretty-citm_catalog.png)

| Command      |    Mean [ms] | Min [ms] | Max [ms] |       Relative |
| :----------- | -----------: | -------: | -------: | -------------: |
| `jsonxf`     |    4.2 ± 0.5 |      3.9 |      8.7 |           1.00 |
| `jsonformat` |    7.1 ± 0.4 |      6.6 |     11.8 |    1.69 ± 0.20 |
| `jjp`        |    9.3 ± 0.5 |      8.5 |     13.8 |    2.21 ± 0.27 |
| `jshon`      |   24.2 ± 0.7 |     23.1 |     28.4 |    5.73 ± 0.64 |
| `jaq`        |   25.5 ± 2.5 |     23.5 |     44.1 |    6.07 ± 0.88 |
| `gojq`       |   31.0 ± 1.3 |     29.3 |     36.7 |    7.37 ± 0.85 |
| `bun`        |   39.0 ± 1.1 |     37.0 |     43.0 |    9.25 ± 1.03 |
| `node`       |   39.6 ± 1.8 |     36.7 |     43.5 |    9.40 ± 1.10 |
| `jq`         |   43.1 ± 2.2 |     40.7 |     54.6 |   10.23 ± 1.21 |
| `dprint`     |  114.6 ± 2.0 |    112.2 |    118.9 |   27.21 ± 2.96 |
| `sjq`        |  122.0 ± 1.7 |    120.1 |    126.9 |   28.97 ± 3.14 |
| `python`     |  130.4 ± 2.4 |    126.8 |    135.0 |   30.97 ± 3.37 |
| `jello`      |  294.8 ± 4.6 |    289.7 |    304.0 |   69.99 ± 7.60 |
| `prettier`   | 508.4 ± 12.3 |    494.0 |    526.8 | 120.70 ± 13.29 |

#### ugly citm catalog

![candlestick benchmark for ugly printing citm-catalog.json](/xtask/bench/output/ugly-citm_catalog.png)

| Command       |   Mean [ms] | Min [ms] | Max [ms] |     Relative |
| :------------ | ----------: | -------: | -------: | -----------: |
| `jsonxf`      |   4.2 ± 0.3 |      3.9 |      9.2 |         1.00 |
| `jjp`         |   8.5 ± 0.6 |      7.8 |     12.4 |  2.02 ± 0.21 |
| `minify`      |  18.7 ± 0.9 |     17.3 |     23.1 |  4.44 ± 0.42 |
| `jaq`         |  21.1 ± 0.6 |     20.1 |     23.6 |  5.00 ± 0.43 |
| `gojq`        |  30.8 ± 1.8 |     28.6 |     43.2 |  7.29 ± 0.73 |
| `bun`         |  34.9 ± 0.9 |     33.4 |     39.2 |  8.27 ± 0.70 |
| `node`        |  36.0 ± 2.5 |     33.1 |     50.9 |  8.52 ± 0.91 |
| `jq`          |  37.3 ± 1.4 |     35.9 |     46.4 |  8.85 ± 0.79 |
| `json-minify` |  45.5 ± 1.9 |     42.6 |     50.2 | 10.78 ± 0.99 |
| `sjq`         | 104.9 ± 1.9 |    102.5 |    112.6 | 24.85 ± 2.07 |
| `python`      | 125.6 ± 2.3 |    121.7 |    129.7 | 29.76 ± 2.48 |
| `jello`       | 295.3 ± 8.1 |    286.0 |    310.1 | 69.96 ± 6.00 |

### twitter

.6MB JSON file with lots of lightly nested, short objects. See [twitter.json](https://github.com/serde-rs/json-benchmark/blob/master/data/twitter.json)

#### pretty twitter

![candlestick benchmark for pretty printing twitter.json](/xtask/bench/output/pretty-twitter.png)

| Command      |   Mean [ms] | Min [ms] | Max [ms] |       Relative |
| :----------- | ----------: | -------: | -------: | -------------: |
| `jsonxf`     |   1.7 ± 0.1 |      1.5 |      2.6 |           1.00 |
| `jsonformat` |   3.2 ± 0.2 |      3.0 |      5.3 |    1.92 ± 0.17 |
| `jjp`        |   4.4 ± 0.2 |      4.0 |      7.1 |    2.66 ± 0.22 |
| `jshon`      |  12.0 ± 0.3 |     11.6 |     14.3 |    7.22 ± 0.52 |
| `jaq`        |  18.0 ± 0.9 |     16.8 |     25.6 |   10.83 ± 0.90 |
| `gojq`       |  21.3 ± 1.5 |     19.7 |     29.4 |   12.77 ± 1.23 |
| `jq`         |  25.2 ± 1.9 |     23.5 |     36.6 |   15.18 ± 1.51 |
| `bun`        |  29.3 ± 1.2 |     27.6 |     34.7 |   17.63 ± 1.39 |
| `node`       |  31.5 ± 2.9 |     29.4 |     49.4 |   18.93 ± 2.14 |
| `sjq`        |  43.4 ± 2.0 |     41.8 |     56.7 |   26.08 ± 2.11 |
| `dprint`     |  45.1 ± 1.7 |     42.6 |     52.6 |   27.12 ± 2.08 |
| `python`     | 101.8 ± 1.6 |     99.4 |    105.2 |   61.19 ± 4.19 |
| `jello`      | 250.4 ± 7.1 |    244.6 |    269.1 | 150.47 ± 10.89 |
| `prettier`   | 307.4 ± 7.8 |    297.4 |    317.7 | 184.76 ± 13.17 |

#### ugly twitter

![candlestick benchmark for ugly printing twitter.json](/xtask/bench/output/ugly-twitter.png)

| Command       |   Mean [ms] | Min [ms] | Max [ms] |       Relative |
| :------------ | ----------: | -------: | -------: | -------------: |
| `jsonxf`      |   1.7 ± 0.2 |      1.5 |      3.9 |           1.00 |
| `jjp`         |   4.2 ± 0.2 |      3.8 |      7.5 |    2.49 ± 0.31 |
| `minify`      |  14.6 ± 0.7 |     13.5 |     18.4 |    8.69 ± 1.06 |
| `jaq`         |  16.8 ± 0.5 |     15.8 |     18.9 |   10.02 ± 1.16 |
| `gojq`        |  20.7 ± 1.1 |     19.0 |     28.1 |   12.33 ± 1.51 |
| `jq`          |  23.3 ± 1.1 |     22.2 |     32.8 |   13.90 ± 1.67 |
| `bun`         |  28.2 ± 0.9 |     26.7 |     30.9 |   16.78 ± 1.93 |
| `node`        |  29.3 ± 1.3 |     27.7 |     37.7 |   17.45 ± 2.09 |
| `sjq`         |  38.3 ± 0.9 |     36.9 |     42.9 |   22.81 ± 2.59 |
| `json-minify` |  41.2 ± 1.7 |     39.0 |     46.0 |   24.53 ± 2.90 |
| `python`      | 101.1 ± 1.6 |     98.7 |    105.1 |   60.22 ± 6.76 |
| `jello`       | 245.7 ± 6.1 |    241.3 |    263.8 | 146.35 ± 16.65 |

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
