use json_parser::tokens::Token;
use json_parser::{Error, ErrorKind};

fn main() {
    println!(
        "{}",
        Error::new(ErrorKind::ExpectedKey(Some(Token::Null)), 0..1, "1\n")
    );

    // println!("{:?}", "".lines().last(),);
}
