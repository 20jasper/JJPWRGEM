mod ast;
mod error;
mod tokens;

use error::{Error, Result};

use crate::{
    ast::{parse_str, Value},
    tokens::NULL,
};

mod string {
    use core::{iter::Peekable, str::CharIndices};

    pub fn build_str_while<'a>(
        start: usize,
        input: &'a str,
        chars: &mut Peekable<CharIndices<'a>>,
    ) -> &'a str {
        let mut end = start;

        while let Some((i, c)) = chars.next_if(|(_, c)| *c != '"') {
            end = i + c.len_utf8();
        }
        chars.next();

        &input[start..end]
    }
}

pub fn uglify_str(json: &str) -> Result<String> {
    Ok(uglify_value(&parse_str(json)?))
}

pub fn uglify_value(val: &Value) -> String {
    match val {
        ast::Value::Null => NULL.to_owned(),
        ast::Value::String(s) => format!("\"{s}\""),
        ast::Value::Object(hash_map) => {
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
        ast::Value::Boolean(b) => b.to_string(),
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
