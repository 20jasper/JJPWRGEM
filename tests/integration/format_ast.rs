use insta::assert_snapshot;
use jjpwrgem_parse::{ast::Document, format::uglify_document};

use crate::{common::format_template, test_json::*};

#[rstest_reuse::apply(format_template)]
fn uglify(#[case] (name, input): (&str, &str)) {
    let doc = Document::parse(input).unwrap();
    let out = uglify_document(&doc);

    assert_snapshot!(format!("uglify_{name}"), out);
}
