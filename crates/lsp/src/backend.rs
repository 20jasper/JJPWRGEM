use std::collections::HashMap;

use gen_lsp_types::Uri;
use jjpwrgem_parse::{ast::Document, format::FormatSettings};

use crate::{lsp_server::req_queue::Outgoing, range::PositionEncoding, transport::Writer};

mod diagnostics;
mod dispatch;
mod documents;
mod folding;
mod formatting;
mod lifecycle;
mod log;
mod settings;
mod symbols;

const SOURCE: &str = "jjpwrgem";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Lifecycle {
    Uninitialized,
    Running,
    ShutDown,
}

pub struct Backend {
    out: Writer,
    outgoing: Outgoing<()>,
    lifecycle: Lifecycle,
    documents: HashMap<Uri, jjpwrgem_parse::Result<Document<String>>>,
    position_encoding: PositionEncoding,
    client_pulls_diagnostics: bool,
    format: FormatSettings,
}

impl Backend {
    pub fn new(out: Writer) -> Self {
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

    fn with_document<T>(&self, uri: &Uri, f: impl FnOnce(&Document<String>) -> T) -> Option<T> {
        self.documents.get(uri)?.as_ref().ok().map(f)
    }
}
