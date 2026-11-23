use std::collections::HashMap;

use gen_lsp_types::Uri;
use jjpwrgem_parse::{SyntaxOptions, ast::Document, format::FormatSettings};

use crate::{
    lsp_server::req_queue::Outgoing,
    range::PositionEncoding,
    transport::{Sink, Writer},
};

mod diagnostics;
mod dispatch;
mod documents;
mod folding;
mod formatting;
mod lifecycle;
mod log;
mod message;
mod settings;
mod symbols;

const SOURCE: &str = "jjpwrgem";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Lifecycle {
    Uninitialized,
    Running,
    ShutDown,
}

struct OpenDocument {
    parsed: jjpwrgem_parse::Result<Document<String>>,
    syntax: SyntaxOptions,
    warned_formatting: bool,
}

pub struct Backend<S> {
    out: Writer<S>,
    outgoing: Outgoing<()>,
    lifecycle: Lifecycle,
    documents: HashMap<Uri, OpenDocument>,
    position_encoding: PositionEncoding,
    client_pulls_diagnostics: bool,
    format: FormatSettings,
}

impl<S: Sink> Backend<S> {
    pub fn new(out: Writer<S>) -> Self {
        Self {
            out,
            outgoing: Outgoing::new(),
            lifecycle: Lifecycle::Uninitialized,
            documents: HashMap::new(),
            position_encoding: PositionEncoding::Utf16,
            client_pulls_diagnostics: false,
            format: FormatSettings::default(),
        }
    }

    pub fn sink_mut(&mut self) -> &mut S {
        self.out.sink_mut()
    }

    fn with_document<T>(&self, uri: &Uri, f: impl FnOnce(&Document<String>) -> T) -> Option<T> {
        self.documents.get(uri)?.parsed.as_ref().ok().map(f)
    }
}
