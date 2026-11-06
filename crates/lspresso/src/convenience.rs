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

use gen_lsp_types::{Position, TextDocumentIdentifier, TextDocumentPositionParams, Uri};

fn document(uri: &Uri) -> TextDocumentIdentifier {
    TextDocumentIdentifier { uri: uri.clone() }
}

fn at(uri: &Uri, position: Position) -> TextDocumentPositionParams {
    TextDocumentPositionParams {
        text_document: document(uri),
        position,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_points_into_the_document() {
        let uri: Uri = "file:///a.json".parse().unwrap();
        let position = Position::new(1, 2);
        let params = at(&uri, position);
        assert_eq!(params.text_document.uri, uri);
        assert_eq!(params.position, position);
    }
}
