use core::range::Range;
use std::borrow::Cow;

use jjpwrgem_parse::{
    diagnostics::{Diagnostic, Source},
    format::serde::uglify_serializable,
};
use serde::Serialize;

use crate::BasicErrorMessage;

/// Output of `jjp check` and `jjp format` with `--reporter json`.
#[cfg(feature = "schema")]
#[derive(schemars::JsonSchema)]
#[schemars(untagged)]
#[expect(dead_code, reason = "only used to derive the schema")]
enum Reporter<'a> {
    Success(Success<'a>),
    Failure(Failure<'a>),
}

#[cfg(feature = "schema")]
pub(crate) fn schema() -> schemars::Schema {
    schemars::schema_for!(Reporter<'static>)
}

/// The command succeeded.
#[derive(Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct Success<'a> {
    /// Always `true`.
    #[cfg_attr(feature = "schema", schemars(extend("const" = true)))]
    success: bool,
    /// Formatted json, only present for `format`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(with = "Option<String>", extend("type" = "string")))]
    output: Option<&'a str>,
}

/// The command failed.
#[derive(Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct Failure<'a> {
    /// Always `false`.
    #[cfg_attr(feature = "schema", schemars(extend("const" = false)))]
    success: bool,
    /// What went wrong.
    error: ErrorBody<'a>,
}

/// A failure, discriminated by `kind`.
#[derive(Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "lowercase")]
enum ErrorBody<'a> {
    /// A syntax error in the input.
    Diagnostic(DiagnosticBody<'a>),
    /// An error with no position, like missing stdin.
    Message(MessageBody),
}

/// A syntax error in the input.
#[derive(Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct DiagnosticBody<'a> {
    /// Human readable description of the error.
    message: &'a str,
    /// Path of the input file, or `"stdin"` when reading stdin.
    file: Cow<'a, str>,
    /// Where the error is, or null when the error has no position.
    range: Option<JsonRange>,
    /// Related spans that explain the error.
    context: Vec<ContextBody<'a>>,
    /// Suggested edits. Apply `replacement` over `range`.
    fixes: Vec<FixBody<'a>>,
}

/// A related span that explains the error.
#[derive(Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct ContextBody<'a> {
    /// Why this span is relevant.
    message: &'a str,
    /// The related span.
    range: JsonRange,
}

/// A suggested edit.
#[derive(Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct FixBody<'a> {
    /// What the edit does.
    message: &'a str,
    /// The span to replace.
    range: JsonRange,
    /// Text to put in place of `range`.
    replacement: &'a str,
}

/// An error with no position, like missing stdin.
#[derive(Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct MessageBody {
    /// Human readable description of the error.
    message: String,
    /// Suggestion for how to resolve the error, or null when there is none.
    help: Option<String>,
}

/// A span of the input.
#[derive(Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct JsonRange {
    /// Where the span starts.
    start: Position,
    /// Where the span ends.
    end: Position,
}

/// A location in the input.
#[derive(Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
struct Position {
    /// UTF-8 bytes from the start of the input, starting at 0.
    byte_from_start: usize,
    /// 1-based line number.
    #[cfg_attr(feature = "schema", schemars(range(min = 1)))]
    line: usize,
    /// 1-based count of characters (Unicode scalar values) within the line.
    #[cfg_attr(feature = "schema", schemars(range(min = 1)))]
    char_column: usize,
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
        byte_from_start: prefix.len(),
        line: lines.count() + 1,
        char_column: last.iter().filter(|&&b| b & 0xC0 != 0x80).count() + 1,
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
