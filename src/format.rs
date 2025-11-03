use core::iter;

use crate::{
    Result,
    ast::{Value, parse_str},
    tokens::NULL,
};

struct FormatOptions {
    key_val_delimiter: Option<(char, usize)>,
    indent: Option<(char, usize)>,
}

pub fn format_str(json: &str) -> Result<String> {
    Ok(format_value(&parse_str(json)?, 0))
}

pub fn format_value(val: &Value, depth: usize) -> String {
    const SPACES_PER_INDENT: usize = 4;
    let spaces = depth * SPACES_PER_INDENT;
    match val {
        Value::Null => NULL.to_owned(),
        Value::String(s) => format!("\"{s}\""),
        Value::Object(hash_map) => {
            let mut pairs = vec![];
            for (k, val) in hash_map {
                pairs.push(format!(
                    "{}{}: {}",
                    " ".repeat(spaces + SPACES_PER_INDENT),
                    format_value(&Value::String(k.clone()), 0),
                    format_value(val, depth + 1)
                ));
            }
            let lines = iter::once("{".into())
                .chain(iter::once(pairs.join(",\n")))
                .chain(iter::once(format!("{}}}", " ".repeat(spaces))));
            let lines = lines.collect::<Vec<_>>();
            lines.join("\n")
        }
        Value::Boolean(b) => b.to_string(),
    }
}

pub mod uglify {
    use crate::{
        Result,
        ast::{Value, parse_str},
        tokens::NULL,
    };

    pub fn uglify_str(json: &str) -> Result<String> {
        Ok(uglify_value(&parse_str(json)?))
    }

    pub fn uglify_value(val: &Value) -> String {
        match val {
            Value::Null => NULL.to_owned(),
            Value::String(s) => format!("\"{s}\""),
            Value::Object(hash_map) => {
                let mut pairs = vec![];
                for (k, val) in hash_map {
                    pairs.push(format!(
                        "{}:{}",
                        uglify_value(&Value::String(k.clone())),
                        uglify_value(val)
                    ));
                }
                format!("{{{}}}", pairs.join(","))
            }
            Value::Boolean(b) => b.to_string(),
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;

        #[rstest_reuse::template]
        #[rstest::rstest]
        #[case("null")]
        #[case("false")]
        #[case("true")]
        #[case("\"string\"")]
        fn primitive_template(#[case] input: &str) {}

        #[rstest_reuse::apply(primitive_template)]
        fn uglify_primitives_should_stay_the_same(#[case] input: &str) {
            assert_eq!(uglify_str(input).unwrap(), input);
        }

        #[rstest_reuse::apply(primitive_template)]
        fn uglify_removes_whitespace_primitive(#[case] input: &str) {
            let ugly_input = format!(
                r#"      {input}    
        
            "#
            );
            assert_eq!(uglify_str(&ugly_input).unwrap(), input);
        }

        #[test]
        fn uglify_removes_whitespace_object() {
            let input = r#"      {


        "hello hi":                       
        
        
                     null



                     ,


                     "by": "hello"


    }    
        
            "#;
            let res = uglify_str(input).unwrap();
            // we aren't guaranteed a key order
            assert!(
                res == r#"{"hello hi":null,"by":"hello"}"#
                    || res == r#"{"by":"hello","hello hi":null}"#
            );
        }

        #[test]
        fn uglify_arbitrarily_nested() {
            let input = r#"
            {"rust": 
            {"rust": 
            {"rust": 
            {"rust": null
            }
            }
            }
            }   
        "#;

            assert_eq!(
                uglify_str(input).unwrap(),
                r#"{"rust":{"rust":{"rust":{"rust":null}}}}"#
            )
        }
    }
}

pub mod prettify {
    use crate::ast::{Value, parse_str};
    use crate::error::Result;
    use crate::format::format_value;

    pub fn prettify_str(json: &str) -> Result<String> {
        Ok(prettify_value(&parse_str(json)?, 0))
    }

    pub fn prettify_value(val: &Value, depth: usize) -> String {
        format_value(val, depth)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn prettify_arbitrarily_nested() {
            let input = r#"
            {"rust": 
            {"rust": 
            {"rust": 
            {"rust": null
            }
            }
            }
            }   
        "#;
            let expected = r#"{
    "rust": {
        "rust": {
            "rust": {
                "rust": null
            }
        }
    }
}"#;

            assert_eq!(prettify_str(input).unwrap(), expected)
        }
    }
}
