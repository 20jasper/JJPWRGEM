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
pub const OBJECT_DOUBLE_OPEN_CURLY: &str = r#"{{"#;
pub const OBJECT_OPEN_CURLY: &str = r#"{"#;
pub const CLOSED_CURLY: &str = r#"}"#;
pub const EMPTY_INPUT: &str = r#""#;
pub const UNEXPECTED_CHARACTER: &str = r#"🦀"#;
pub const UNEXPECTED_ESCAPED_CHARACTER: &str = "\u{B}";
pub const DOUBLE_QUOTE: &str = r#"""#;
pub const OBJECT_WITH_LINE_BREAK_VALUE: &str = "{\"hi\": \"line\nbreak\"}";
pub const OBJECT_WITH_ADJACENT_STRINGS: &str = r#"{"hi": "bye" "ferris": null"#;
pub const OBJECT_EMPTY_THEN_OPEN: &str = r#"{}{"#;
pub const MINUS_SIGN: &str = "-";
pub const LEADING_ZERO_MINUS_SIGN_ZERO: &str = "-000";
pub const LEADING_ZERO_ZERO: &str = "000";
pub const LEADING_ZERO_MINUS_SIGN_NONZERO: &str = "-012";
pub const LEADING_ZERO_NON_ZERO: &str = "012";
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
pub const ARRAY_MULTIPLE_EMPTY_OBJECTS: &str = r#"[{}, {}, {}, {}]"#;
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
pub const MULTIKEY_OBJECT_WITH_LOTS_OF_WHITESPACE: &str = r#"      {

    "hello hi":                       


             null




             ,


             "by": "hello"




    }    
        
        "#;
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
