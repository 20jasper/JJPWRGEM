use core::fmt::Debug;
use std::process::ExitCode;

use jjpwrgem_parse::diagnostics::Diagnostic;
use jjpwrgem_ui::Style;

pub struct Output {
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub exit_code: ExitCode,
}

impl Output {
    pub fn success(stdout: impl Into<String>) -> Self {
        Output {
            stdout: Some(stdout.into()),
            stderr: None,
            exit_code: ExitCode::SUCCESS,
        }
    }

    pub fn failure(stderr: impl Into<String>) -> Self {
        Output {
            stdout: None,
            stderr: Some(stderr.into()),
            exit_code: ExitCode::FAILURE,
        }
    }

    pub fn failure_diagnostic(diagnostic: Diagnostic<'_>, style: Style) -> Self {
        Output {
            stdout: None,
            stderr: Some(style.render_diagnostic(diagnostic)),
            exit_code: ExitCode::FAILURE,
        }
    }
}

impl Debug for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stdout = self.stdout.as_deref().unwrap_or("<empty>");
        let stderr = self.stderr.as_deref().unwrap_or("<empty>");
        write!(f, "stdout --- \n{stdout}\nstderr --- \n{stderr}")
    }
}
