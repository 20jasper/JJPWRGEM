use annotate_snippets::{Renderer, renderer::DecorStyle};
use jjpwrgem_parse::diagnostics::Diagnostic;

mod json;
mod pretty;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Ansi,
    Plain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Pretty(Color),
    Json,
}

fn renderer(color: Color) -> Renderer {
    match color {
        Color::Ansi => Renderer::styled(),
        Color::Plain => Renderer::plain(),
    }
    .decor_style(DecorStyle::Ascii)
}

impl Style {
    pub fn render_diagnostic(self, diag: Diagnostic<'_>) -> String {
        match self {
            Style::Pretty(color) => renderer(color).render(&pretty::report_diagnostic(diag)),
            Style::Json => json::diagnostic(&diag),
        }
    }

    pub fn render_message(self, m: BasicErrorMessage) -> String {
        match self {
            Style::Pretty(color) => renderer(color).render(&pretty::report_message(m)),
            Style::Json => json::message(m),
        }
    }

    /// Renders a successful run, with the formatted document when there is one.
    pub fn render_success(self, output: Option<String>) -> String {
        match self {
            Style::Pretty(_) => output.unwrap_or_default(),
            Style::Json => json::success(output.as_deref()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BasicErrorMessage {
    pub error: String,
    pub help: Option<String>,
}

/// JSON Schema for the `--reporter json` output.
#[cfg(feature = "schema")]
pub fn reporter_schema() -> schemars::Schema {
    json::schema()
}
