use core::iter;

use crate::{
    Result,
    ast::{Value, parse_str},
    tokens::NULL,
};

pub struct FormatOptions {
    key_val_delimiter: Option<(char, usize)>,
    indent: Option<(char, usize)>,
    eol: Option<(char, usize)>,
}

impl FormatOptions {
    fn uglify() -> Self {
        Self {
            key_val_delimiter: None,
            indent: None,
            eol: None,
        }
    }

    fn prettify() -> Self {
        Self {
            key_val_delimiter: Some((' ', 1)),
            indent: Some((' ', 4)),
            eol: Some(('\n', 1)),
        }
    }

    fn get_key_val_delimiter(&self) -> String {
        if let Some((c, size)) = self.key_val_delimiter {
            [c].repeat(size).into_iter().collect()
        } else {
            "".into()
        }
    }

    fn get_indent(&self, level: usize) -> String {
        if let Some((c, size)) = self.indent {
            [c].repeat(size * level).into_iter().collect()
        } else {
            "".into()
        }
    }
}

pub fn format_str(json: &str, options: &FormatOptions) -> Result<String> {
    Ok(format_value(&parse_str(json)?, options, 0))
}

pub fn format_value(val: &Value, options: &FormatOptions, depth: usize) -> String {
    let kv_delim = options.get_key_val_delimiter();

    match val {
        Value::Null => NULL.to_owned(),
        Value::String(s) => format!("\"{s}\""),
        Value::Object(hash_map) => {
            let mut pairs = vec![];
            for (k, val) in hash_map {
                let indent = options.get_indent(depth + 1);
                pairs.push(format!(
                    "{indent}{}:{kv_delim}{}",
                    format_value(&Value::String(k.clone()), options, 0),
                    format_value(val, options, depth + 1)
                ));
            }
            let lines = iter::once("{".into())
                .chain(iter::once(pairs.join(",\n")))
                .chain(iter::once(format!("{}}}", options.get_indent(depth))));
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
    use crate::format::{FormatOptions, format_value};

    pub fn prettify_str(json: &str) -> Result<String> {
        Ok(prettify_value(&parse_str(json)?, 0))
    }

    pub fn prettify_value(val: &Value, depth: usize) -> String {
        format_value(val, &FormatOptions::prettify(), depth)
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
