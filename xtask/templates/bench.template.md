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

{{PRETTY_CANADA_TABLE}}

#### ugly canada

![candlestick benchmark for ugly printing canada.json](/xtask/bench/output/ugly-canada.png)

{{UGLY_CANADA_TABLE}}

### citm catalog

1.7MB JSON file with lots of lightly nested, long objects. See [citm_catalog.json](https://github.com/serde-rs/json-benchmark/blob/master/data/citm_catalog.json)

#### pretty citm catalog

![candlestick benchmark for pretty printing citm-catalog.json](/xtask/bench/output/pretty-citm_catalog.png)

{{PRETTY_CITM_CATALOG_TABLE}}

#### ugly citm catalog

![candlestick benchmark for ugly printing citm-catalog.json](/xtask/bench/output/ugly-citm_catalog.png)

{{UGLY_CITM_CATALOG_TABLE}}

### twitter

.6MB JSON file with lots of lightly nested, short objects. See [twitter.json](https://github.com/serde-rs/json-benchmark/blob/master/data/twitter.json)

#### pretty twitter

![candlestick benchmark for pretty printing twitter.json](/xtask/bench/output/pretty-twitter.png)

{{PRETTY_TWITTER_TABLE}}

#### ugly twitter

![candlestick benchmark for ugly printing twitter.json](/xtask/bench/output/ugly-twitter.png)

{{UGLY_TWITTER_TABLE}}

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
