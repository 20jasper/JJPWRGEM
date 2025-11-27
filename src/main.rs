use std::io::Read;

use annotate_snippets::{Renderer, renderer::DecorStyle};
use jjpwrgem::cli::{Output, run};

fn main() {
    let mut buf = vec![];
    std::io::stdin()
        .read_to_end(&mut buf)
        .expect("Failed to read from stdin");

    let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
    let output = run(buf, &renderer);
    print_output(&output);
}

fn print_output(output: &Output) {
    if let Some(stdout) = &output.stdout {
        anstream::println!("{stdout}");
    }
    if let Some(stderr) = &output.stderr {
        anstream::eprintln!("{stderr}");
    }
}
