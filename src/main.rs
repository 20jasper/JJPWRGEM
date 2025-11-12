use json_parser::format::prettify_str;
// use json_parser::tokens::Token;
// use json_parser::{Error, ErrorKind};

fn main() {
    // println!(
    //     "{}",
    //     Error::new(ErrorKind::ExpectedKey(Some(Token::Null)), 0..1, "1\n")
    // );

    // println!("{:?}", "".lines().last(),);

    println!("{}", prettify_str("\"\u{000B}{}").unwrap_err());
}
