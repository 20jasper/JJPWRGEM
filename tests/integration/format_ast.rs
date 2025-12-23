use crate::common::format_template;
use crate::test_json::*;
use insta::assert_snapshot;
use jjpwrgem_parse::{ast::parse_str, format::uglify_value};

#[rstest_reuse::apply(format_template)]
fn uglify(#[case] (name, input): (&str, &str)) {
    let val = parse_str(input).unwrap();
    let out = uglify_value(&val);

    assert_snapshot!(format!("uglify_{name}"), out);
}
