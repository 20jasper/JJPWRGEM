use core::range::Range;
use std::borrow::Cow;

use jjpwrgem_parse::{
    diagnostics::{Diagnostic, Source},
    format::serde::uglify_serializable,
};
use serde::Serialize;

use crate::BasicErrorMessage;

#[derive(Serialize)]
struct Success<'a> {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    output: Option<&'a str>,
}

#[derive(Serialize)]
struct Failure<'a> {
    success: bool,
    error: ErrorBody<'a>,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum ErrorBody<'a> {
    Diagnostic(DiagnosticBody<'a>),
    Message(MessageBody),
}

#[derive(Serialize)]
struct DiagnosticBody<'a> {
    message: &'a str,
    file: Cow<'a, str>,
    range: Option<JsonRange>,
    context: Vec<ContextBody<'a>>,
    fixes: Vec<FixBody<'a>>,
}

#[derive(Serialize)]
struct ContextBody<'a> {
    message: &'a str,
    range: JsonRange,
}

#[derive(Serialize)]
struct FixBody<'a> {
    message: &'a str,
    range: JsonRange,
    replacement: &'a str,
}

#[derive(Serialize)]
struct MessageBody {
    message: String,
    help: Option<String>,
}

#[derive(Serialize)]
struct JsonRange {
    start: Position,
    end: Position,
}

#[derive(Serialize)]
struct Position {
    offset: usize,
    line: usize,
    column: usize,
}

fn source_text(source: Source<'_>) -> &str {
    match source {
        Source::Stdin(source) | Source::File { source, .. } => source,
    }
}

fn file(source: Source<'_>) -> Cow<'_, str> {
    match source {
        Source::Stdin(_) => Cow::Borrowed("stdin"),
        Source::File { path, .. } => path.to_string_lossy(),
    }
}

/// Line and column are 1-based, and the column counts chars.
fn position(source: &str, offset: usize) -> Position {
    let prefix = &source.as_bytes()[..offset.min(source.len())];
    let mut lines = prefix.split(|&b| b == b'\n');
    let last = lines.next_back().unwrap_or_default();
    Position {
        offset: prefix.len(),
        line: lines.count() + 1,
        column: last.iter().filter(|&&b| b & 0xC0 != 0x80).count() + 1,
    }
}

fn range(source: Source<'_>, span: Range<usize>) -> JsonRange {
    let text = source_text(source);
    JsonRange {
        start: position(text, span.start),
        end: position(text, span.end),
    }
}

fn to_json(value: impl Serialize) -> String {
    uglify_serializable(value).expect("ui structs should always serialize")
}

pub(crate) fn success(output: Option<&str>) -> String {
    to_json(Success {
        success: true,
        output,
    })
}

pub(crate) fn diagnostic(diag: &Diagnostic<'_>) -> String {
    let source = diag.source;
    to_json(Failure {
        success: false,
        error: ErrorBody::Diagnostic(DiagnosticBody {
            message: &diag.message,
            file: file(source),
            range: diag.range.map(|span| range(source, span)),
            context: diag
                .context
                .iter()
                .map(|ctx| ContextBody {
                    message: &ctx.message,
                    range: range(ctx.source, ctx.span),
                })
                .collect(),
            fixes: diag
                .patches
                .iter()
                .map(|patch| FixBody {
                    message: &patch.message,
                    range: range(patch.source, patch.span),
                    replacement: &patch.replacement,
                })
                .collect(),
        }),
    })
}

pub(crate) fn message(message: BasicErrorMessage) -> String {
    to_json(Failure {
        success: false,
        error: ErrorBody::Message(MessageBody {
            message: message.error,
            help: message.help,
        }),
    })
}
