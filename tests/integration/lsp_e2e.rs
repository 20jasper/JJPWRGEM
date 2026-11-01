//! Drives real JSON language servers through lspresso under every client
//! capability preset, with generated documents full of non-ASCII text, and
//! checks what every server must get right: the ranges in its diagnostics
//! and in each feature it declares valid in the negotiated position encoding
//! and nested where they must be, and formatting edits that preserve the
//! document's value and settle after one round. jjp always runs; other
//! servers run when installed, and must be when named in the comma
//! separated `LSP_E2E_REQUIRE` (as `mise run test-lsp-e2e` does).

use std::{process::Command, time::Duration};

use lspresso::{
    Client, LanguageKind, capabilities,
    gen_lsp_types::{ClientCapabilities, PositionEncodingKind},
};
use proptest::{prelude::*, test_runner::TestRunner};

const TIMEOUT: Duration = Duration::from_secs(5);

type Preset = fn() -> ClientCapabilities;

const PRESETS: [(&str, Preset); 4] = [
    ("vscode", capabilities::vscode),
    ("helix", capabilities::helix),
    ("neovim", capabilities::neovim),
    ("zed", capabilities::zed),
];

struct Server {
    label: &'static str,
    command: fn() -> Command,
    /// Push-diagnostics latency differs wildly (vscode-json debounces for
    /// ~500ms), so slow servers get fewer cases.
    cases: u32,
    /// Known bug: some diagnostics land between the two UTF-16 code units
    /// of a non-BMP character, e.g. "Colon expected" on `{"😀`.
    splits_surrogate_pairs: bool,
    /// Known bug: in an unclosed container, the selection range for its
    /// contents ends a character early, as if before the missing bracket,
    /// so it can miss the value inside it.
    nests_selection_ranges_only_when_closed: bool,
}

fn jjp() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_jjp"));
    command.arg("lsp");
    command
}

fn vscode_json() -> Command {
    let mut command = Command::new("vscode-json-language-server");
    command.arg("--stdio");
    command
}

fn light_json_lsp() -> Command {
    Command::new("light-json-lsp")
}

fn cases(default: u32) -> u32 {
    std::env::var("LSP_E2E_CASES")
        .ok()
        .and_then(|cases| cases.parse().ok())
        .unwrap_or(default)
}

const SERVERS: [Server; 3] = [
    Server {
        label: "jjpwrgem",
        command: jjp,
        cases: 48,
        splits_surrogate_pairs: false,
        nests_selection_ranges_only_when_closed: false,
    },
    Server {
        label: "vscode-json",
        command: vscode_json,
        cases: 6,
        splits_surrogate_pairs: true,
        nests_selection_ranges_only_when_closed: true,
    },
    Server {
        label: "light-json-lsp",
        command: light_json_lsp,
        cases: 12,
        splits_surrogate_pairs: false,
        nests_selection_ranges_only_when_closed: false,
    },
];

#[derive(Debug, Clone, Copy)]
enum Encoding {
    Utf8,
    Utf16,
    Utf32,
}

impl Encoding {
    fn negotiated(client: &Client) -> Self {
        let declared = client
            .server_capabilities()
            .and_then(|caps| caps.position_encoding.clone());
        match declared {
            Some(PositionEncodingKind::UTF8) => Self::Utf8,
            Some(PositionEncodingKind::UTF32) => Self::Utf32,
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

/// The byte offset `position` names in `text`, or `None` if it isn't a
/// valid position: past the last line, or splitting a character. A
/// character past the end of its line means the end of the line.
fn byte_offset(text: &str, position: lspresso::Position, encoding: Encoding) -> Option<usize> {
    let line_start: usize = text
        .split_inclusive('\n')
        .take(usize::try_from(position.line).unwrap())
        .map(str::len)
        .sum();
    if usize::try_from(position.line).unwrap() >= line_count(text) {
        return None;
    }
    let line = text[line_start..].split('\n').next().unwrap_or("");
    let mut units = 0;
    for (offset, c) in line.char_indices() {
        if units == position.character {
            return Some(line_start + offset);
        }
        units += encoding.units(c);
        if units > position.character {
            return None;
        }
    }
    Some(line_start + line.len())
}

/// Like [`byte_offset`], but a line past the end means the end of the
/// document, as VS Code reads edits: servers commonly replace the whole
/// document with an edit ending at `u32::MAX`.
fn edit_offset(text: &str, position: lspresso::Position, encoding: Encoding) -> Option<usize> {
    if usize::try_from(position.line).unwrap() >= line_count(text) {
        return Some(text.len());
    }
    byte_offset(text, position, encoding)
}

fn line_count(text: &str) -> usize {
    (text.split_inclusive('\n').count() + usize::from(text.ends_with('\n'))).max(1)
}

/// The position of byte `offset` in `text`, which must be a char boundary.
fn position_of(text: &str, offset: usize, encoding: Encoding) -> lspresso::Position {
    let before = &text[..offset];
    let line_start = before.rfind('\n').map_or(0, |newline| newline + 1);
    lspresso::Position::new(
        u32::try_from(before.matches('\n').count()).unwrap(),
        before[line_start..]
            .chars()
            .map(|c| encoding.units(c))
            .sum(),
    )
}

/// Whether `position` falls between the two UTF-16 code units of a
/// non-BMP character.
fn splits_surrogate_pair(text: &str, position: lspresso::Position) -> bool {
    let Some(before) = position.character.checked_sub(1) else {
        return false;
    };
    let before = lspresso::Position::new(position.line, before);
    byte_offset(text, before, Encoding::Utf16)
        .and_then(|offset| text[offset..].chars().next())
        .is_some_and(|c| c.len_utf16() == 2)
}

fn apply(text: &str, edits: &[lspresso::TextEdit], encoding: Encoding) -> String {
    let mut spans: Vec<_> = edits
        .iter()
        .map(|edit| {
            let start = edit_offset(text, edit.range.start, encoding)
                .unwrap_or_else(|| panic!("edit starts at an invalid position: {edit:?}"));
            let end = edit_offset(text, edit.range.end, encoding)
                .unwrap_or_else(|| panic!("edit ends at an invalid position: {edit:?}"));
            assert!(start <= end, "edit range is backwards: {edit:?}");
            (start, end, edit.new_text.as_str())
        })
        .collect();
    spans.sort_by_key(|&(start, end, _)| (start, end));
    for pair in spans.windows(2) {
        let [(_, earlier_end, _), (later_start, _, _)] = pair else {
            unreachable!("windows(2) yields pairs")
        };
        assert!(earlier_end <= later_start, "overlapping edits: {edits:?}");
    }
    let mut out = text.to_owned();
    for (start, end, new_text) in spans.into_iter().rev() {
        out.replace_range(start..end, new_text);
    }
    out
}

fn string() -> impl Strategy<Value = String> {
    proptest::collection::vec(
        prop_oneof![
            4 => proptest::char::range('a', 'z'),
            1 => Just('é'),
            1 => Just('中'),
            1 => Just('😀'),
            1 => Just('𝄞'),
            1 => Just('"'),
            1 => Just('\n'),
        ],
        0..6,
    )
    .prop_map(String::from_iter)
}

fn value() -> impl Strategy<Value = serde_json::Value> {
    let leaf = prop_oneof![
        Just(serde_json::Value::Null),
        any::<bool>().prop_map(serde_json::Value::from),
        (-1000i64..1000).prop_map(serde_json::Value::from),
        string().prop_map(serde_json::Value::from),
    ];
    leaf.prop_recursive(3, 24, 4, |inner| {
        prop_oneof![
            proptest::collection::vec(inner.clone(), 0..4).prop_map(serde_json::Value::from),
            proptest::collection::btree_map(string(), inner, 0..4)
                .prop_map(|map| serde_json::Value::Object(map.into_iter().collect())),
        ]
    })
}

/// A serialized value (compact or pretty), sometimes cut off at a char
/// boundary: an unclosed container, string, or literal that no lenient
/// JSON dialect accepts either. Never cut to nothing, since servers
/// disagree on whether an empty document is an error.
fn document() -> impl Strategy<Value = String> {
    (
        value(),
        any::<bool>(),
        prop::option::of(any::<prop::sample::Index>()),
    )
        .prop_map(|(value, pretty, cut)| {
            let text = if pretty {
                serde_json::to_string_pretty(&value).unwrap()
            } else {
                serde_json::to_string(&value).unwrap()
            };
            let boundaries: Vec<_> = text.char_indices().map(|(i, _)| i).skip(1).collect();
            match cut {
                Some(cut) if !boundaries.is_empty() => text[..*cut.get(&boundaries)].to_owned(),
                _ => text,
            }
        })
}

fn spawn_initialized(server: &Server, preset: Preset) -> Option<Client> {
    let mut client = Client::spawn((server.command)()).ok()?;
    client.initialize(preset()).ok()?;
    Some(client)
}

/// The document as the server has it, to check the positions it answers
/// with against.
struct Seen<'a> {
    uri: &'a lspresso::Uri,
    text: &'a str,
    valid: bool,
    encoding: Encoding,
    server: &'a Server,
}

impl Seen<'_> {
    /// The byte offset of `position`, reading a tolerated split surrogate
    /// pair as the start of its character.
    fn offset(&self, position: lspresso::Position) -> Option<usize> {
        if self.server.splits_surrogate_pairs
            && matches!(self.encoding, Encoding::Utf16)
            && splits_surrogate_pair(self.text, position)
        {
            let before = lspresso::Position::new(position.line, position.character - 1);
            return byte_offset(self.text, before, Encoding::Utf16);
        }
        byte_offset(self.text, position, self.encoding)
    }

    fn span(
        &self,
        range: lspresso::Range,
        context: &dyn std::fmt::Display,
    ) -> Result<(usize, usize), TestCaseError> {
        let (Some(start), Some(end)) = (self.offset(range.start), self.offset(range.end)) else {
            return Err(TestCaseError::fail(format!(
                "invalid {:?} range {range:?} in {context}",
                self.encoding
            )));
        };
        prop_assert!(start <= end, "backwards range {range:?} in {context}");
        Ok((start, end))
    }

    /// Every range in `value` that's in this document: those under another
    /// document's `uri` or `targetUri` are skipped.
    fn ranges(&self, value: &serde_json::Value, out: &mut Vec<lspresso::Range>) {
        match value {
            serde_json::Value::Array(items) => {
                for item in items {
                    self.ranges(item, out);
                }
            }
            serde_json::Value::Object(fields) => {
                let elsewhere = ["uri", "targetUri"].into_iter().any(|key| {
                    fields
                        .get(key)
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|uri| uri != self.uri.as_str())
                });
                if elsewhere {
                    return;
                }
                if let Ok(range) = serde_json::from_value(value.clone()) {
                    out.push(range);
                }
                for field in fields.values() {
                    self.ranges(field, out);
                }
            }
            _ => {}
        }
    }

    /// Checks every range in `answer`, returning it as JSON.
    fn check_answer(
        &self,
        method: &str,
        answer: lspresso::Result<impl serde::Serialize>,
    ) -> Result<serde_json::Value, TestCaseError> {
        let answer =
            answer.map_err(|err| TestCaseError::fail(format!("{method} failed: {err}")))?;
        let value = serde_json::to_value(answer).unwrap();
        let mut ranges = Vec::new();
        self.ranges(&value, &mut ranges);
        for range in ranges {
            self.span(range, &format_args!("{method}: {value}"))?;
        }
        Ok(value)
    }

    fn contains(
        &self,
        outer: lspresso::Range,
        inner: lspresso::Range,
        context: &dyn std::fmt::Display,
    ) -> Result<bool, TestCaseError> {
        let (outer_start, outer_end) = self.span(outer, context)?;
        let (inner_start, inner_end) = self.span(inner, context)?;
        Ok(outer_start <= inner_start && inner_end <= outer_end)
    }
}

fn declares(client: &Client, capability: &str) -> bool {
    let capabilities = serde_json::to_value(client.server_capabilities()).unwrap();
    capabilities
        .get(capability)
        .is_some_and(|provider| !provider.is_null() && *provider != false)
}

fn range(value: &serde_json::Value) -> lspresso::Range {
    serde_json::from_value(value.clone()).unwrap()
}

/// Calls each position and document feature the server declares, checking
/// that every range it answers with is valid, and that the ranges that must
/// nest do: a hover's around its position, a symbol's name within it, and
/// each selection range within its parent.
fn check_features(
    client: &mut Client,
    seen: &Seen<'_>,
    probes: &[usize],
) -> Result<(), TestCaseError> {
    let positions: Vec<_> = probes
        .iter()
        .map(|&offset| position_of(seen.text, offset, seen.encoding))
        .collect();
    for &position in &positions {
        if declares(client, "hoverProvider") {
            let hover = seen.check_answer("hover", client.hover(seen.uri, position))?;
            if let Some(around) = hover.get("range") {
                let at = lspresso::Range::new(position, position);
                prop_assert!(
                    seen.contains(range(around), at, &hover)?,
                    "hover at {position:?} isn't around it: {hover}"
                );
            }
        }
        if declares(client, "completionProvider") {
            seen.check_answer("completion", client.completion(seen.uri, position))?;
        }
        if declares(client, "definitionProvider") {
            seen.check_answer("definition", client.definition(seen.uri, position))?;
        }
    }

    if declares(client, "documentSymbolProvider") {
        let symbols = seen.check_answer("documentSymbol", client.document_symbol(seen.uri))?;
        let mut pending = vec![&symbols];
        while let Some(value) = pending.pop() {
            if let (Some(whole), Some(name)) = (value.get("range"), value.get("selectionRange")) {
                prop_assert!(
                    seen.contains(range(whole), range(name), &symbols)?,
                    "a symbol's selection range is outside it: {value}"
                );
            }
            match value {
                serde_json::Value::Array(items) => pending.extend(items),
                serde_json::Value::Object(fields) => pending.extend(fields.values()),
                _ => {}
            }
        }
    }
    if declares(client, "foldingRangeProvider") {
        let folds = client
            .folding_range(seen.uri)
            .map_err(|err| TestCaseError::fail(format!("foldingRange failed: {err}")))?;
        for fold in folds.unwrap_or_default() {
            prop_assert!(
                fold.start_line <= fold.end_line
                    && usize::try_from(fold.end_line).unwrap() < line_count(seen.text),
                "folding range outside the document: {fold:?}"
            );
        }
    }
    if declares(client, "documentLinkProvider") {
        seen.check_answer("documentLink", client.document_link(seen.uri))?;
    }
    if declares(client, "colorProvider") {
        seen.check_answer("documentColor", client.document_color(seen.uri))?;
    }
    if declares(client, "selectionRangeProvider") {
        let params = serde_json::from_value(serde_json::json!({
            "textDocument": { "uri": seen.uri },
            "positions": positions,
        }))
        .unwrap();
        let answer = seen.check_answer("selectionRange", client.selection_range(params))?;
        let chains: Vec<_> = answer.as_array().cloned().unwrap_or_default();
        prop_assert_eq!(
            chains.len(),
            positions.len(),
            "one selection range per position"
        );
        for (chain, &position) in chains.iter().zip(&positions) {
            let mut inner = lspresso::Range::new(position, position);
            let mut link = Some(chain);
            while let Some(selection) = link {
                let outer = range(&selection["range"]);
                let tolerated = seen.server.nests_selection_ranges_only_when_closed && !seen.valid;
                prop_assert!(
                    seen.contains(outer, inner, &answer)? || tolerated,
                    "selection range {outer:?} doesn't contain {inner:?}: {answer}"
                );
                inner = outer;
                link = selection.get("parent");
            }
        }
    }
    Ok(())
}

/// Opens `text` and checks the server's answers, probing its features at
/// the char boundaries `probes` pick.
fn check_document(
    client: &mut Client,
    server: &Server,
    encoding: Encoding,
    uri: &lspresso::Uri,
    text: &str,
    probes: &[prop::sample::Index],
) -> Result<(), TestCaseError> {
    let original = serde_json::from_str::<serde_json::Value>(text).ok();
    let seen = Seen {
        uri,
        text,
        valid: original.is_some(),
        encoding,
        server,
    };
    client
        .did_open(uri.clone(), text, LanguageKind::JSON)
        .unwrap();
    let diagnostics = client.next_diagnostics(uri, TIMEOUT).unwrap();
    for diagnostic in &diagnostics {
        seen.span(diagnostic.range, &format_args!("diagnostic {diagnostic:?}"))?;
    }
    prop_assert_eq!(
        diagnostics.is_empty(),
        original.is_some(),
        "diagnostics disagree with whether the document is valid JSON: {:?}",
        diagnostics
    );

    let boundaries: Vec<_> = text
        .char_indices()
        .map(|(offset, _)| offset)
        .chain([text.len()])
        .collect();
    let probes: Vec<_> = probes.iter().map(|probe| *probe.get(&boundaries)).collect();
    check_features(client, &seen, &probes)?;

    if let Some(original) = original
        && client.supports_formatting()
    {
        let edits = client.formatting(uri).unwrap().unwrap_or_default();
        let formatted = apply(text, &edits, encoding);
        let reparsed = serde_json::from_str::<serde_json::Value>(&formatted);
        prop_assert!(
            reparsed.is_ok(),
            "formatting broke the document: {formatted:?}"
        );
        prop_assert_eq!(reparsed.unwrap(), original, "formatting changed the value");

        client
            .did_change(uri, &edits)
            .map_err(|err| TestCaseError::fail(format!("syncing the edits failed: {err}")))?;
        prop_assert_eq!(
            client.document_text(uri),
            Some(formatted.as_str()),
            "lspresso applied the edits differently"
        );
        let diagnostics = client.next_diagnostics(uri, TIMEOUT).unwrap();
        prop_assert!(
            diagnostics.is_empty(),
            "the formatted document, synced as its edits, has diagnostics: {diagnostics:?}"
        );
        let again = client.formatting(uri).unwrap().unwrap_or_default();
        prop_assert_eq!(
            apply(&formatted, &again, encoding),
            formatted,
            "formatting didn't settle after one round"
        );
    }
    client.did_close(uri).unwrap();
    Ok(())
}

/// Each case gets its own server process, so a crash is pinned on the
/// document that caused it and shrinking stays meaningful.
fn check_server(server: &Server) {
    for (preset_name, preset) in PRESETS {
        if spawn_initialized(server, preset).is_none() {
            let required = std::env::var("LSP_E2E_REQUIRE").unwrap_or_default();
            assert!(
                !required.split(',').any(|label| label == server.label),
                "{} is required but couldn't be spawned and initialized",
                server.label
            );
            return;
        }
        let mut runner = TestRunner::new(ProptestConfig::with_cases(cases(server.cases)));
        let uri = lspresso::parse_uri("file:///e2e.json");
        let probes = proptest::collection::vec(any::<prop::sample::Index>(), 1..4);
        let result = runner.run(&(document(), probes), |(text, probes)| {
            let mut client =
                spawn_initialized(server, preset).expect("the server initialized before");
            let encoding = Encoding::negotiated(&client);
            check_document(&mut client, server, encoding, &uri, &text, &probes)
                .map_err(|err| TestCaseError::fail(format!("({encoding:?}) {err}")))
        });
        if let Err(err) = result {
            panic!("{} under the {preset_name} preset: {err}", server.label);
        }
    }
}

#[test]
fn jjpwrgem_satisfies_the_e2e_properties_under_every_preset() {
    check_server(&SERVERS[0]);
}

#[test]
fn vscode_json_satisfies_the_e2e_properties_under_every_preset() {
    check_server(&SERVERS[1]);
}

#[test]
fn light_json_lsp_satisfies_the_e2e_properties_under_every_preset() {
    check_server(&SERVERS[2]);
}

#[test]
fn byte_offset_counts_in_the_negotiated_encoding() {
    let text = "a😀b\nc";
    let at = |line, character| lspresso::Position::new(line, character);
    assert_eq!(byte_offset(text, at(0, 3), Encoding::Utf16), Some(5));
    assert_eq!(byte_offset(text, at(0, 2), Encoding::Utf16), None);
    assert_eq!(byte_offset(text, at(0, 5), Encoding::Utf8), Some(5));
    assert_eq!(byte_offset(text, at(0, 2), Encoding::Utf32), Some(5));
    assert_eq!(byte_offset(text, at(0, 99), Encoding::Utf16), Some(6));
    assert_eq!(byte_offset(text, at(1, 1), Encoding::Utf16), Some(8));
    assert_eq!(byte_offset(text, at(2, 0), Encoding::Utf16), None);
    assert!(splits_surrogate_pair(text, at(0, 2)));
    assert!(!splits_surrogate_pair(text, at(0, 3)));
    assert!(!splits_surrogate_pair(text, at(0, 0)));
}
