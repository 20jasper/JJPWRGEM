use std::io::Read;

use annotate_snippets::{Renderer, renderer::DecorStyle};
use jjpwrgem::cli::run;

fn main() {
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .expect("Failed to read from stdin");

    let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
    let s = run(&buf, &renderer);
    anstream::println!("{s}");
}
