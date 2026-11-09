use std::ops::Range as ByteRange;

use gen_lsp_types::*;

use crate::error::ErrorKind;

pub(crate) fn negotiated(kind: Option<&PositionEncodingKind>) -> PositionEncodingKind {
    match kind {
        Some(PositionEncodingKind::UTF8) => PositionEncodingKind::UTF8,
        Some(PositionEncodingKind::UTF32) => PositionEncodingKind::UTF32,
        _ => PositionEncodingKind::UTF16,
    }
}

fn units(encoding: &PositionEncodingKind, c: char) -> u32 {
    match encoding {
        PositionEncodingKind::UTF8 => u32::try_from(c.len_utf8()).unwrap(),
        PositionEncodingKind::UTF16 => u32::try_from(c.len_utf16()).unwrap(),
        _ => 1,
    }
}

fn span(text: &str, range: Range, encoding: &PositionEncodingKind) -> Option<ByteRange<usize>> {
    let start = offset(text, range.start, encoding)?;
    let end = offset(text, range.end, encoding)?;
    (start <= end).then_some(start..end)
}

fn line_end(text: &str, from: usize) -> Option<(usize, usize)> {
    let end = from + text[from..].find(['\n', '\r'])?;
    let terminator = if text[end..].starts_with("\r\n") {
        2
    } else {
        1
    };
    Some((end, end + terminator))
}

/// Clamps past the end; `None` inside a character.
fn offset(text: &str, position: Position, encoding: &PositionEncodingKind) -> Option<usize> {
    let mut start = 0;
    for _ in 0..position.line {
        match line_end(text, start) {
            Some((_, next)) => start = next,
            None => return Some(text.len()),
        }
    }
    let end = line_end(text, start).map_or(text.len(), |(end, _)| end);
    let mut count = 0;
    for (index, c) in text[start..end].char_indices() {
        if count == position.character {
            return Some(start + index);
        }
        count += units(encoding, c);
        if count > position.character {
            return None;
        }
    }
    Some(end)
}

/// `offset` must be a char boundary outside any line terminator.
fn position(text: &str, offset: usize, encoding: &PositionEncodingKind) -> Position {
    let mut line = 0;
    let mut start = 0;
    while let Some((_, next)) = line_end(text, start)
        && next <= offset
    {
        line += 1;
        start = next;
    }
    let character = text[start..offset]
        .chars()
        .map(|c| units(encoding, c))
        .sum();
    Position::new(line, character)
}

#[derive(Debug)]
pub(crate) struct Applied {
    pub(crate) text: String,
    pub(crate) changes: Vec<TextDocumentContentChangeEvent>,
}

pub(crate) fn apply(
    text: &str,
    edits: &[TextEdit],
    encoding: &PositionEncodingKind,
) -> crate::Result<Applied> {
    let mut spans = edits
        .iter()
        .enumerate()
        .map(|(index, edit)| {
            span(text, edit.range, encoding)
                .map(|range| (range, index))
                .ok_or_else(|| {
                    ErrorKind::InvalidEdit {
                        index,
                        range: edit.range,
                    }
                    .into()
                })
        })
        .collect::<crate::Result<Vec<_>>>()?;
    spans.sort_unstable_by_key(|(range, index)| (range.start, range.end, *index));

    let mut reach: Option<(usize, usize)> = None;
    for (range, index) in &spans {
        if let Some((reach_end, owner)) = reach
            && range.start < reach_end
        {
            return Err(ErrorKind::OverlappingEdits {
                a: owner.min(*index),
                b: owner.max(*index),
            }
            .into());
        }
        if range.start < range.end && reach.is_none_or(|(reach_end, _)| range.end > reach_end) {
            reach = Some((range.end, *index));
        }
    }

    let mut result = String::with_capacity(text.len());
    let mut last = 0;
    for (range, index) in &spans {
        result.push_str(&text[last..range.start]);
        result.push_str(&edits[*index].new_text);
        last = range.end;
    }
    result.push_str(&text[last..]);

    let changes = match spans.first() {
        Some((first, _)) => {
            let start = first.start;
            let end = spans
                .iter()
                .map(|(range, _)| range.end)
                .max()
                .unwrap_or(start);
            let replaced = result[start..result.len() - (text.len() - end)].to_owned();
            vec![partial_change(text, start, end, replaced, encoding)]
        }
        None => Vec::new(),
    };
    Ok(Applied {
        text: result,
        changes,
    })
}

pub(crate) fn apply_changes(
    text: &str,
    changes: &[TextDocumentContentChangeEvent],
    encoding: &PositionEncodingKind,
) -> crate::Result<String> {
    let mut result = text.to_owned();
    for (index, change) in changes.iter().enumerate() {
        match change {
            TextDocumentContentChangeEvent::TextDocumentContentChangeWholeDocument(whole) => {
                result.clone_from(&whole.text);
            }
            TextDocumentContentChangeEvent::TextDocumentContentChangePartial(partial) => {
                let Some(range) = span(&result, partial.range, encoding) else {
                    return Err(ErrorKind::InvalidEdit {
                        index,
                        range: partial.range,
                    }
                    .into());
                };
                result.replace_range(range, &partial.text);
            }
        }
    }
    Ok(result)
}

fn partial_change(
    text: &str,
    start: usize,
    end: usize,
    new_text: String,
    encoding: &PositionEncodingKind,
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
    fn encoding_defaults_to_utf16() {
        assert_eq!(negotiated(None), PositionEncodingKind::UTF16);
        assert_eq!(
            negotiated(Some(&PositionEncodingKind::UTF8)),
            PositionEncodingKind::UTF8
        );
        assert_eq!(
            negotiated(Some(&PositionEncodingKind::UTF32)),
            PositionEncodingKind::UTF32
        );
        assert_eq!(
            negotiated(Some(&PositionEncodingKind::new("custom"))),
            PositionEncodingKind::UTF16
        );
    }

    #[test]
    fn offsets_count_in_the_encoding_and_clamp_past_the_end() {
        let text = "a😀b\r\nc\rd";
        assert_eq!(
            offset(text, at(0, 3), &PositionEncodingKind::UTF16),
            Some(5)
        );
        assert_eq!(offset(text, at(0, 2), &PositionEncodingKind::UTF16), None);
        assert_eq!(offset(text, at(0, 5), &PositionEncodingKind::UTF8), Some(5));
        assert_eq!(offset(text, at(0, 3), &PositionEncodingKind::UTF8), None);
        assert_eq!(
            offset(text, at(0, 2), &PositionEncodingKind::UTF32),
            Some(5)
        );
        assert_eq!(
            offset(text, at(0, 99), &PositionEncodingKind::UTF16),
            Some(6)
        );
        assert_eq!(
            offset(text, at(1, 0), &PositionEncodingKind::UTF16),
            Some(8)
        );
        assert_eq!(
            offset(text, at(2, 1), &PositionEncodingKind::UTF16),
            Some(11)
        );
        assert_eq!(
            offset(text, at(9, 0), &PositionEncodingKind::UTF16),
            Some(text.len())
        );
        assert_eq!(
            offset(text, at(u32::MAX, u32::MAX), &PositionEncodingKind::UTF8),
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
                .filter(|&index| {
                    !(text[..index].ends_with('\r') && text[index..].starts_with('\n'))
                })
                .collect();
            for encoding in [
                &PositionEncodingKind::UTF8,
                &PositionEncodingKind::UTF16,
                &PositionEncodingKind::UTF32,
            ] {
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
            &PositionEncodingKind::UTF16,
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
            &PositionEncodingKind::UTF16,
        )
        .unwrap();
        assert_eq!(applied.text, "a12Rc");
    }

    #[test]
    fn changes_that_would_join_a_line_break_become_one() {
        let applied = apply(
            "\ra",
            &[edit((1, 0), (1, 0), "x"), edit((1, 0), (1, 0), "\n")],
            &PositionEncodingKind::UTF16,
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
            apply_changes("\ra", &applied.changes, &PositionEncodingKind::UTF16).unwrap(),
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
            let err = apply("abc", &edits, &PositionEncodingKind::UTF16).unwrap_err();
            assert!(
                matches!(err.kind(), ErrorKind::OverlappingEdits { a: 0, b: 1 }),
                "{err}"
            );
        }
        let touching = [edit((0, 0), (0, 1), "x"), edit((0, 1), (0, 2), "y")];
        assert_eq!(
            apply("abc", &touching, &PositionEncodingKind::UTF16)
                .unwrap()
                .text,
            "xyc"
        );

        for range in [((0, 2), (0, 2)), ((0, 3), (0, 1))] {
            let err = apply(
                "a😀b",
                &[edit(range.0, range.1, "")],
                &PositionEncodingKind::UTF16,
            )
            .unwrap_err();
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
        let applied = apply(text, &edits, &PositionEncodingKind::UTF8).unwrap();
        assert_eq!(
            apply_changes(text, &applied.changes, &PositionEncodingKind::UTF8).unwrap(),
            applied.text
        );
    }

    #[test]
    fn whole_and_partial_content_changes_apply_in_order() {
        assert_eq!(
            apply_changes("abc", &[whole("xyz")], &PositionEncodingKind::UTF16).unwrap(),
            "xyz"
        );
        assert_eq!(
            apply_changes(
                "abc",
                &[partial((0, 1), (0, 2), "X"), partial((0, 1), (0, 1), "Y")],
                &PositionEncodingKind::UTF16,
            )
            .unwrap(),
            "aYXc"
        );
        assert_eq!(
            apply_changes(
                "abc",
                &[whole("xyz"), partial((0, 0), (0, 1), "A")],
                &PositionEncodingKind::UTF16,
            )
            .unwrap(),
            "Ayz"
        );
    }
}
