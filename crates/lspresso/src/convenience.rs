mod capabilities;
mod decorations;
mod diagnostics;
mod documents;
mod editing;
mod formatting;
mod general;
mod hierarchy;
mod navigation;
mod symbols;
mod workspace;

use gen_lsp_types::*;

fn document(uri: &Uri) -> TextDocumentIdentifier {
    TextDocumentIdentifier { uri: uri.clone() }
}

fn at(uri: &Uri, position: Position) -> TextDocumentPositionParams {
    TextDocumentPositionParams {
        text_document: document(uri),
        position,
    }
}
