use std::io::Read;

use annotate_snippets::{Renderer, renderer::DecorStyle};
use json_parser::format::prettify_str;

fn main() {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf).unwrap();

    let s = annotate(&buf);
    anstream::println!("{s}");
}

fn annotate(json: &str) -> String {
    match prettify_str(json) {
        Ok(s) => s,
        Err(e) => {
            let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
            renderer.render(&e.report())
        }
    }
}
