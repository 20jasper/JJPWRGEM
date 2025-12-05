use cli::{Output, run};
use jjpwrgem_ui::Color;
use jjpwrgem_ui::Style;
use std::io::Read;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut buf = vec![];
    std::io::stdin()
        .read_to_end(&mut buf)
        .expect("Failed to read from stdin");

    let output = run(buf, Style::Pretty(Color::Plain));
    print_output(&output);

    output.exit_code
}

fn print_output(output: &Output) {
    if let Some(stdout) = &output.stdout {
        anstream::println!("{stdout}");
    }
    if let Some(stderr) = &output.stderr {
        anstream::eprintln!("{stderr}");
    }
}
