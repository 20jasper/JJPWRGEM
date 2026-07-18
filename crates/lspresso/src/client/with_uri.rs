use gen_lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    TextDocumentIdentifier, TextDocumentItem, Uri, VersionedTextDocumentIdentifier,
};

pub(super) trait WithUri {
    fn uri(&self) -> &Uri;
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

with_uri_direct!(TextDocumentIdentifier, TextDocumentItem);

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
