| Command | Mean Wall Time [ms] | Change | Factor |
|:---|---:|---:|:---|
| `jjp` | 21.5 ± 0.4 |  |  |
| `json-pp-rust` | 24.9 ± 0.9 | +15.8% |  |
| `jsonice` | 22.5 ± 1.0 | +4.8% |  |
| `prettier` | 1000.4 ± 14.1 | +4554.0% | (46.5x slower) |
| `jq` | 95.4 ± 3.2 | +343.6% | (4.4x slower) |
| `gojq` | 35.6 ± 0.7 | +65.5% | (1.7x slower) |
| `jaq` | 39.2 ± 1.8 | +82.3% | (1.8x slower) |
| `sjq` | 415.5 ± 6.4 | +1832.8% | (19.3x slower) |
| `dprint` | 416.7 ± 4.0 | +1838.5% | (19.4x slower) |
| `python` | 271.9 ± 5.1 | +1165.0% | (12.6x slower) |
| `node` | 69.4 ± 1.9 | +223.1% | (3.2x slower) |
| `bun` | 24.0 ± 0.7 | +11.9% |  |
| `jsonxf` | 12.2 ± 0.2 | -43.1% | (1.8x faster) |
| `jsonformat` | 15.7 ± 0.2 | -26.8% |  |
| `oxfmt` | 93.3 ± 3.0 | +333.9% | (4.3x slower) |
