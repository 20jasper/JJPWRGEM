use gen_lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    PublishDiagnosticsParams, TextDocumentIdentifier, TextDocumentItem, Uri,
    VersionedTextDocumentIdentifier,
};

pub(super) trait WithUri {
    fn uri(&self) -> &Uri;
}

impl WithUri for Uri {
    fn uri(&self) -> &Uri {
        self
    }
}

pub(super) fn for_uri<'a, T: WithUri>(
    uri: &'a Uri,
    mut predicate: impl FnMut(&T) -> bool + 'a,
) -> impl FnMut(&T) -> bool + 'a {
    move |p| p.uri() == uri && predicate(p)
}

impl WithUri for VersionedTextDocumentIdentifier {
    fn uri(&self) -> &Uri {
        self.text_document_identifier.uri()
    }
}

macro_rules! with_uri_direct {
    ($($t:ty),+ $(,)?) => {
        $(
            impl WithUri for $t {
                fn uri(&self) -> &Uri {
                    &self.uri
                }
            }
        )+
    };
}

with_uri_direct!(
    TextDocumentIdentifier,
    TextDocumentItem,
    PublishDiagnosticsParams
);

macro_rules! with_uri_via_text_document {
    ($($t:ty),+ $(,)?) => {
        $(
            impl WithUri for $t {
                fn uri(&self) -> &Uri {
                    self.text_document.uri()
                }
            }
        )+
    };
}

with_uri_via_text_document!(
    DidOpenTextDocumentParams,
    DidChangeTextDocumentParams,
    DidCloseTextDocumentParams,
);
