use const_format::str_repeat;

pub const OBJECT_MISSING_COLON_WITH_COMMA: &str = r#"{"hi", "#;
pub const OBJECT_MISSING_COLON_WITH_NULL: &str = r#"{"hi" null "#;
pub const OBJECT_MISSING_COLON_WITH_CLOSED_CURLY: &str = r#"{"hi" }"#;
pub const OBJECT_MISSING_COLON_WITH_LEADING_WHITESPACE: &str = r#"  {"hi"    "#;
pub const OBJECT_MISSING_COLON: &str = r#"{"hi"    "#;
pub const OBJECT_MISSING_VALUE: &str = r#"{"hi":"#;
pub const OBJECT_MISSING_COMMA_BETWEEN_VALUES: &str = r#"{"hi": null null"#;
pub const OBJECT_MISSING_COMMA_OR_CLOSING_WITH_WHITESPACE: &str = r#"{"hi": null     "#;
pub const OBJECT_TRAILING_COMMA_WITH_CLOSED: &str = r#"{"hi": null, }"#;
pub const OBJECT_TRAILING_COMMA: &str = r#"{"hi": null, "#;
pub const OBJECT_DOUBLE_OPEN_CURLY: &str = r"{{";
pub const OBJECT_OPEN_CURLY: &str = r"{";
pub const CLOSED_CURLY: &str = r"}";
pub const EMPTY_INPUT: &str = r"";
pub const UNEXPECTED_CHARACTER: &str = r"🦀";
pub const UNEXPECTED_ESCAPED_CHARACTER: &str = "\u{B}";
pub const DOUBLE_QUOTE: &str = r#"""#;
pub const OBJECT_WITH_LINE_BREAK_VALUE: &str = "{\"hi\": \"line\nbreak\"}";
pub const OBJECT_WITH_ADJACENT_STRINGS: &str = r#"{"hi": "bye" "ferris": null"#;
pub const OBJECT_EMPTY_THEN_OPEN: &str = r"{}{";
pub const MINUS_SIGN: &str = "-";
pub const LEADING_ZERO_MINUS_SIGN_ZERO: &str = "-000";
pub const LEADING_ZERO_ZERO: &str = "000";
pub const LEADING_ZERO_MINUS_SIGN_NONZERO: &str = "-012";
pub const LEADING_ZERO_NON_ZERO: &str = "012";
pub const VALID_TRAILING_ZEROS_FRACTION: &str = "1.0000";
pub const UNEXPECTED_LETTER_IN_NEGATIVE: &str = "-abcd";
pub const UNEXPECTED_LETTER_IN_NUMBER: &str = "1a";
// pub const FRACTION_MISSING_INTEGER: &str = ".29";
pub const NEGATIVE_FRACTION_MISSING_INTEGER: &str = "-.29";
pub const VALID_INTEGER: &str = "298";
pub const VALID_NEGATIVE_INTEGER: &str = "-298";
pub const MISSING_FRACTION: &str = "98.";
pub const VALID_FRACTION: &str = "98.8";
pub const VALID_NEGATIVE_FRACTION: &str = "-98.123456789";
pub const LONG_INTEGER: &str = "4390430989084309809824123456780099876654433231123413847890813843897873986381727319297072310970972784365768257862";
pub const LONG_FRACTION: &str = "439043098908430980982412345678009987.6654433231123413847890813843897873986381727319297072310970972784365768257862";
pub const EXPONENT_WITH_PLUS_SIGN: &str = "429e+6";
pub const EXPONENT_WITH_MINUS_SIGN: &str = "429e-6";
pub const NEGATIVE_FLOAT_WITH_EXPONENT: &str = "-98.25e12";
pub const EXPONENT_MISSING_TRAILING_DIGITS: &str = "98e";
pub const EXPONENT_MISSING_DIGITS_AFTER_SIGN: &str = "98e+";
pub const ARRAY_EMPTY: &str = "[]";
pub const ARRAY_SINGLE: &str = "[1]";
pub const ARRAY_MANY: &str = "[1, 2, 3]";
pub const ARRAY_SUBARRAYS: &str = "[[\"a\"], [true, false]]";
pub const ARRAY_OPEN: &str = "[";
pub const ARRAY_OPEN_WITH_VALUE: &str = "[1, [";
pub const ARRAY_MISSING_VALUE: &str = "[1, ]";
pub const ARRAY_OBJECTS_WITH_INCREASING_KEYS: &str = r#"[
    {},
    {"alpha": 1},
    {"alpha": 1, "beta": true},
    {"first": null, "second": "two", "third": 3},
    {
        "id": 42,
        "name": "nested",
        "flags": [true, false],
        "meta": {"note": "varied keys"}
    }
]"#;
pub const ARRAY_MULTIPLE_EMPTY_OBJECTS: &str = r"[{}, {}, {}, {}]";
pub const ARRAY_MANY_SINGLE_KEY_OBJECTS: &str = r#"[
    {"alpha": 1},
    {"beta": true},
    {"gamma": null},
    {"delta": "value"},
    {"epsilon": [1, 2, 3]}
]"#;
pub const ARRAY_MANY_TWO_KEY_OBJECTS: &str = r#"[
    {"id": 1, "label": "one"},
    {"id": 2, "label": "two"},
    {"id": 3, "label": "three"},
    {"id": 4, "label": "four"}
]"#;
pub const ARRAY_MANY_FIVE_KEY_OBJECTS: &str = r#"[
    {
        "id": 1,
        "name": "alpha",
        "flags": [true, false],
        "meta": {"info": "level1"},
        "count": 10
    },
    {
        "id": 2,
        "name": "beta",
        "flags": [false, true],
        "meta": {"info": "level2"},
        "count": 20
    }
]"#;
pub const ARRAYS_NESTED_FIVE_LEVELS_WITH_OBJECT: &str = r#"[[[[[
    {
        "depth": 5,
        "payload": ["text", 42, {"inner": true}, [null, false]],
        "meta": {"notes": "deep array"}
    }
]]]]]"#;
pub const INVALID_HEX_DIGIT_IN_ESCAPE: &str = r#""\u1FZA""#;
pub const INVALID_ESCAPED_CURLY: &str = r#""\{""#;
pub const OBJECT_WITH_LONG_KEYS: &str = r#"{
    "this is a very very very long key name with spaces and punctuation like --- ???": "value",
    "another extremely verbose key used for stress testing": {
        "inner object key with numbers 12345": "data"
    }
}"#;
pub const ARRAY_WITH_NESTED_OBJECTS: &str = r#"[
    {
        "level1": {
            "level2": {
                "level3": "value"
            }
        }
    },
    {
        "another": [
            {
                "deep": {
                    "key": 1
                }
            },
            {
                "deep": {
                    "key": 2
                }
            }
        ]
    }
]"#;
pub const ARRAY_WITH_LONG_STRING: &str = r#"[1,2,3,4,5,"helllllllooooooo"]"#;
pub const MIXED_ARRAY_WITH_LONG_STRINGS: &str = r#"[
    "a long string value that includes\nline breaks\nand\ttabs",
    {
        "outer": {
            "inner": [1, 2, {"deep": "value"}]
        }
    },
    [
        {
            "arrayKey": {
                "nestedArray": [true, false, null]
            }
        }
    ]
]"#;
pub const STANDALONE_NULL: &str = "null";
pub const STANDALONE_FALSE: &str = "false";
pub const STANDALONE_TRUE: &str = "true";
pub const STANDALONE_STRING: &str = r#""string""#;
pub const NESTED_OBJECT_SINGLE_KEY: &str = r#"
            {"rust": 
            {"rust": 
            {"rust": 
            {"rust": null
            }
            }
            }
            }   
        "#;
pub const STANDALONE_STRING_WS: &str = r#"      "string"    
        
            "#;

pub const DEEPLY_NESTED: &str = r#"
[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[
"rust is a must"
]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]
"#;
pub const OBJECT_WITH_LONG_KEY_AND_ARR_VAL: &str = r#"
{
    "reallllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllyyy-long-key": ["should expand"]
}
"#;
pub const OBJECT_WITH_EXPANDED_AND_NON_EXPANDED_ARR: &str = r#"
{
    "hi1": ["rust is a must","rust is a must","rust is a must","rust is a must", "rust is a must"],
    "hi2": ["rust is a must","rust is a must","rust is a must", "rust is a must"]
}
"#;
pub const DEEPLY_NESTED_OBJECT_WITH_ARR_VALUES: &str = r#" 
{
    "obj": { "obj": { "obj": { "obj": { "obj": { "obj": { "obj": { "obj": { "obj": { "obj": { "obj": { "obj": { "obj": { "obj": { "obj": { "obj": { "obj": { "obj": {
                    "slkdfjlsdfjsdsldkjflsfljdflksdjksdfljksdfjlkdflksjfdkl": ["hello"]
                } } } } } } } } } } } } } } } } } } }
"#;
pub const STRING_ESCAPED_SOLIDUS: &str = r#""\/""#;
pub const STRING_UNICODE_ESCAPE_PRINTABLE: &str = r#""\u0041""#;
pub const STRING_UNICODE_ESCAPE_CONTROL: &str = r#""\u0000""#;
pub const STRING_SURROGATE_PAIR: &str = r#""\uD83D\uDE00""#;
pub const STRING_UNESCAPED_UNICODE: &str = "\"中文\"";
pub const OBJECT_KEY_UNICODE_ESCAPE: &str = r#"{"\u006bey": "value"}"#;

pub const NEGATIVE_ZERO: &str = "-0";
pub const FLOAT_WITH_TRAILING_ZERO: &str = "1.0";
pub const ZERO: &str = "0";
pub const ZERO_FLOAT: &str = "0.0";
pub const EXPONENT_ZERO: &str = "1e0";
pub const EXPONENT_UPPERCASE_E: &str = "429E6";
pub const EXPONENT_UPPERCASE_E_PLUS: &str = "1E+10";
pub const EXPONENT_LEADING_ZEROS: &str = "10e000010000";

pub const STRING_BACKSPACE_ESCAPE: &str = r#""\b""#;
pub const STRING_FORM_FEED_ESCAPE: &str = r#""\f""#;
pub const STRING_UNICODE_SPACE_ESCAPE: &str = r#""\u0020""#;
pub const STRING_ALL_BASIC_ESCAPES: &str = r#""\"\\\/\n\r\t\b\f""#;

pub const ARRAY_THREE_NEWLINES_BETWEEN_ITEMS: &str = "[1,\n\n\n2,\n\n\n3]";
pub const OBJECT_BLANK_LINES_BETWEEN_KEYS: &str = "{ \"hi\": \"hi\",\n\n\n\"bye\": \"Bye\"\n\n}";
pub const OBJECT_ONLY_NEWLINE: &str = "{\n}";
pub const OBJECT_LEADING_NEWLINE: &str = "{\n\"key\": \"val\"}";

pub const OBJECT_WITH_80_CHAR_STRING_ARRAY_VAL: &str = r#"{"key":["aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"]}"#;

pub const OBJECT_EMPTY_STRING_KEY: &str = r#"{"": "value"}"#;
pub const OBJECT_DUPLICATE_KEYS: &str = r#"{"a": 1, "a": 2}"#;
pub const OBJECT_NUMERIC_STRING_KEY: &str = r#"{"1": "a"}"#;
pub const OBJECT_SINGLE_KEY_ROOT: &str = r#"{"k": "v"}"#;
pub const OBJECT_INLINE_ROOT_80: &str =
    const_format::concatcp!(r#"{"k":""#, str_repeat!("a", 69), r#""}"#);
pub const OBJECT_INLINE_ROOT_83: &str =
    const_format::concatcp!(r#"{"k":""#, str_repeat!("a", 72), r#""}"#);
pub const OBJECT_INLINE_SHORT_ARRAY_VALUE: &str = r#"{"k":[1,2,3]}"#;
pub const OBJECT_INLINE_NESTED_OBJECT_VALUE: &str = r#"{"k":{"a":1},"x":1}"#;
pub const OBJECT_INLINE_UNICODE_VALUE: &str =
    const_format::concatcp!(r#"{"k":""#, str_repeat!("🦀", 17), r#""}"#);
pub const ARRAY_SINGLE_OBJECT: &str = r#"[{"single": "object"}]"#;
pub const ARRAY_INLINE_THREE_SHORT_OBJECTS: &str = r#"[{"k":"v"},{"x":"y"},{"a":"b"}]"#;
pub const ARRAY_EXPANDED_SIX_SHORT_OBJECTS: &str =
    r#"[{"k":"v"},{"x":"y"},{"a":"b"},{"c":"d"},{"e":"f"},{"g":"h"}]"#;
pub const ARRAY_STRING_LONGER_THAN_PRINT_WIDTH: &str =
    r#"["this string is way way way way way way way way way way way way way way too long"]"#;
pub const ARRAY_OVER_80_CHARS: &str =
    const_format::concatcp!(r"[", str_repeat!("1234567890, ", 14), r"1234567890]");

pub const ARRAY_OBJECT_ELEMENT_INLINE_77: &str =
    const_format::concatcp!(r#"[{"k": ""#, str_repeat!("a", 64), r#""}]"#);
pub const ARRAY_OBJECT_ELEMENT_INLINE_80: &str =
    const_format::concatcp!(r#"[{"k": ""#, str_repeat!("a", 67), r#""}]"#);
pub const ARRAY_OBJECT_ELEMENT_INLINE_83: &str =
    const_format::concatcp!(r#"[{"k": ""#, str_repeat!("a", 70), r#""}]"#);
pub const ARRAY_OBJECT_CRAB_EMOJI_INLINE: &str =
    const_format::concatcp!(r#"[{"k": ""#, str_repeat!("🦀", 20), r#""}]"#);

pub const ARRAY_NUMBERS_FILL_MIXED_LENGTHS: &str =
    "[12345,123434,343434,343434,3433434,34343434,34433434,34343434,343443,34433434,344334]";
pub const ARRAY_NUMBERS_WITH_EXPONENTS_FILL: &str =
    "[1e10, 2e-3, 1.5e2, 12345, 1234567, 1e100, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]";
pub const ARRAY_STRINGS_OVER_80: &str = r#"["aaaaaaaa","bbbbbbbb","cccccccc","dddddddd","eeeeeeee","ffffffff","gggggggg","hhhhhhhh","iiiiiiii","jjjjjjjj"]"#;
pub const ARRAY_BOOLS_NULLS_OVER_80: &str = "[true, false, null, true, false, null, true, false, null, true, false, null, true, false, null, true, false, null]";
pub const ARRAY_MIXED_PRIMITIVES_OVER_80: &str = r#"[1, 2, "three", true, null, false, "six", 7.5, 8, "nine", 10, true, 12, null, "fifteen", 16]"#;
pub const ARRAY_NUMBERS_WITH_ONE_STRING_OVER_80: &str = r#"[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, "twenty-five"]"#;
pub const OBJECT_WITH_NESTED_FILL_ARRAY: &str = r#"{"deeply":{"nested":{"key":[12345,123434,343434,343434,3433434,34343434,34433434,34343434,343443,34433434,344334]}}}"#;
pub const ARRAY_NUMERIC_MATRIX_SHORT: &str = "[[1,2,3],[4,5,6]]";
pub const ARRAY_NUMERIC_MATRIX_HETEROGENEOUS: &str =
    r#"[[1, null], [1, null], [null], [0], [false], [""]]"#;

// valid subset of prettier's JSON format test suite
pub const PRETTIER_KEY_VALUE: &str =
    include_str!("../conformance/prettier/tests/format/json/json/key-value.json");
pub const PRETTIER_MULTI_LINE: &str =
    include_str!("../conformance/prettier/tests/format/json/json/multi-line.json");
pub const PRETTIER_SINGLE_LINE: &str =
    include_str!("../conformance/prettier/tests/format/json/json/single-line.json");
pub const PRETTIER_PASS1: &str =
    include_str!("../conformance/prettier/tests/format/json/json/pass1.json");
pub const PRETTIER_ARRAY: &str = "[[1, null], [null], [0], [false], [\"\"]]";

pub const STRING_MIXED_CASE_HEX_UNICODE: &str = r#""\u0123\u4567\u89AB\uCDEF\uabcd\uef4A""#;
pub const EXPONENT_ZERO_PADDED: &str = "1e00";
pub const EXPONENT_ZERO_PADDED_PLUS: &str = "2e+00";
pub const EXPONENT_ZERO_PADDED_MINUS: &str = "2e-00";
pub const ARRAY_BLANK_LINES_AND_EXTRA_WHITESPACE: &str =
    "[1,2 , 3\n\n,\n\n4 , 5        ,          6           ,7        ]";
pub const OBJECT_KEY_COMPLEX_ESCAPES: &str = r#"{"\/\\\"\uCAFE\uBABE\uAB98\uFCDE\ubcda\uef4A\b\f\n\r\t`1~!@#$%^&*()_+-=[]{}|;:',./<>?": "A key can be any string"}"#;

pub const TSCONFIG: &str = r#"
{
  "compilerOptions": {
    "target": "ES6",
    "module": "CommonJS",
    "lib": [
      "ES2020",
      "DOM",
      "DOM.Iterable"
    ],
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "outDir": "./dist",
    "rootDir": "./src",
    "noImplicitAny": true,
    "removeComments": true,
    "preserveConstEnums": true,
    "sourceMap": true
  },
  "include": [
    "src/**/*"
  ],
  "exclude": [
    "node_modules",
    "**/*.spec.ts",
    "**/*.test.ts",
    "dist",
    "temp",
    "examples"
  ]
}
"#;
