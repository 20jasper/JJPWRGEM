use jjpwrgem::format::{prettify_str, uglify_str};

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
        res == r#"{"hello hi":null,"by":"hello"}"# || res == r#"{"by":"hello","hello hi":null}"#
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
