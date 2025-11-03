use crate::{
    Result,
    ast::{Value, parse_str},
    tokens::NULL,
};

pub struct FormatOptions {
    pub key_val_delimiter: Option<(char, usize)>,
    pub indent: Option<(char, usize)>,
    pub eol: Option<(char, usize)>,
}

impl FormatOptions {
    pub fn uglify() -> Self {
        Self {
            key_val_delimiter: None,
            indent: None,
            eol: None,
        }
    }

    pub fn prettify() -> Self {
        Self {
            key_val_delimiter: Some((' ', 1)),
            indent: Some((' ', 4)),
            eol: Some(('\n', 1)),
        }
    }

    fn get(opts: Option<(char, usize)>) -> String {
        if let Some((c, size)) = opts {
            [c].repeat(size).into_iter().collect()
        } else {
            "".into()
        }
    }

    pub fn get_key_val_delimiter(&self) -> String {
        Self::get(self.key_val_delimiter)
    }

    pub fn get_eol(&self) -> String {
        Self::get(self.eol)
    }

    pub fn get_indent(&self, level: usize) -> String {
        Self::get(self.indent.map(|(c, size)| (c, size * level)))
    }
}

pub fn format_str(json: &str, options: &FormatOptions) -> Result<String> {
    Ok(format_value(&parse_str(json)?, options, 0))
}

pub fn format_value(val: &Value, options: &FormatOptions, depth: usize) -> String {
    match val {
        Value::Null => NULL.to_owned(),
        Value::String(s) => format!("\"{s}\""),
        Value::Object(hash_map) => {
            let kv_delim = options.get_key_val_delimiter();
            let key_indent = options.get_indent(depth + 1);
            let eol = options.get_eol();

            let pairs = hash_map
                .iter()
                .map(|(key, val)| {
                    (
                        format_value(&Value::String(key.clone()), options, 0),
                        format_value(val, options, depth + 1),
                    )
                })
                .map(|(key, val)| format!("{key_indent}{key}:{kv_delim}{val}",))
                .collect::<Vec<_>>()
                .join(&format!(",{eol}"));
            [
                "{".into(),
                pairs,
                format!("{}}}", options.get_indent(depth)),
            ]
            .join(&eol)
        }
        Value::Boolean(b) => b.to_string(),
    }
}

pub mod uglify {
    use crate::{
        Result,
        ast::{Value, parse_str},
        format::{FormatOptions, format_value},
    };

    pub fn uglify_str(json: &str) -> Result<String> {
        Ok(uglify_value(&parse_str(json)?))
    }

    pub fn uglify_value(val: &Value) -> String {
        format_value(val, &FormatOptions::uglify(), 0)
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
        Ok(prettify_value(&parse_str(json)?))
    }

    pub fn prettify_value(val: &Value) -> String {
        format_value(val, &FormatOptions::prettify(), 0)
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
