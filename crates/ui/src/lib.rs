use annotate_snippets::{Renderer, renderer::DecorStyle};
use jjpwrgem_parse::error::diagnostics::Diagnostic;

pub mod pretty;

pub fn render(diag: Diagnostic<'_>, opts: Style) -> String {
    let Style::Pretty(color) = opts;

    match color {
        Color::Ansi => Renderer::styled(),
        Color::Plain => Renderer::plain(),
    }
    .decor_style(DecorStyle::Ascii)
    .render(&pretty::report(diag))
}

pub enum Color {
    Ansi,
    Plain,
}
pub enum Style {
    Pretty(Color),
}
