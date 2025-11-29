use annotate_snippets::{Renderer, renderer::DecorStyle};
use insta::assert_snapshot;
use jjpwrgem::cli::{Output, run};

#[macro_export]
macro_rules! fixture_tuple {
    ($const:ident) => {
        (stringify!($const), $const)
    };
}

pub fn annotate_and_assert_snapshot(name: &str, json: &str) {
    let json_bytes = json.as_bytes().to_vec();

    let renderer = Renderer::plain().decor_style(DecorStyle::Ascii);
    let annotated = run(json_bytes.clone(), &renderer);

    assert_snapshot!(
        name.to_ascii_lowercase(),
        format_output_snapshot(json_bytes, &annotated)
    );
}

pub fn format_output_snapshot(input: Vec<u8>, output: &Output) -> String {
    let input = String::from_utf8(input.clone()).unwrap_or(format!("<raw bytes>\n{input:?}"));
    format!("case --- \n{input}\n{output:?}")
}
