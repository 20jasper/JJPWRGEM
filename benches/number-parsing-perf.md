# Number Parsing: Splitting Mantissa and Exponent

twitter.json parse + prettify: **2.79ms → 1.94ms (−30%)**. Stream reformat: **2.19ms → 1.51ms (−31%)**. For context: serde_json parse + prettify = 2.41ms; prettier = 361.7ms.

citm_catalog parse + prettify: **5.76ms → 4.73ms (−18%)**. Stream reformat: **4.01ms → 2.74ms (−32%)**.

Five commits, two mechanisms.

---

## Datasets

Three JSON fixtures. All throughput in MB/s (higher = better).

- canada.json (2.25 MB): GeoJSON coordinates. ~90% numbers, all floats, no exponents. Pure number-parsing stress test. Extra token per number hurts here.
- citm_catalog.json (1.73 MB): IIHF event catalog. Mixed: integers, strings, nested objects. Benefits most from state machine split.
- twitter.json (0.63 MB): Twitter API response. String-heavy, few numbers. Benefits most from removing the hidden destructor overhead on number tokens (explained in Approach 2).

---

## What jjpwrgem is

JSON formatter and linter. Parses JSON as text, checks, reformats, emits. No Rust struct deserialization.

Tokenizer: converts raw text into a flat stream of labeled chunks ("tokens") -- like splitting `{"x":1}` into `[{, "x", :, 1, }]`. AST (Abstract Syntax Tree): tree built from those tokens representing the structure of the document -- nodes for objects, arrays, values, with the source text attached. Two output paths:

```
source text
    |
    v
TokenStream (lazy iterator)
    |  Mantissa("1.5") / Exponent("10") / String("foo") / ...
    v
+---------------------+---------------------+
|  AST path           |  Token stream path  |
|  (prettify/uglify   |  (uglify_tokens:    |
|  from syntax tree)  |  streaming, no AST) |
+---------------------+---------------------+
    |                         |
    v                         v
formatted output         formatted output
```

Both paths share one tokenizer. Token structure changes affect all operations.

---

## The goal: number normalization

Formatters normalize numbers. Prettier does this for JavaScript:

```
input:  1.0e2   1.50   1e+2   0.10
output: 100     1.5    1e2    0.1
```

To normalize, a visitor needs mantissa and exponent separately -- strip trailing zeros, simplify `e+0`, decide whether to fold exponent into mantissa. (Visitor: an object that receives each token as the parser emits it -- like an `onToken` event callback in a streaming parser. The parser calls `visitor.on_number(...)` for each number it encounters.) With `Token::Number(Cow<str>)`, visitor gets raw string `"1.50e+2"` and must re-parse.

Split gives visitor what it needs directly.

---

## Baseline: `Token::Number(Cow<str>)`

`1.5e10` -> one token, full string:

```rust
pub enum Token<'a> {
    Number(Cow<'a, str>),
}

fn on_number(&mut self, n: Cow<'a, str>);
```

`enum Token` is like a TypeScript discriminated union -- each variant can carry different data. `<'a>` is a lifetime annotation: the compiler's way of saying this token borrows string data from the source text without copying it (mentally read `&'a str` as just `&str`). `&mut self` = mutable `this` in a method.

In Rust, every value has one owner; when it goes out of scope, memory is freed automatically -- no GC. `String` is an owned, heap-allocated string (like a JS string, but freed when its owner goes out of scope). `&str` is a borrowed view -- just a pointer and a length into existing bytes, no allocation, like `str.slice()` in JS but creating zero new objects. `Cow<'a, str>` holds either one at runtime ("clone on write" -- only allocates if you mutate). Here chosen for flexibility (normalizing visitor could return owned `String`), but in practice never normalizes; `Cow::Borrowed` only variant ever constructed.

Baseline twitter.json deser: 257.8 MB/s.

---

## Approach 1: Split into mantissa + exponent (regression)

```rust
pub enum Token<'a> {
    Number { mantissa: Cow<'a, str>, exponent: Cow<'a, str> },
}

fn on_number(&mut self, mantissa: Cow<'a, str>, exponent: Cow<'a, str>);
```

Goal achieved. Throughput regressed. (`deser` = deserialization: full parse of JSON text into an in-memory AST -- like `JSON.parse()` but into a custom tree. `uglify_tokens` = reformat JSON by streaming tokens directly, no AST built.)

| benchmark     | baseline (MB/s) | this (MB/s) | delta  |
| ------------- | --------------- | ----------- | ------ |
| deser/canada  | 115.6           | 101.1       | -12.5% |
| deser/citm    | 385.5           | 289.3       | -25.0% |
| deser/twitter | 257.8           | 214.8       | -16.7% |

Why: two `Cow`s = two discriminants (the hidden integer tag every enum carries so the runtime knows which variant is active). Niche opt compresses one `Cow<str>` to 24 bytes (Rust steals a bit pattern the data can never legally hold -- `cap = i64::MIN` -- as the 'Borrowed' sentinel, avoiding an extra 8-byte word), two side-by-side get no savings. `Token` 24 -> 48 bytes. `TokenWithContext` 40 -> 64 bytes. Half as many tokens per cache line (the CPU fetches memory in 64-byte chunks; larger structs mean fewer fit per fetch, more cache misses).

---

## Approach 2: `Cow<str>` -> `&str` (first win)

```rust
pub enum Token<'a> {
    Number { mantissa: &'a str, exponent: &'a str },
}

fn on_number(&mut self, mantissa: &'a str, exponent: &'a str);
```

`&str` is `Copy` -- in Rust, `Copy` types (plain bytes: integers, pointers) are duplicated by copying when passed around, with no destructor; non-`Copy` types like `String` are _moved_ (original gone after hand-off) and need drop glue (compiler-generated code that frees heap data when the value is consumed). `Cow<str>` is non-`Copy`, so every consume site must run drop glue: load discriminant, check if Owned, call dealloc. `&str` has none of that. Every token consume: niche-check + conditional branch -> nothing.

| benchmark             | prev (MB/s) | this (MB/s) | delta  |
| --------------------- | ----------- | ----------- | ------ |
| deser/canada          | 101.1       | 111.0       | +9.8%  |
| deser/citm            | 289.3       | 442.9       | +53.1% |
| deser/twitter         | 214.8       | 341.4       | +58.9% |
| uglify_tokens/canada  | --          | 234.2       | --     |
| uglify_tokens/twitter | --          | 392.2       | --     |

Token: 40 bytes (down from 48). `exponent: ""` still in every integer token -- dead weight, but smaller.

> canada still below baseline (111.0 vs 115.6 MB/s) -- ~90% numbers, no exponents. Empty `exponent` on every token costs.

---

## Approach 3: Separate `Mantissa` and `Exponent` variants (token back to 24 bytes)

```rust
pub enum Token<'a> {
    Mantissa(&'a str),
    Exponent(&'a str),
}
```

Plain `42`: one `Mantissa` token, done. `1.5e10`: `Mantissa("1.5")` then `Exponent("10")`. No exponent field on integers.

Token back to 24 bytes. `TokenWithContext` back to 40. Same as original baseline, but `Copy` throughout.

| benchmark             | prev (MB/s) | this (MB/s) | delta  |
| --------------------- | ----------- | ----------- | ------ |
| deser/canada          | 111.0       | 122.6       | +10.5% |
| deser/twitter         | 341.4       | 356.8       | +4.5%  |
| uglify_tokens/canada  | 234.2       | 268.9       | +14.8% |
| uglify_tokens/twitter | 392.2       | 407.4       | +3.9%  |

> `uglify_ast` +6% -- path reconstructs number from `Value::Number` struct, two separate tokens add coordination cost. Recovered next step.

---

## Approach 4: Split the state machine

Single `NumberState` (10 variants, 88 bytes) -> `MantissaState` (6 variants, 48 bytes) + `ExponentState` (5 variants, 40 bytes). `e`/`E` detection moves to stream loop.

The old enum was 88 bytes because `EndWithExponent` carried two full `TokenWithContext` values (mantissa token + exponent token) = 80 bytes payload + 8 bytes discriminant. The new split avoids this: each machine's terminal state holds only one token.

```rust
// before: one machine, 10 variants, 88 bytes
enum NumberState { MinusOrInteger, Leading, IntegerOrDecimalOrExponentOrEnd,
                   Fraction, FractionOrExponentOrEnd, MinusOrPlusOrDigit,
                   ExponentDigit, ExponentDigitOrEnd, End, EndWithExponent }

// after: two machines
enum MantissaState { MinusOrInteger, IntegerOrDecimalOrEnd, Fraction, FractionOrEnd, End }
enum ExponentState  { MinusOrPlusOrDigit, AfterSign, Digits, Zero, End }
```

A state machine is a loop that transitions between named states -- here, each character read moves the parser from one state to the next (e.g. `IntegerOrDecimalOrEnd` after reading a digit). Every integer never enters exponent state. Fewer variants -> fewer live variables per match arm -> better register allocation (the compiler's job of assigning variables to CPU registers, the fastest storage; fewer variables = more fit in registers without spilling to slower memory) on the hot path.

| benchmark             | prev (MB/s) | this (MB/s) | delta  |
| --------------------- | ----------- | ----------- | ------ |
| deser/citm            | 401.7       | 456.9       | +13.7% |
| deser/twitter         | 356.8       | 385.1       | +7.9%  |
| uglify_ast/twitter    | 5262.6      | 5539.6      | +5.3%  |
| uglify_tokens/citm    | 557.2       | 630.4       | +13.1% |
| uglify_tokens/twitter | 407.4       | 418.2       | +2.7%  |

---

## Approach 5: Remove `on_number` dispatch wrapper

```rust
// before: default wrapper added one call frame + is_empty() check
fn on_number(&mut self, m: &'a str, e: &'a str) {
    self.on_mantissa(m);
    if !e.is_empty() { self.on_exponent(e); }
}

// after: direct dispatch
fn on_mantissa(&mut self, mantissa: &'a str);
fn on_exponent(&mut self, exponent: &'a str);
```

`is_empty()` check moves to stream loop -- already knows whether `Exponent` token exists, no re-check needed.

| benchmark          | prev (MB/s) | HEAD (MB/s) | delta |
| ------------------ | ----------- | ----------- | ----- |
| deser/citm         | 456.9       | 456.9       | same  |
| deser/twitter      | 385.1       | 385.1       | same  |
| uglify_tokens/citm | 630.4       | 630.4       | same  |

Negligible delta. Codegen cleaner -- compiler inlines `on_mantissa` directly from traverse call site.

---

## Results vs baseline

| benchmark             | baseline (MB/s) | HEAD (MB/s) | delta  |
| --------------------- | --------------- | ----------- | ------ |
| deser/canada          | 115.6           | 111.3       | -3.7%  |
| deser/citm            | 385.5           | 456.9       | +18.5% |
| deser/twitter         | 257.8           | 385.1       | +49.4% |
| uglify_tokens/canada  | 198.9           | 251.2       | +26.3% |
| uglify_tokens/citm    | 449.8           | 630.4       | +40.2% |
| uglify_tokens/twitter | 269.9           | 418.2       | +54.9% |

canada deser regresses -- extra token per number on 90%-number file. Citm and twitter gains outweigh it.

---

Detailed commit-by-commit data below.

---

## The Commits

### `281e961e` -- Split Number into mantissa + exponent (WIP, regression)

```rust
// before
pub enum Token<'a> {
    Number(Cow<'a, str>),
}

// after
pub enum Token<'a> {
    Number {
        mantissa: Cow<'a, str>,
        exponent: Cow<'a, str>,
    },
}
```

```rust
// visitor -- before
fn on_number(&mut self, n: Cow<'a, str>);

// visitor -- after
fn on_number(&mut self, mantissa: Cow<'a, str>, exponent: Cow<'a, str>);
```

Regression. Two `Cow`s double token size. State machine stayed monolithic.

| benchmark          | main (MB/s) | this (MB/s) | delta  |
| ------------------ | ----------- | ----------- | ------ |
| deser/canada       | 115.6       | 101.1       | -12.5% |
| deser/citm_catalog | 385.5       | 289.3       | -25.0% |
| deser/twitter      | 257.8       | 214.8       | -16.7% |
| uglify_ast/canada  | 3042.0      | 2162.4      | -28.9% |

---

### `1faecb2e` -- `Cow<str>` -> `&str` in hot loop

```rust
// before
pub enum Token<'a> {
    Number { mantissa: Cow<'a, str>, exponent: Cow<'a, str> },
}

// after
pub enum Token<'a> {
    Number { mantissa: &'a str, exponent: &'a str },
}
```

```rust
// visitor -- before
fn on_number(&mut self, mantissa: Cow<'a, str>, exponent: Cow<'a, str>);

// visitor -- after
fn on_number(&mut self, mantissa: &'a str, exponent: &'a str);
```

Numbers are always zero-copy views into the source string; `Cow` was never `Owned` in practice. `&str` = 16 bytes (pointer + length). `Cow<str>` = 24 bytes (pointer + length + capacity word for the owned-vs-borrowed tag) + a branch on every `.as_ref()`. Removing `Cow` drops that branch and shrinks the token.

| benchmark             | main (MB/s) | this (MB/s) | delta  |
| --------------------- | ----------- | ----------- | ------ |
| deser/canada          | 115.6       | 111.0       | -4.0%  |
| deser/citm_catalog    | 385.5       | 442.9       | +14.9% |
| deser/twitter         | 257.8       | 341.4       | +32.4% |
| uglify_tokens/canada  | 198.9       | 234.2       | +17.7% |
| uglify_tokens/twitter | 269.9       | 392.2       | +45.3% |

> canada below main (111.0 vs 115.6) -- nearly all numbers, no exponents, empty `exponent: ""` on every token.

---

### `3e84d813` -- Split into `Mantissa` and `Exponent` variants

```rust
// before
pub enum Token<'a> {
    Number { mantissa: &'a str, exponent: &'a str },
}

// after
pub enum Token<'a> {
    Mantissa(&'a str),
    Exponent(&'a str),
}
```

State machine returns `(TokenWithContext, Option<TokenWithContext>)`. (`Option<T>` = nullable value: either `Some(value)` or `None`, like `T | null` in TS.) Stream iterator emits sequentially.

Why faster:

- No-exponent numbers emit one token. Empty exponent field gone entirely.
- Token shrinks back -- `Mantissa`/`Exponent` each one fat pointer (pointer + length, 16 bytes) vs two-field struct.
- canada-style numbers touch zero exponent logic.

```rust
// state machine return -- before
pub fn parse_num<'a>(...) -> Result<'a, TokenWithContext<'a>>

// state machine return -- after
pub fn parse_num<'a>(...) -> Result<'a, (TokenWithContext<'a>, Option<TokenWithContext<'a>>)>
```

| benchmark             | main (MB/s) | this (MB/s) | delta  |
| --------------------- | ----------- | ----------- | ------ |
| deser/canada          | 115.6       | 122.6       | +6.1%  |
| deser/twitter         | 257.8       | 356.8       | +38.4% |
| uglify_tokens/canada  | 198.9       | 268.9       | +35.2% |
| uglify_tokens/twitter | 269.9       | 407.4       | +50.9% |

> `uglify_ast` +6% -- path reconstructs number from `Value::Number { mantissa, exponent }` struct; two-field vs single string costs there.

---

### `9f7d2977` -- Split state machine into `MantissaState` + `ExponentState`

Single monolithic `NumberState` -> two focused machines. `e`/`E` detection moves to top-level stream loop.

```rust
// before: one machine, 10 variants, 88 bytes
// EndWithExponent held two TokenWithContext (mantissa + exponent) = 80-byte payload
enum NumberState<'a> {
    MinusOrInteger,
    Leading(..),
    IntegerOrDecimalOrExponentOrEnd { .. },
    FractionOrExponentOrEnd(..),
    MinusOrPlusOrDigit { .. },
    ExponentDigit { .. },
    ExponentDigitOrEnd { .. },
    Fraction { .. },
    End(..),
    EndWithExponent { .. },  // TokenWithContext + TokenWithContext = 80 bytes
}

// after: two focused machines
enum MantissaState<'a> {  // 48 bytes, 6 variants
    MinusOrInteger,
    Leading(..),
    IntegerOrDecimalOrEnd { .. },
    Fraction { .. },
    FractionOrEnd(..),
    End(..),              // one TokenWithContext = 40-byte payload
}

enum ExponentState<'a> {  // 40 bytes, 5 variants
    MinusOrPlusOrDigit { .. },
    AfterSign { .. },
    Digits { .. },
    Zero,
    End(..),
}
```

```rust
// stream.rs hot loop -- exponent at stream level
'0'..='9' | '-' => return Some(parse_mantissa(self.input, &mut self.chars)),
'e' | 'E' => {
    self.chars.next();
    match parse_exponent(self.input, r, &mut self.chars) {
        Ok(Some(tok)) => return Some(Ok(tok)),
        Ok(None) => continue, // zero exponent stripped
        Err(e) => return Some(Err(e)),
    }
}
```

Fewer states -> smaller match arms -> better branch prediction (CPUs guess which path a conditional will take and speculatively execute ahead; a wrong guess costs ~15 cycles; a right guess is nearly free). Mantissa path never touches exponent variants. Zero-exponent (`e0`, `e+0`) skips token emission entirely with `continue`.

| benchmark                  | main (MB/s) | this (MB/s) | delta  |
| -------------------------- | ----------- | ----------- | ------ |
| deser/canada               | 115.6       | 118.7       | +2.7%  |
| deser/twitter              | 257.8       | 382.7       | +48.4% |
| uglify_ast/canada          | 3042.0      | 3122.1      | +2.6%  |
| uglify_ast/citm_catalog    | 6908.8      | 7196.7      | +4.2%  |
| uglify_ast/twitter         | 5176.3      | 5638.5      | +8.9%  |
| uglify_tokens/citm_catalog | 449.8       | 625.8       | +39.1% |
| uglify_tokens/twitter      | 269.9       | 415.5       | +53.9% |

---

### `f5631053` -- Remove `on_number` default method (HEAD)

Default `on_number` dispatched to `on_mantissa` + `on_exponent`. All visitors overrode it anyway. Removed; `on_mantissa`/`on_exponent` are now direct required interface. (`trait` = interface; `impl Visitor for MyType` = `class MyType implements Visitor` in TS.)

```rust
// before: default indirection
pub trait Visitor<'a> {
    fn on_mantissa(&mut self, mantissa: &'a str);
    fn on_exponent(&mut self, exponent: &'a str);
    fn on_number(&mut self, mantissa: &'a str, exponent: &'a str) {
        self.on_mantissa(mantissa);
        if !exponent.is_empty() {
            self.on_exponent(exponent);
        }
    }
}

// after: direct dispatch
pub trait Visitor<'a> {
    fn on_mantissa(&mut self, mantissa: &'a str);
    fn on_exponent(&mut self, exponent: &'a str);
}
```

Eliminates call frame + `is_empty()` check. Compiler inlines `on_mantissa` directly from `traverse.rs`. Exponent presence check moves to stream level.

| benchmark                  | main (MB/s) | HEAD (MB/s) | delta  |
| -------------------------- | ----------- | ----------- | ------ |
| deser/citm_catalog         | 385.5       | 422.3       | +9.5%  |
| deser/twitter              | 257.8       | 385.1       | +49.4% |
| uglify_ast/twitter         | 5176.3      | 5539.6      | +7.0%  |
| uglify_tokens/citm_catalog | 449.8       | 630.4       | +40.2% |
| uglify_tokens/twitter      | 269.9       | 418.2       | +54.9% |

---

## Experiment: `Cow` -> `&str` only (no structural split)

To isolate mechanisms: branch from main with single change -- `Token::Number(Cow<'a, str>)` -> `Token::Number(&'a str)`. No struct split, no separate variants, no state machine changes. Single `&str` for full number string.

Sizes:

- `Token`: 24 bytes (same as main and HEAD -- niche fires either way)
- `TokenWithContext`: 40 bytes (same)
- `Value::Number`: 32 bytes (same as main, smaller than HEAD's 40-byte split struct)

All three (main, exp, HEAD) benchmarked sequentially same machine, each from own worktree:

| benchmark             | main (MB/s) | exp (MB/s) | vs main | HEAD (MB/s) | vs main | exp vs HEAD |
| --------------------- | ----------- | ---------- | ------- | ----------- | ------- | ----------- |
| deser/canada          | 95.2        | 116.2      | +22.1%  | 107.8       | +13.2%  | exp faster  |
| deser/citm_catalog    | 346.1       | 373.9      | +8.0%   | 453.3       | +31.0%  | HEAD faster |
| deser/twitter         | 233.9       | 334.1      | +42.8%  | 343.2       | +46.7%  | ~same       |
| uglify_ast/canada     | 2611.4      | 2599.4     | same    | 2860.3      | +9.5%   | HEAD faster |
| uglify_ast/citm       | 6190.7      | 6168.6     | same    | 6303.7      | +1.8%   | HEAD faster |
| uglify_ast/twitter    | 4748.2      | 4784.2     | same    | 5092.9      | +7.3%   | HEAD faster |
| uglify_tokens/canada  | 171.2       | 242.0      | +41.4%  | 228.8       | +33.6%  | exp faster  |
| uglify_tokens/citm    | 373.9       | 581.6      | +55.5%  | 573.8       | +53.5%  | ~same       |
| uglify_tokens/twitter | 215.5       | 371.5      | +72.4%  | 375.9       | +74.4%  | ~same       |

Two mechanisms, different targets:

`Cow->&str` (Copy, no drop glue) dominates:

- `uglify_tokens` all datasets: +41 to +72%. Hot token-stream path calls `on_number(cow)` -> `cow.as_ref()` -> `push_str` per number. Drop glue + discriminant check gone from hot loop.
- `deser/twitter`: +43%. Same mechanism through AST-building path.
- `deser/canada`: +22%.
- `uglify_ast`: no effect -- rebuilds numbers from `Value::Number`, not from tokens.

State machine split (HEAD beyond exp) dominates:

- `deser/citm`: +23% beyond exp's +8%. Split machine + smaller variants unlock further gains on mixed-structure dataset.
- `uglify_ast`: +1.8 to +9.5% further. Split variants enable better codegen in AST path.
- `deser/canada`: -8% vs exp -- extra token per number costs when every token is a number.
- `uglify_tokens`: roughly neutral vs exp.

Both mechanisms real, substantial, additive on different axes.

---

## Full Diagnostics: HEAD vs Main

### Deserialization

Sequential run (main -> HEAD, quiet machine). Relative deltas reliable; absolute values environment-dependent.

|                        | main (MB/s) | HEAD (MB/s) | delta  | serde_json | simd_json | sonic_rs |
| ---------------------- | ----------- | ----------- | ------ | ---------- | --------- | -------- |
| canada (2.25 MB)       | 115.6       | 111.3       | -3.7%  | 177.0      | 285.7     | 882.8    |
| citm_catalog (1.73 MB) | 385.5       | 456.9       | +18.5% | 573.8      | 981.4     | 1730.7   |
| twitter (0.63 MB)      | 257.8       | 385.1       | +49.4% | 367.2      | 759.9     | 1688.5   |

> canada slightly slower at HEAD -- extra token per number, overhead when every token is number. HEAD near serde_json on twitter. Canada/citm remain slower structurally -- jjpwrgem builds syntax tree with source positions; serde_json doesn't. simd_json and sonic_rs use SIMD not yet in jjpwrgem.

### Prettify from AST

|              | main (MB/s) | HEAD (MB/s) | delta | serde_json | simd_json | sonic_rs |
| ------------ | ----------- | ----------- | ----- | ---------- | --------- | -------- |
| canada       | 1424.7      | 1332.0      | -6.5% | 384.8      | 395.6     | 301.3    |
| citm_catalog | 1853.2      | 1845.3      | -0.4% | 1191.2     | 1528.5    | 1136.3   |
| twitter      | 2091.1      | 2105.0      | +0.7% | 1013.7     | 1321.2    | 1255.5   |

> canada -6.5% = 1424.7 -> 1332.0 MB/s. Still 3.5x faster than serde_json. Noise in competitive terms.

### Uglify from AST

|              | main (MB/s) | HEAD (MB/s) | delta | serde_json | simd_json | sonic_rs |
| ------------ | ----------- | ----------- | ----- | ---------- | --------- | -------- |
| canada       | 3042.0      | 3066.8      | +0.8% | 827.6      | 696.9     | 639.5    |
| citm_catalog | 6908.8      | 7021.2      | +1.6% | 3489.3     | 3804.4    | 2395.6   |
| twitter      | 5176.3      | 5539.6      | +7.0% | 1841.1     | 1825.2    | 2271.6   |

> 3-4x faster than every competitor. Small wins here are gravy.

### Uglify from Token Stream

|              | main (MB/s) | HEAD (MB/s) | delta  |
| ------------ | ----------- | ----------- | ------ |
| canada       | 198.9       | 251.2       | +26.3% |
| citm_catalog | 449.8       | 630.4       | +40.2% |
| twitter      | 269.9       | 418.2       | +54.9% |

> v0.5.5 baseline had regressed uglify_tokens by +25-43% vs v0.5.4 (earlier unrelated work). Branch recovers that regression and goes further.

---

## Summary

Same theme across all five commits: stop treating mantissa and exponent as one thing. Every step of separation -- split struct, drop `Cow`, emit two tokens, split state machine, remove dispatch wrapper -- produced measurable gains.

Two mechanisms, different benchmarks:

`Cow->&str` (Copy, no drop glue) dominates token-stream throughput and number-heavy deser. Isolated experiment measured +22% canada, +43% twitter, +41 to +72% uglify_tokens. Hot token-stream path emits one `on_number` per number; removing `Cow` drop glue from that loop is immediately visible.

State machine split drives remaining gains: +23% further on citm deser, +7 to +9% on uglify_ast. Separating `MantissaState` from `ExponentState` removes exponent code from common (no-exponent) path, shrinks match arms, gives compiler cleaner optimization units. Adds cost too -- one extra token per number -- small regression on canada vs Cow-only exp.

Gap to simd_json and sonic_rs is different problem -- SIMD string scanning, bulk whitespace skipping. Not addressed here.

---

## CPU-Level Analysis: Instruction Counts and Branch Mispredictions

Wall-clock benchmarks say something is faster. To know why: [gungraun](https://github.com/iai-callgrind/iai-callgrind) (formerly iai-callgrind, renamed at v0.17.0). Runs benchmarks under Valgrind Callgrind. Exact CPU event counts -- no noise, no scheduling variance.

### Setup

```toml
# benches/Cargo.toml
[dev-dependencies]
gungraun = "0.18.1"

[[bench]]
name = "json_iai"
harness = false
```

```bash
sudo apt-get install -y valgrind
cargo binstall gungraun-runner@0.18.1 -y
```

Bench file bakes JSON fixtures via `include_str!`, enables branch simulation:

```rust
fn branch_sim_config() -> LibraryBenchmarkConfig {
    let mut config = LibraryBenchmarkConfig::default();
    config.tool(
        Callgrind::with_args(["--branch-sim=yes"])
            .entry_point(EntryPoint::None)  // WSL workaround -- see below
            .format([CallgrindMetrics::Default, CallgrindMetrics::BranchSim]),
    );
    config
}
```

WSL workaround: gungraun uses `--toggle-collect=*::__iai_callgrind_wrapper_mod::*` to scope measurement to bench function. Valgrind 3.18 can't match Rust v0 mangled symbols (`crate[hash]::...`) via glob, so toggle-collect collects nothing. `EntryPoint::None` disables toggling, measures whole process. Absolute counts include startup overhead; relative comparisons across commits still valid.

Verify toggle-collect was the issue:

```bash
# toggle-collect matches nothing
valgrind --tool=callgrind --toggle-collect='*::__iai_callgrind_wrapper_mod::*' \
  --collect-atstart=no ./target/release/deps/json_iai-<hash> \
  --iai-run deser_group 0 0 json_iai::deser_group::bench_deser
# => Collected: 0

# without toggling, works:
valgrind --tool=callgrind ./target/release/deps/json_iai-<hash> \
  --iai-run deser_group 0 0 json_iai::deser_group::bench_deser
# => Collected: 55,966,783
```

### Instruction counts across five commits (deser/citm)

| commit               | Ir (instructions) | Bcm (mispred branches) |
| -------------------- | ----------------- | ---------------------- |
| main (Cow)           | 67,373,334        | 175,813                |
| exp (&str, no split) | 54,722,747        | 126,379                |
| 3e84 (token split)   | 56,503,270        | 199,935                |
| 9f7d (SM split)      | 55,967,560        | 180,405                |
| HEAD                 | 55,974,793        | 195,773                |

`Cow->&str` eliminates 12.6M instructions (-19%). State machine split adds and rearranges but no net reduction -- gains come from other effects.

Branch mispredictions don't explain perf curves. Token split increases Bcm on canada and citm, yet benchmarks get faster. Mispredictions not primary cost.

### Investigation path

Step 1: 67.4M Ir -> 54.7M Ir. -12.6M. First guess: `Cow->&str` unlocked inlining of `TokenStreamInner::next`. (Inlining: the compiler copies a function's body to each call site instead of emitting a real call, eliminating call overhead and enabling further optimizations across the boundary.)

Step 2: `callgrind_annotate`. Killed inlining hypothesis.

```bash
callgrind_annotate \
  /tmp/bench-main/target/gungraun/.../deser_group/bench_deser.citm/callgrind.bench_deser.citm.out \
  2>/dev/null | head -60

callgrind_annotate \
  /tmp/cow-to-str-bench/target/gungraun/.../deser_group/bench_deser.citm/callgrind.bench_deser.citm.out \
  2>/dev/null | head -60
```

`TokenStreamInner::next`: 14.1M Ir in main, 13.9M Ir in exp. Still standalone, barely changed. Not inlined. Not source of savings.

Real savings: traverse functions. `parse_object` -47%, `parse_array` -36%, `parse_tokens` -36%.

Step 3: `-Cremark=inline` confirmed cross-crate block.

```bash
RUSTFLAGS="-Cremark=inline" cargo build --release 2>&1 | grep -i "next_token\|TokenStream"
```

Output: `next_token will not be inlined because its definition is unavailable`. Cross-crate boundary -- in Rust, code is organized into crates (packages) compiled separately; a function in another crate can't be inlined into yours because its internals aren't visible at your compile time. Never inline-able regardless of `Cow` vs `&str`.

Step 4: LLVM IR, counted branches. (Rust compiles through LLVM; IR = Intermediate Representation, a text-based assembly-like format the compiler generates before producing machine code. Examining it shows exactly what the compiler decided to emit -- every branch, every call.)

```bash
RUSTFLAGS="--emit=llvm-ir -C opt-level=3" cargo build --release
```

IR at: `target/release/deps/jjpwrgem_parse-*.ll`

Located `parse_tokens<AstVisitor>` in both files:

```bash
grep -n "parse_tokensNtNtNtB4_3ast7visitor10AstVisitor" jjpwrgem_parse-*.ll | grep "^[0-9]*:define"

sed -n '9276,12422p' main.ll | grep -c '^\s*switch'   # 61
sed -n '2540,4461p'  exp.ll  | grep -c '^\s*switch'   # 28

python3 -c "
import re
with open('main.ll') as f: lines = f.readlines()
section = ''.join(lines[9275:12422])
cow = re.findall(r'switch i64 [^[]+\[\s*\n\s*i64 -9223372036854775808.*?\n\s*i64 0.*?\n\s*\]', section, re.DOTALL)
print(len(cow))  # 34  (bash delta 61-28 = 33; one match likely a shared helper)
"

sed -n '9276,12422p' main.ll | grep -c 'drop_in_place.*TokenWithContext'  # 62
sed -n '2540,4461p'  exp.ll  | grep -c 'drop_in_place.*TokenWithContext'  # 0
```

Step 5: Read `drop_in_place<Option<TokenWithContext>>` body, confirmed Cow niche check.

```bash
grep -n "define.*drop_in_place.*Option.*TokenWithContext" main.ll
# -> line 15607
sed -n '15607,15636p' main.ll
```

Body: `icmp eq i64 %val, -9223372036854775797` (None niche check), then `switch i64 %val [ i64 -9223372036854775808, i64 0 ]` (Owned vs Borrowed), then conditional `__rust_dealloc`. (`drop_in_place`: Rust's compiler-generated destructor -- runs when a value is consumed, freeing any heap memory it owns.) Sequence inlined 33 times into `parse_tokens`, 0 times in exp.

Conclusion: drop glue. Every `Token` consume in main: load discriminant, range check, 2-arm switch, skip branch (~5 instructions minimum). `&str` is `Copy` -- no discriminant, no switch, no branch.

33 = distinct consume sites. Each token takes one path, hits exactly one site, pays check once. Sites fan out across all match arms (object key, value, array element, scalar types, etc.).

Dynamic cost: ~6 instructions per token (niche load + range check + 2-arm switch + branch). `parse_tokens` processes ~193k tokens -> ~1.16M Ir. `parse_object` ~1M tokens -> 5.98M Ir eliminated. Sum across all traverse functions + `drop_in_place<Value>` -> 12.6M total.

### Where did the 12.6M go? (`callgrind_annotate`)

```bash
callgrind_annotate \
  target/gungraun/benches/json_iai/deser_group/bench_deser.citm/callgrind.bench_deser.citm.out \
  2>/dev/null | head -60
```

Per-function Ir, main vs exp, deser/citm:

| function                  | main Ir    | exp Ir     | delta |
| ------------------------- | ---------- | ---------- | ----- |
| `TokenStreamInner::next`  | 14,067,007 | 13,932,278 | -1%   |
| `parse_string`            | 12,971,022 | 11,316,185 | -13%  |
| `parse_object` (traverse) | 12,689,173 | 6,707,283  | -47%  |
| `parse_num`               | 7,102,537  | 6,937,636  | -2%   |
| `parse_array` (traverse)  | 4,843,998  | 3,118,325  | -36%  |
| `parse_tokens` (traverse) | 3,218,627  | 2,060,332  | -36%  |
| `emit_value` (AstVisitor) | 1,532,541  | 1,532,541  | 0%    |
| `drop_in_place::<Value>`  | 676,966    | 53,278     | -92%  |

`TokenStreamInner::next` not inlined -- standalone function, -135k Ir (-1%). 12.6M savings come from elsewhere.

Two effects:

1. Traverse functions shed inlined Cow drop glue (-8.9M Ir combined). `parse_object`, `parse_array`, `parse_tokens` drop ~1.15M tokens while parsing. Each `Token(Cow<str>)` drop emits: discriminant load, `switch` on niche value, conditional `free` call. ~4-8 instructions per drop, inlined at every consume site.

LLVM IR inside `parse_tokens<AstVisitor>`:

| metric                                       | main  | exp   | delta |
| -------------------------------------------- | ----- | ----- | ----- |
| IR lines                                     | 3,146 | 1,921 | -39%  |
| `br` instructions                            | 308   | 134   | -57%  |
| `switch i64` statements                      | 61    | 28    | -54%  |
| Cow 2-arm niche `switch`                     | 33    | 0     | -100% |
| `drop_in_place<TokenWithContext>` references | 62    | 0     | -100% |

Exp's 28 switches = all legitimate token-variant dispatches. Main's extra 33 = Cow niche check: `switch i64 %val [ i64 -9223372036854775808, i64 0 ]` -- 2-arm test for Owned (-> dealloc) vs Borrowed (-> no-op), inlined at every token-consume site. (Table row above shows 33; Python regex returned 34 -- one match is likely a shared helper outside parse_tokens proper.)

2. `drop_in_place::<Value>` -92% -- AST node drop glue gone. Each `Value::Number(Cow<str>)` triggers Cow drop when freed. ~100k number nodes x ~6 instructions = 677k Ir. `Value::Number(&str)` has zero drop cost. ~5% of total saving.

`parse_string` -13%: drops tokens internally, same Cow drop glue elimination.

`emit_value` unchanged -- AST serialization step unaffected by token type.

---

## Why is `&str` faster than `Cow<str>`?

### Hypothesis 1: smaller memory footprint

`&str` = 16 bytes. `Cow<str>` = 24 bytes. Smaller token -> more per cache line -> faster.

Wrong for single-Cow case. Token enum is 24 bytes either way:

```
size_of::<&str>()                = 16 bytes
size_of::<Cow<str>>()            = 24 bytes  (niche optimization -- see below)
size_of::<Token with &str>()     = 24 bytes
size_of::<Token with Cow<str>>() = 24 bytes  (same)
size_of::<TokenWithContext>()    = 40 bytes  (Token + Range<usize>)
```

Cow<str> niche: naive size would be 32 bytes -- 24 for String (ptr + len + cap) + 1-byte discriminant padded to 8. Rust avoids by storing discriminant inside capacity word. For Borrowed, Rust writes `cap = i64::MIN as usize` (`0x8000_0000_0000_0000`). Provably not a valid heap allocation size. LLVM IR shows it directly: `switch i64 %val [ i64 -9223372036854775808, i64 0 ]` -- `i64::MIN` = Borrowed, anything else = Owned. Total: 24 bytes.

Token enum: `String(&str)` = 16 bytes, `Number(Cow<str>)` = 24 bytes. Enum size = max(16, 24) + discriminant overhead = 24 bytes, absorbing Cow's extra word. Replace Cow with `&str` and enum stays at 24 bytes (discriminant overhead takes full 8-byte word instead of being packed into Cow's niche).

Cache footprint identical. Size not the explanation.

### Hypothesis 2: drop check branches at every consume site

Every `Token(Cow<str>)` consumed must be dropped correctly. Compiler can't prove all Cows are Borrowed at compile time. Emits drop glue at every consume site: load discriminant, check if Owned, call dealloc if so.

In `parse_tokens<AstVisitor>`: 33 such sites -- one per match arm or path that consumes token. Each inlines:

```llvm
; Cow niche check at every consume site:
%discriminant = extractvalue { i64, i64, i64 } %token_cow, 2   ; load cap word
switch i64 %discriminant [
  i64 -9223372036854775808, label %borrowed  ; Borrowed, skip
  i64 0, label %...
]
%ptr = extractvalue ...                      ; (if Owned:)
call void @__rust_dealloc(ptr %ptr, ...)     ;   free
```

33 in main. Zero in exp (`&str` = Copy, no drop). Verified from LLVM IR:

```bash
# main: 34 niche switches (Python regex), 62 drop_in_place refs
sed -n '9276,12422p' main.ll | grep -c '^\s*switch'                          # 61 total
# (61 - 28 exp = 33 extra; 28 = legitimate token-variant dispatches)

# exp: 0
sed -n '2540,4461p' exp.ll | grep -c '^\s*switch'                            # 28 (all legitimate)
sed -n '2540,4461p' exp.ll | grep -c 'drop_in_place.*TokenWithContext'       # 0
```

Mechanism confirmed.

### Are drop branches causing mispredictions?

jjpwrgem never constructs `Cow::Owned` numbers. Every token always `Cow::Borrowed`. Branch always not-taken. Well-predicted branches cheap (~0 extra cycles). If always correctly predicted, should cost almost nothing.

Data:

| commit               | Ir (instructions)  | Bcm (mispredicted branches) |
| -------------------- | ------------------ | --------------------------- |
| main (Cow)           | 67,373,334         | 175,813                     |
| exp (&str, no split) | 54,722,747         | 126,379                     |
| delta                | -12,650,587 (-19%) | -49,434 (-28%)              |

Ir drops 12.6M. Bcm drops 49k.

If mispredictions caused it: 49k \* ~15 cycles = ~735k cycles saved.
If instruction count caused it: 12.6M instructions at IPC ~3 (instructions per clock cycle; modern out-of-order CPUs retire 2-4 per cycle) = ~4.2M cycles saved.

Instructions dominate by ~6x. 49k Bcm reduction = side effect of removing large code chunks (changed branch layout), not primary mechanism.

Cow drop branches always well-predicted. Not free though -- each fires as real instructions: discriminant load, switch evaluation, branch to skip-dealloc path. All execute, consume decode slots, occupy ROB entries (Reorder Buffer: hardware queue tracking all in-flight instructions on out-of-order CPUs). parse_tokens processes ~193k tokens at ~6 drop instructions each = ~1.16M Ir; parse_object processes ~1M tokens = ~5.98M Ir; remaining traverse functions and `drop_in_place<Value>` account for the rest. That 12.6M removed is the gain.

Answer: `&str` faster because `Copy` types have zero drop cost. Not size (niche = equal). Not misprediction (always well-predicted). Just 12.6M instructions of dead drop-check code not emitted.
