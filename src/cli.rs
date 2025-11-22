use core::fmt::Debug;

use annotate_snippets::Renderer;

use crate::format;

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

pub fn run(json: &str, renderer: &Renderer) -> Output {
    match format::prettify_str(json) {
        Ok(pretty) => Output {
            stdout: Some(pretty),
            stderr: None,
        },
        Err(error) => Output {
            stdout: None,
            stderr: Some(renderer.render(&error.report())),
        },
    }
}
