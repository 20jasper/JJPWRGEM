use jjpwrgem::cli::Output;

#[macro_export]
macro_rules! fixture_tuple {
    ($const:ident) => {
        (stringify!($const), $const)
    };
}

pub fn format_output_snapshot(input: Vec<u8>, output: &Output) -> String {
    let input = String::from_utf8(input.clone()).unwrap_or(format!("<raw bytes>\n{input:?}"));
    format!("case --- \n{input}\n{output:?}")
}
