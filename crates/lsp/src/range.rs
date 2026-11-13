use line_index::{LineIndex, TextSize, WideEncoding};
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

#[derive(Debug, Clone, Copy, Default)]
pub enum PositionEncoding {
    #[default]
    Utf16,
    Utf8,
}

impl PositionEncoding {
    fn character_count(self, s: &str) -> usize {
        match self {
            Self::Utf8 => s.len(),
            Self::Utf16 => s.chars().map(|c| c.len_utf16()).sum(),
        }
    }
}

pub fn byte_offset_to_position(text: &str, offset: usize, encoding: PositionEncoding) -> Position {
    let before = &text[..offset.min(text.len())];

    let line = u32::try_from(before.bytes().filter(|&b| b == b'\n').count())
        .expect(FILE_TOO_LARGE_PANIC_MSG);
    let character = u32::try_from(
        encoding.character_count(before.rsplit_once('\n').map_or(before, |(_, after)| after)),
    )
    .expect(FILE_TOO_LARGE_PANIC_MSG);
    Position { line, character }
}

pub fn span_to_range(
    text: &str,
    span: std::range::Range<usize>,
    encoding: PositionEncoding,
) -> Range {
    Range {
        start: byte_offset_to_position(text, span.start, encoding),
        end: byte_offset_to_position(text, span.end, encoding),
    }
}

pub struct Positions {
    index: LineIndex,
    encoding: PositionEncoding,
}

impl Positions {
    pub fn new(text: &str, encoding: PositionEncoding) -> Self {
        Self {
            index: LineIndex::new(text),
            encoding,
        }
    }

    pub fn line(&self, offset: usize) -> u32 {
        self.line_col(offset).line
    }

    pub fn range(&self, span: std::range::Range<usize>) -> Range {
        Range {
            start: self.position(span.start),
            end: self.position(span.end),
        }
    }

    pub fn offset(&self, text: &str, position: Position) -> usize {
        let Some(line_range) = self.index.line(position.line) else {
            return text.len();
        };
        let line_start = usize::from(line_range.start());
        let mut line_end = usize::from(line_range.end());
        if line_end > line_start && text.as_bytes()[line_end - 1] == b'\n' {
            line_end -= 1;
            if line_end > line_start && text.as_bytes()[line_end - 1] == b'\r' {
                line_end -= 1;
            }
        }
        let line = &text[line_start..line_end];

        let mut units = 0;
        for (offset, character) in line.char_indices() {
            if units >= position.character as usize {
                return line_start + offset;
            }
            units += match self.encoding {
                PositionEncoding::Utf8 => character.len_utf8(),
                PositionEncoding::Utf16 => character.len_utf16(),
            };
        }

        line_end
    }

    fn line_col(&self, offset: usize) -> line_index::LineCol {
        self.index.line_col(TextSize::new(
            u32::try_from(offset).expect(FILE_TOO_LARGE_PANIC_MSG),
        ))
    }

    fn position(&self, offset: usize) -> Position {
        let utf8 = self.line_col(offset);
        match self.encoding {
            PositionEncoding::Utf8 => Position {
                line: utf8.line,
                character: utf8.col,
            },
            PositionEncoding::Utf16 => {
                let utf16 = self
                    .index
                    .to_wide(WideEncoding::Utf16, utf8)
                    .expect("offset should be on a character boundary");
                Position {
                    line: utf16.line,
                    character: utf16.col,
                }
            }
        }
    }
}

#[cfg(test)]
fn position_to_byte_offset(text: &str, position: Position, encoding: PositionEncoding) -> usize {
    Positions::new(text, encoding).offset(text, position)
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
    // multibyte: é = 2 UTF-8 bytes, 1 UTF-16 code unit
    #[case("hé", 1, 0, 1)]
    #[case("hé", 3, 0, 3)]
    // multibyte: 😀 = 4 UTF-8 bytes, 2 UTF-16 code units
    #[case("a😀b", 1, 0, 1)]
    #[case("a😀b", 5, 0, 5)]
    #[case("a😀b", 6, 0, 6)]
    fn byte_offset_to_position_utf8(
        #[case] text: &str,
        #[case] offset: usize,
        #[case] line: u32,
        #[case] character: u32,
    ) {
        assert_eq!(
            byte_offset_to_position(text, offset, PositionEncoding::Utf8),
            Position { line, character }
        );
    }

    #[rstest]
    #[case("hello", 0, 0, 0)]
    #[case("hello", 3, 0, 3)]
    #[case("hello\nworld", 6, 1, 0)]
    #[case("hello\nworld", 8, 1, 2)]
    #[case("hi", 100, 0, 2)]
    // multibyte: é = 2 UTF-8 bytes, 1 UTF-16 code unit
    #[case("hé", 1, 0, 1)]
    #[case("hé", 3, 0, 2)]
    // multibyte: 😀 = 4 UTF-8 bytes, 2 UTF-16 code units
    #[case("a😀b", 1, 0, 1)]
    #[case("a😀b", 5, 0, 3)]
    #[case("a😀b", 6, 0, 4)]
    fn byte_offset_to_position_utf16(
        #[case] text: &str,
        #[case] offset: usize,
        #[case] line: u32,
        #[case] character: u32,
    ) {
        assert_eq!(
            byte_offset_to_position(text, offset, PositionEncoding::Utf16),
            Position { line, character }
        );
    }

    #[rstest]
    #[case("hello", Position { line: 0, character: 0 }, 0)]
    #[case("hello", Position { line: 0, character: 3 }, 3)]
    #[case("hello\nworld", Position { line: 1, character: 0 }, 6)]
    #[case("hello\nworld", Position { line: 1, character: 2 }, 8)]
    #[case("hi", Position { line: 0, character: 100 }, 2)]
    #[case("hi", Position { line: 100, character: 0 }, 2)]
    // é = 2 UTF-8 bytes
    #[case("hé", Position { line: 0, character: 1 }, 1)]
    #[case("hé", Position { line: 0, character: 2 }, 3)]
    // 😀 = 4 UTF-8 bytes
    #[case("a😀b", Position { line: 0, character: 1 }, 1)]
    #[case("a😀b", Position { line: 0, character: 5 }, 5)]
    #[case("a😀b", Position { line: 0, character: 6 }, 6)]
    fn position_to_byte_offset_utf8(
        #[case] text: &str,
        #[case] position: Position,
        #[case] expected: usize,
    ) {
        assert_eq!(
            position_to_byte_offset(text, position, PositionEncoding::Utf8),
            expected
        );
    }

    #[rstest]
    #[case("hello", Position { line: 0, character: 0 }, 0)]
    #[case("hello", Position { line: 0, character: 3 }, 3)]
    #[case("hello\nworld", Position { line: 1, character: 0 }, 6)]
    #[case("hello\nworld", Position { line: 1, character: 2 }, 8)]
    #[case("hi", Position { line: 0, character: 100 }, 2)]
    #[case("hi", Position { line: 100, character: 0 }, 2)]
    // é = 1 UTF-16 code unit, 2 UTF-8 bytes
    #[case("hé", Position { line: 0, character: 1 }, 1)]
    #[case("hé", Position { line: 0, character: 2 }, 3)]
    // 😀 = 2 UTF-16 code units, 4 UTF-8 bytes
    #[case("a😀b", Position { line: 0, character: 1 }, 1)]
    #[case("a😀b", Position { line: 0, character: 3 }, 5)]
    #[case("a😀b", Position { line: 0, character: 4 }, 6)]
    fn position_to_byte_offset_utf16(
        #[case] text: &str,
        #[case] position: Position,
        #[case] expected: usize,
    ) {
        assert_eq!(
            position_to_byte_offset(text, position, PositionEncoding::Utf16),
            expected
        );
    }
}
