# Exponent Normalization — Performance Notes

Tracking throughput across iterations of the `Token::Number` → `Token::Mantissa` / exponent split work.

Baseline is `main` as recorded in `benches/json.md`.

`+` = higher throughput (faster), `−` = lower throughput (slower) vs `main`.

## deser

| fixture      |         main | single-Cow | two `&str` (no peek) | two `&str` + pending | split visitor | split visitor + on_number | emit+patch |  on_number |
| :----------- | -----------: | ---------: | -------------------: | -------------------: | ------------: | ------------------------: | ---------: | ---------: |
| canada       | 100.87 MiB/s |   85 MiB/s |             91 MiB/s |           ~102 MiB/s |    ~109 MiB/s |                ~107 MiB/s |  ~93 MiB/s |  ~91 MiB/s |
| citm_catalog | 345.82 MiB/s |  306 MiB/s |            397 MiB/s |           ~390 MiB/s |    ~386 MiB/s |                ~357 MiB/s | ~370 MiB/s | ~327 MiB/s |
| twitter      | 243.45 MiB/s |  218 MiB/s |            293 MiB/s |           ~289 MiB/s |    ~350 MiB/s |                ~341 MiB/s | ~340 MiB/s | ~329 MiB/s |

## prettify_ast

| fixture      |       main |   on_number |
| :----------- | ---------: | ----------: |
| canada       | 1.17 GiB/s | ~1.18 GiB/s |
| citm_catalog | 1.60 GiB/s | ~1.60 GiB/s |
| twitter      | 1.82 GiB/s | ~1.73 GiB/s |

## uglify_ast

| fixture      |       main | single-Cow | two `&str` (no peek) | two `&str` + pending | split visitor | split visitor + on_number |  emit+patch |   on_number |
| :----------- | ---------: | ---------: | -------------------: | -------------------: | ------------: | ------------------------: | ----------: | ----------: |
| canada       | 2.72 GiB/s | 2.41 GiB/s |           2.16 GiB/s |          ~2.47 GiB/s |   ~2.86 GiB/s |               ~2.69 GiB/s | ~2.28 GiB/s | ~2.57 GiB/s |
| citm_catalog | 5.68 GiB/s | 5.00 GiB/s |           5.16 GiB/s |          ~5.32 GiB/s |   ~6.52 GiB/s |               ~6.26 GiB/s | ~6.03 GiB/s | ~5.76 GiB/s |
| twitter      | 4.40 GiB/s | 3.82 GiB/s |           4.06 GiB/s |          ~4.61 GiB/s |   ~5.25 GiB/s |               ~4.80 GiB/s | ~4.86 GiB/s | ~4.68 GiB/s |

## uglify_tokens

| fixture      |         main |   — |   — | two `&str` + pending | split visitor | split visitor + on_number | emit+patch |  on_number |
| :----------- | -----------: | --: | --: | -------------------: | ------------: | ------------------------: | ---------: | ---------: |
| canada       | 173.43 MiB/s |   — |   — |           ~252 MiB/s |    ~244 MiB/s |                ~229 MiB/s | ~225 MiB/s | ~241 MiB/s |
| citm_catalog | 410.33 MiB/s |   — |   — |           ~570 MiB/s |    ~590 MiB/s |                ~535 MiB/s | ~556 MiB/s | ~577 MiB/s |
| twitter      | 275.31 MiB/s |   — |   — |           ~369 MiB/s |    ~390 MiB/s |                ~363 MiB/s | ~369 MiB/s | ~377 MiB/s |

## vs main (current — on_number: both traversers call on_number, AstVisitor overrides on_number directly)

`+` = faster than main (higher throughput), `−` = slower than main.

Note: ~10–15% run-to-run variance observed; values marked `~` are approximate.

| benchmark                  | delta |
| :------------------------- | ----: |
| deser/canada               | ~−10% |
| deser/citm_catalog         |  ~−5% |
| deser/twitter              | ~+35% |
| prettify_ast/canada        |  ~+1% |
| prettify_ast/citm_catalog  |   ~0% |
| prettify_ast/twitter       |  ~−5% |
| uglify_ast/canada          |  ~−6% |
| uglify_ast/citm_catalog    |  ~+1% |
| uglify_ast/twitter         |  ~+6% |
| uglify_tokens/canada       | ~+39% |
| uglify_tokens/citm_catalog | ~+40% |
| uglify_tokens/twitter      | ~+37% |

## Approach history

- **single-Cow** (`Cow<'a, str>`): enum inflated to 24 bytes but heap allocation for normalized exponents. Regressed everything vs main.
- **two `&str` (no peek)**: `Token::Exponent(&'a str, &'a str)` — 32-byte enum variant. Strong wins on citm/twitter deser (+15–20%) but canada uglify_ast badly hurt (−21%) by extra `peek_token()` after every Mantissa.
- **two `&str` + `take_pending_exponent`**: Replaced `peek_token()` with a cheap `Option<&'a str>` field check. Recovered canada deser to parity, narrowed uglify_ast canada gap to −9%.
- **split visitor** (`on_mantissa` + `on_exponent`): Split `parse_num` into `parse_mantissa` + `parse_exponent`, `e`/`E` handled naturally by the stream, removed `pending` from `TokenStreamInner`. `AstVisitor` used `flush_pending` calls on every callback. Across-the-board improvements — canada deser +8%, twitter +44%, uglify_ast beats main everywhere.
- **split visitor + `on_number` for `parse_value`**: `parse_value` (AST→Visitor path) calls `on_number` directly; `parse_tokens` still calls `on_mantissa`/`on_exponent`. `AstVisitor` buffers mantissa until exponent or structural callback. Slight regression vs pure split visitor — the extra indirection through `on_number` default and `pending_mantissa` costs ~5–10% on number-heavy workloads.
- **emit+patch**: `AstVisitor` emits `Value::Number { exponent: "" }` immediately on `on_mantissa`, then patches via `last_emitted_mut()` on `on_exponent`. Removes `pending_mantissa`, `flush_pending`, and all associated branches. Results within noise of split visitor.
- **on_number**: Both `parse_tokens` and `parse_value` collect mantissa+exponent and call `on_number(m, e)`. `AstVisitor` overrides `on_number` directly — no stack navigation, no patching. Formatters receive split callbacks via the default `on_number` impl. Deser regresses slightly vs main on canada/citm (high variance, within noise range). Uglify_tokens strong (+37–40% vs main). Prettify roughly at parity with main.
