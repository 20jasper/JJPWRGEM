use tower_lsp_server::ls_types::*;

pub(crate) const FULL_DOCUMENT_RANGE: Range = Range {
    start: Position {
        line: 0,
        character: 0,
    },
    end: Position {
        line: u32::MAX,
        character: u32::MAX,
    },
};

pub(crate) const FILE_TOO_LARGE_PANIC_MSG: &str = "file over 4GiB; please open an issue at https://github.com/20jasper/jjpwrgem/issues if you'd like this supported";

/// Assumes utf8 encoding
pub fn byte_offset_to_position(text: &str, offset: usize) -> Position {
    let before = &text[..offset.min(text.len())];

    let line = u32::try_from(before.bytes().filter(|&b| b == b'\n').count())
        .expect(FILE_TOO_LARGE_PANIC_MSG);
    let character = u32::try_from(
        before
            .rsplit_once('\n')
            .map_or(before, |(_, after)| after)
            .len(),
    )
    .expect(FILE_TOO_LARGE_PANIC_MSG);
    Position { line, character }
}

pub fn span_to_range(text: &str, span: std::range::Range<usize>) -> Range {
    Range {
        start: byte_offset_to_position(text, span.start),
        end: byte_offset_to_position(text, span.end),
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("hello", 0, 0, 0)]
    #[case("hello", 3, 0, 3)]
    #[case("hello\nworld", 6, 1, 0)]
    #[case("hello\nworld", 8, 1, 2)]
    #[case("hi", 100, 0, 2)]
    #[case("hé", 3, 0, 3)]
    #[case("a😀b", 6, 0, 6)]
    fn byte_offset_to_position_cases(
        #[case] text: &str,
        #[case] offset: usize,
        #[case] line: u32,
        #[case] character: u32,
    ) {
        assert_eq!(
            byte_offset_to_position(text, offset),
            Position { line, character }
        );
    }
}
