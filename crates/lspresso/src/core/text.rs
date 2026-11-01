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

/// Applies `edits`, each against `text` as it is, as the spec has a set of
/// `TextEdit`s apply. Edits may not overlap, though they may touch, and
/// inserts at the same place land in order; an insert inside a replaced
/// range overlaps it.
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
        changes.push(change(&result, start, end, new_text.clone(), encoding));
        result.replace_range(start..end, new_text);
    }
    if !representable && let Some(&(start, ..)) = spans.first() {
        let end = spans.iter().map(|&(_, end, _)| end).max().unwrap_or(start);
        let replaced = result[start..result.len() - (text.len() - end)].to_owned();
        changes = vec![change(text, start, end, replaced, encoding)];
    }
    Ok(Applied {
        text: result,
        changes,
    })
}

/// Whether `offset` falls between a `\r` and a `\n`, where no position can
/// point: one change can make that of where the next one starts, by
/// inserting a `\n` after a lone `\r`.
fn splits_line_break(text: &str, offset: usize) -> bool {
    text[..offset].ends_with('\r') && text[offset..].starts_with('\n')
}

fn change(
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
    use proptest::prelude::*;

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

    fn encoding() -> impl Strategy<Value = Encoding> {
        prop_oneof![
            Just(Encoding::Utf8),
            Just(Encoding::Utf16),
            Just(Encoding::Utf32)
        ]
    }

    fn text() -> impl Strategy<Value = String> {
        proptest::collection::vec(
            prop_oneof![
                Just("a"),
                Just("é"),
                Just("😀"),
                Just("\n"),
                Just("\r\n"),
                Just("\r")
            ],
            0..12,
        )
        .prop_map(|parts| parts.concat())
    }

    /// The char boundaries outside line terminators: where positions can
    /// point.
    fn boundaries(text: &str) -> Vec<usize> {
        text.char_indices()
            .map(|(index, _)| index)
            .chain([text.len()])
            .filter(|&index| !(text[..index].ends_with('\r') && text[index..].starts_with('\n')))
            .collect()
    }

    /// Non-overlapping byte spans and their replacement texts, shuffled.
    fn edits(text: &str) -> impl Strategy<Value = Vec<(usize, usize, String)>> + use<> {
        let boundaries = boundaries(text);
        proptest::collection::vec(
            (any::<prop::sample::Index>(), any::<bool>(), "[x😀\n]{0,2}"),
            0..5,
        )
        .prop_map(move |picks| {
            let mut cuts: Vec<_> = picks
                .iter()
                .map(|(cut, ..)| *cut.get(&boundaries))
                .collect();
            cuts.sort_unstable();
            let mut previous = 0;
            picks
                .into_iter()
                .zip(cuts)
                .map(|((_, wide, new_text), cut)| {
                    let start = if wide { previous } else { cut };
                    previous = cut;
                    (start, cut, new_text)
                })
                .collect::<Vec<_>>()
        })
        .prop_shuffle()
    }

    proptest! {
        #[test]
        fn position_and_offset_round_trip(text in text(), encoding in encoding()) {
            for offset_ in boundaries(&text) {
                let position = position(&text, offset_, encoding);
                prop_assert_eq!(offset(&text, position, encoding), Some(offset_));
            }
        }

        /// Applying the edits matches splicing their byte spans in, and the
        /// incremental changes, applied in turn, get to the same text.
        #[test]
        fn edits_and_their_changes_agree_with_splicing(
            (text, spans, encoding) in text().prop_flat_map(|text| {
                (Just(text.clone()), edits(&text), encoding())
            })
        ) {
            let as_edits: Vec<_> = spans
                .iter()
                .map(|(start, end, new_text)| TextEdit {
                    range: Range::new(position(&text, *start, encoding), position(&text, *end, encoding)),
                    new_text: new_text.clone(),
                })
                .collect();
            let applied = apply(&text, &as_edits, encoding).unwrap();

            let mut by_place: Vec<_> = spans.iter().enumerate().collect();
            by_place.sort_by_key(|&(index, (start, end, _))| (*start, *end, index));
            let mut expected = text.clone();
            for (_, (start, end, new_text)) in by_place.into_iter().rev() {
                expected.replace_range(start..end, new_text);
            }
            prop_assert_eq!(&applied.text, &expected);

            let mut replayed = text.clone();
            for change in applied.changes {
                let TextDocumentContentChangeEvent::TextDocumentContentChangePartial(change) = change else {
                    unreachable!("apply only makes partial changes")
                };
                let start = offset(&replayed, change.range.start, encoding).unwrap();
                let end = offset(&replayed, change.range.end, encoding).unwrap();
                replayed.replace_range(start..end, &change.text);
            }
            prop_assert_eq!(replayed, expected);
        }
    }
}
