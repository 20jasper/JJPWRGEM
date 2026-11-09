use gen_lsp_types::{
    Position, PositionEncodingKind, Range, TextDocumentContentChangeEvent,
    TextDocumentContentChangePartial, TextEdit,
};

use crate::error::ErrorKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Encoding {
    Utf8,
    Utf16,
    Utf32,
}

impl Encoding {
    /// UTF-16 unless the server picked another, as the spec defaults.
    pub(crate) fn negotiated(kind: Option<&PositionEncodingKind>) -> Self {
        match kind {
            Some(kind) if *kind == PositionEncodingKind::UTF8 => Self::Utf8,
            Some(kind) if *kind == PositionEncodingKind::UTF32 => Self::Utf32,
            _ => Self::Utf16,
        }
    }

    fn units(self, c: char) -> u32 {
        match self {
            Self::Utf8 => u32::try_from(c.len_utf8()).unwrap(),
            Self::Utf16 => u32::try_from(c.len_utf16()).unwrap(),
            Self::Utf32 => 1,
        }
    }
}

/// Where the line starting at `from` ends, and where the next one starts:
/// lines end at `\n`, `\r\n` or `\r`.
fn line_end(text: &str, from: usize) -> Option<(usize, usize)> {
    let end = from + text[from..].find(['\n', '\r'])?;
    let terminator = if text[end..].starts_with("\r\n") {
        2
    } else {
        1
    };
    Some((end, end + terminator))
}

/// The byte offset of `position`, reading a line past the last as the end
/// of the document and a character past its line's end as the end of the
/// line, as editors do. `None` if it falls inside a character.
pub(crate) fn offset(text: &str, position: Position, encoding: Encoding) -> Option<usize> {
    let mut start = 0;
    for _ in 0..position.line {
        match line_end(text, start) {
            Some((_, next)) => start = next,
            None => return Some(text.len()),
        }
    }
    let end = line_end(text, start).map_or(text.len(), |(end, _)| end);
    let mut units = 0;
    for (index, c) in text[start..end].char_indices() {
        if units == position.character {
            return Some(start + index);
        }
        units += encoding.units(c);
        if units > position.character {
            return None;
        }
    }
    Some(end)
}

/// The position of byte `offset`, which must be a char boundary outside any
/// line terminator.
pub(crate) fn position(text: &str, offset: usize, encoding: Encoding) -> Position {
    let mut line = 0;
    let mut start = 0;
    while let Some((_, next)) = line_end(text, start)
        && next <= offset
    {
        line += 1;
        start = next;
    }
    let character = text[start..offset].chars().map(|c| encoding.units(c)).sum();
    Position::new(line, character)
}

#[derive(Debug)]
pub(crate) struct Applied {
    pub(crate) text: String,
    /// The same edits as changes applied in turn, as incremental sync sends
    /// them.
    pub(crate) changes: Vec<TextDocumentContentChangeEvent>,
}

/// Applies `edits` to `text` as the spec says a set of `TextEdit`s applies,
/// each against the text as it is. Edits may not overlap, though they may
/// touch, and inserts at the same place land in order; an insert inside a
/// replaced range overlaps it.
pub(crate) fn apply(text: &str, edits: &[TextEdit], encoding: Encoding) -> crate::Result<Applied> {
    let mut spans = edits
        .iter()
        .enumerate()
        .map(|(index, edit)| {
            match (
                offset(text, edit.range.start, encoding),
                offset(text, edit.range.end, encoding),
            ) {
                (Some(start), Some(end)) if start <= end => Ok((start, end, index)),
                _ => Err(ErrorKind::InvalidEdit {
                    index,
                    range: edit.range,
                }
                .into()),
            }
        })
        .collect::<crate::Result<Vec<_>>>()?;
    spans.sort_unstable();

    let mut reach: Option<(usize, usize)> = None;
    for &(start, end, index) in &spans {
        if let Some((reach_end, owner)) = reach
            && start < reach_end
        {
            return Err(ErrorKind::OverlappingEdits {
                a: owner.min(index),
                b: owner.max(index),
            }
            .into());
        }
        if start < end && reach.is_none_or(|(reach_end, _)| end > reach_end) {
            reach = Some((end, index));
        }
    }

    let mut result = text.to_owned();
    let mut changes = Vec::with_capacity(spans.len());
    let mut representable = true;
    for &(start, end, index) in spans.iter().rev() {
        representable &= !splits_line_break(&result, start) && !splits_line_break(&result, end);
        let new_text = &edits[index].new_text;
        changes.push(partial_change(
            &result,
            start,
            end,
            new_text.clone(),
            encoding,
        ));
        result.replace_range(start..end, new_text);
    }
    // A change can't point between \r and \n. If applying the edits in turn
    // would put a boundary there, send one change covering the edited range.
    if !representable && let Some(&(start, ..)) = spans.first() {
        let end = spans.iter().map(|&(_, end, _)| end).max().unwrap_or(start);
        let replaced = result[start..result.len() - (text.len() - end)].to_owned();
        changes = vec![partial_change(text, start, end, replaced, encoding)];
    }
    Ok(Applied {
        text: result,
        changes,
    })
}

/// Applies the content changes a `didChange` carries, in order, the way a
/// server applies them.
pub(crate) fn apply_changes(
    text: &str,
    changes: &[TextDocumentContentChangeEvent],
    encoding: Encoding,
) -> crate::Result<String> {
    let mut result = text.to_owned();
    for (index, change) in changes.iter().enumerate() {
        match change {
            TextDocumentContentChangeEvent::TextDocumentContentChangeWholeDocument(whole) => {
                if changes.len() != 1 {
                    return Err(ErrorKind::InvalidContentChange { index }.into());
                }
                result.clone_from(&whole.text);
            }
            TextDocumentContentChangeEvent::TextDocumentContentChangePartial(partial) => {
                let Some(start) = offset(&result, partial.range.start, encoding) else {
                    return Err(ErrorKind::InvalidEdit {
                        index,
                        range: partial.range,
                    }
                    .into());
                };
                let Some(end) = offset(&result, partial.range.end, encoding) else {
                    return Err(ErrorKind::InvalidEdit {
                        index,
                        range: partial.range,
                    }
                    .into());
                };
                if start > end {
                    return Err(ErrorKind::InvalidEdit {
                        index,
                        range: partial.range,
                    }
                    .into());
                }
                result.replace_range(start..end, &partial.text);
            }
        }
    }
    Ok(result)
}

/// Whether `offset` falls between a `\r` and a `\n`, where no position can
/// point: one change can make that of where the next one starts, by
/// inserting a `\n` after a lone `\r`.
fn splits_line_break(text: &str, offset: usize) -> bool {
    text[..offset].ends_with('\r') && text[offset..].starts_with('\n')
}

fn partial_change(
    text: &str,
    start: usize,
    end: usize,
    new_text: String,
    encoding: Encoding,
) -> TextDocumentContentChangeEvent {
    TextDocumentContentChangePartial::new(
        Range::new(
            position(text, start, encoding),
            position(text, end, encoding),
        ),
        None,
        new_text,
    )
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(line: u32, character: u32) -> Position {
        Position::new(line, character)
    }

    fn edit(start: (u32, u32), end: (u32, u32), new_text: &str) -> TextEdit {
        TextEdit {
            range: Range::new(at(start.0, start.1), at(end.0, end.1)),
            new_text: new_text.to_owned(),
        }
    }

    fn whole(text: &str) -> TextDocumentContentChangeEvent {
        gen_lsp_types::TextDocumentContentChangeWholeDocument {
            text: text.to_owned(),
        }
        .into()
    }

    fn partial(start: (u32, u32), end: (u32, u32), text: &str) -> TextDocumentContentChangeEvent {
        TextDocumentContentChangePartial::new(
            Range::new(at(start.0, start.1), at(end.0, end.1)),
            None,
            text.to_owned(),
        )
        .into()
    }

    #[test]
    fn offsets_count_in_the_encoding_and_clamp_past_the_end() {
        let text = "a😀b\r\nc\rd";
        assert_eq!(offset(text, at(0, 3), Encoding::Utf16), Some(5));
        assert_eq!(offset(text, at(0, 2), Encoding::Utf16), None);
        assert_eq!(offset(text, at(0, 5), Encoding::Utf8), Some(5));
        assert_eq!(offset(text, at(0, 3), Encoding::Utf8), None);
        assert_eq!(offset(text, at(0, 2), Encoding::Utf32), Some(5));
        assert_eq!(offset(text, at(0, 99), Encoding::Utf16), Some(6));
        assert_eq!(offset(text, at(1, 0), Encoding::Utf16), Some(8));
        assert_eq!(offset(text, at(2, 1), Encoding::Utf16), Some(11));
        assert_eq!(offset(text, at(9, 0), Encoding::Utf16), Some(text.len()));
        assert_eq!(
            offset(text, at(u32::MAX, u32::MAX), Encoding::Utf8),
            Some(text.len())
        );
    }

    #[test]
    fn position_and_offset_round_trip() {
        for text in ["", "a", "a😀b\r\nc\rd", "😀", "\r\nx"] {
            let boundaries: Vec<usize> = text
                .char_indices()
                .map(|(index, _)| index)
                .chain([text.len()])
                .filter(|&index| !splits_line_break(text, index))
                .collect();
            for encoding in [Encoding::Utf8, Encoding::Utf16, Encoding::Utf32] {
                for &boundary in &boundaries {
                    let position = position(text, boundary, encoding);
                    assert_eq!(offset(text, position, encoding), Some(boundary));
                }
            }
        }
    }

    #[test]
    fn edits_apply_against_the_original_text() {
        let applied = apply(
            "ab\ncd",
            &[edit((1, 0), (1, 2), "x"), edit((0, 0), (0, 1), "yy")],
            Encoding::Utf16,
        )
        .unwrap();
        assert_eq!(applied.text, "yyb\nx");
    }

    #[test]
    fn inserts_at_one_place_land_in_order_before_a_replacement_there() {
        let applied = apply(
            "abc",
            &[
                edit((0, 1), (0, 2), "R"),
                edit((0, 1), (0, 1), "1"),
                edit((0, 1), (0, 1), "2"),
            ],
            Encoding::Utf16,
        )
        .unwrap();
        assert_eq!(applied.text, "a12Rc");
    }

    #[test]
    fn changes_that_would_join_a_line_break_become_one() {
        let applied = apply(
            "\ra",
            &[edit((1, 0), (1, 0), "x"), edit((1, 0), (1, 0), "\n")],
            Encoding::Utf16,
        )
        .unwrap();
        assert_eq!(applied.text, "\rx\na");
        assert_eq!(
            applied.changes,
            [TextDocumentContentChangePartial::new(
                Range::new(at(1, 0), at(1, 0)),
                None,
                "x\n".to_owned()
            )
            .into()]
        );
        assert_eq!(
            apply_changes("\ra", &applied.changes, Encoding::Utf16).unwrap(),
            applied.text
        );
    }

    #[test]
    fn overlapping_or_invalid_edits_are_rejected() {
        let overlapping = [
            vec![edit((0, 0), (0, 2), ""), edit((0, 1), (0, 3), "")],
            vec![edit((0, 0), (0, 2), ""), edit((0, 1), (0, 1), "x")],
            vec![edit((0, 0), (0, 3), ""), edit((0, 1), (0, 2), "")],
        ];
        for edits in overlapping {
            let err = apply("abc", &edits, Encoding::Utf16).unwrap_err();
            assert!(
                matches!(err.kind(), ErrorKind::OverlappingEdits { a: 0, b: 1 }),
                "{err}"
            );
        }
        let touching = [edit((0, 0), (0, 1), "x"), edit((0, 1), (0, 2), "y")];
        assert_eq!(
            apply("abc", &touching, Encoding::Utf16).unwrap().text,
            "xyc"
        );

        for range in [((0, 2), (0, 2)), ((0, 3), (0, 1))] {
            let err = apply("a😀b", &[edit(range.0, range.1, "")], Encoding::Utf16).unwrap_err();
            assert!(
                matches!(err.kind(), ErrorKind::InvalidEdit { index: 0, .. }),
                "{err}"
            );
        }
    }

    #[test]
    fn applied_changes_replay_to_the_same_text() {
        let edits = [edit((0, 9), (0, 10), "10"), edit((1, 7), (1, 7), " ")];
        let text = "{\"😀\": 1,\n \"b\": 2}";
        let applied = apply(text, &edits, Encoding::Utf8).unwrap();
        assert_eq!(
            apply_changes(text, &applied.changes, Encoding::Utf8).unwrap(),
            applied.text
        );
    }

    #[test]
    fn whole_and_partial_content_changes_apply_in_order() {
        assert_eq!(
            apply_changes("abc", &[whole("xyz")], Encoding::Utf16).unwrap(),
            "xyz"
        );
        assert_eq!(
            apply_changes(
                "abc",
                &[partial((0, 1), (0, 2), "X"), partial((0, 1), (0, 1), "Y")],
                Encoding::Utf16,
            )
            .unwrap(),
            "aYXc"
        );
    }

    #[test]
    fn a_whole_document_change_mixed_with_others_is_rejected() {
        let err = apply_changes(
            "abc",
            &[whole("xyz"), partial((0, 0), (0, 1), "a")],
            Encoding::Utf16,
        )
        .unwrap_err();
        assert!(
            matches!(err.kind(), ErrorKind::InvalidContentChange { index: 0 }),
            "{err}"
        );
    }
}
