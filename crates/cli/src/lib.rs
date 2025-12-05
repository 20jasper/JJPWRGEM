use core::fmt::Debug;
use jjpwrgem_parse::{
    error::diagnostics::{Diagnostic, Source, invalid_encoding},
    format,
};
use jjpwrgem_ui::Style;
use std::process::ExitCode;

pub struct Output {
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub exit_code: ExitCode,
}

impl Debug for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stdout = self.stdout.as_deref().unwrap_or("<empty>");
        let stderr = self.stderr.as_deref().unwrap_or("<empty>");
        write!(f, "stdout --- \n{stdout}\nstderr --- \n{stderr}",)
    }
}

pub fn run(json: Vec<u8>, style: Style) -> Output {
    let json = match String::from_utf8(json) {
        Err(_) => {
            return Output {
                stdout: None,
                stderr: Some(jjpwrgem_ui::render(
                    invalid_encoding(Source::Stdin("")),
                    style,
                )),
                exit_code: ExitCode::FAILURE,
            };
        }
        Ok(s) => s,
    };

    match format::prettify_str(&json) {
        Ok(pretty) => Output {
            stdout: Some(pretty),
            stderr: None,
            exit_code: ExitCode::SUCCESS,
        },
        Err(error) => Output {
            stdout: None,
            stderr: Some(jjpwrgem_ui::render(Diagnostic::from(&error), style)),
            exit_code: ExitCode::FAILURE,
        },
    }
}
