use annotate_snippets::Renderer;
use core::fmt::Debug;
use jjpwrgem_parse::{
    error::diagnostics::{Source, invalid_encoding},
    format,
};
use jjpwrgem_ui::{pretty, report_error};

pub struct Output {
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

impl Debug for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stdout = self.stdout.as_deref().unwrap_or("<empty>");
        let stderr = self.stderr.as_deref().unwrap_or("<empty>");
        write!(f, "stdout --- \n{stdout}\nstderr --- \n{stderr}",)
    }
}

pub fn run(json: Vec<u8>, renderer: &Renderer) -> Output {
    let json = match String::from_utf8(json) {
        Err(_) => {
            return Output {
                stdout: None,
                stderr: Some(renderer.render(&pretty::report(invalid_encoding(Source::Stdin(""))))),
            };
        }
        Ok(s) => s,
    };

    match format::prettify_str(&json) {
        Ok(pretty) => Output {
            stdout: Some(pretty),
            stderr: None,
        },
        Err(error) => Output {
            stdout: None,
            stderr: Some(renderer.render(&report_error(&error))),
        },
    }
}
