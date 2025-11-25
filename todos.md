## Todo

- [x] primitive values
  - [x] string values
    - [x] escapes
    - [x] bug where does not check for termination properly
    - [x] no unescaped control characters
    - [x] char to unicode escape
      - [ ] domain struct JSONChar
    - [ ] parse unicode escapes
    - [ ] check rfc
  - [ ] numbers
    - [ ] positive/negative (+/-)
    - [ ] powers (e/E)
  - [x] null
  - [x] boolean
- [x] objects
  - [x] multi keys
  - [ ] validate duplicate keys
    - [ ] what if key and val are both same?
- [ ] arrays
- [x] nested object
  - [x] arbitrarily nested
- [x] validate whitespace to skip
- [x] formatting
  - [x] make uglifier
  - [x] make prettifier
  - [x] general options
  - [ ] If object is empty, should be "{}"
  - [ ] standardize exponents
    - [ ] minifier should remove + and insignificant exponent 0s
- [ ] error handling
  - [x] display implementation for each token
  - [x] line/column
  - [ ] proper handling for multibyte characters
  - [x] describe expected token
    - [x] line/column of reason why expected (expected closing curly due to umatched open curly at xyz spot)
  - [x] Unterminated
  - [x] make naming more specific regarding Object and arrays (curly/square braces, open/opening/close/closing)
  - [x] point to character like in rust's errors
  - [ ] patches for each error type
  - [ ] patch applicability (how confident am I that this is right?)
  - [ ] snapshot testing
- [x] fix up From impls for Tokens. From<String> should not be on token, but boolean makes sense
- [ ] cli
  - [x] stdin
  - [ ] files
  - [ ] help screen
  - [ ] --fix option
  - [ ] colors (support-color)
  - [ ] look into insta snapshot docs
  - [ ]
- [ ] rebrand
  - [x] jjpwrgem crate name
  - [x] jjp bin name
  - [ ] axolotl skateboard

### dev tooling

- [ ] hook to format on commit
- [ ] just script to install all tools
- [ ] submodule installation

### Testing

- [x] conformance testing
  - [x] [JSONTestSuite](https://github.com/nst/JSONTestSuite) — Pure JSON

### bugs

- [ ] Tokenizer errors shouldn't report "expected start of JSON value" when parsing may be incomplete
- [ ] When the prior token is `{`, provide help suggesting the next token should be quoted or flag an unidentified token
  - [ ] Investigate `n_object_emoji`
  - [ ] Investigate `n_object_key_with_single_quotes`
  - [ ] Investigate `n_object_missing_colon.json`
- [x] Handle inappropriately escaped forward slash in `n_object_trailing_comment.json`
- [ ] Provide a hint to insert a value and closing curly when no significant characters remain
  - [ ] Investigate `n_object_missing_value.json`
- [ ] Fix error message to state that a quote is expected after the opening quote
  - [ ] Investigate `n_object_unterminated-value.json`
- [ ] Add helper messaging explaining that comments aren't allowed
- [ ] expected key but found , should add key val beforehand
  - [ ] n_structure_open_object_comma
- multichar emojis are not properly handled, see n_object_emoji.json
- [ ] remove multiple trailing commas
  - [ ] n_object_several_trailing_commas.json
- [ ] should recommend changing single quotes and backticks to double
  - [ ] n_object_single_quote
- [ ] error reporting for keywords, for example trbe marks t as error
