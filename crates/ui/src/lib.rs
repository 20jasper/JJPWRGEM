use crate::message::BasicErrorMessage;
use annotate_snippets::{Renderer, renderer::DecorStyle};

pub use jjpwrgem_parse::error::diagnostics::Diagnostic;

mod pretty;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Ansi,
    Plain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Pretty(Color),
}

impl Style {
    fn get_renderer(self: Style) -> Renderer {
        let Style::Pretty(color) = self;

        match color {
            Color::Ansi => Renderer::styled(),
            Color::Plain => Renderer::plain(),
        }
        .decor_style(DecorStyle::Ascii)
    }

    pub fn render_diagnostic(self, diag: Diagnostic<'_>) -> String {
        self.get_renderer().render(&pretty::report_diagnostic(diag))
    }

    pub fn render_message(self, m: BasicErrorMessage) -> String {
        self.get_renderer().render(&pretty::report_message(m))
    }
}

pub mod message {

    #[derive(Debug, PartialEq, Eq, Clone)]
    pub struct BasicErrorMessage {
        pub error: String,
        pub help: Option<String>,
    }

    impl BasicErrorMessage {
        pub fn new(error: impl Into<String>, help: Option<String>) -> Self {
            Self {
                error: error.into(),
                help,
            }
        }
    }
}
