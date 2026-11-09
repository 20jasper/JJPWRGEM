//! Drives real JSON language servers through lspresso under every client
//! capability preset and checks what every server must get right: ranges in
//! diagnostics and declared features are valid in the negotiated position
//! encoding and nest where they must, and formatting keeps the document's
//! value and settles after one round. jjp always runs; other servers run when
//! installed, and must be when named in `LSP_E2E_REQUIRE`.

use std::{process::Command, time::Duration};

use lspresso::{
    Client, capabilities,
    gen_lsp_types::{
        ClientCapabilities, Diagnostic, FormattingOptions, LanguageKind, Position,
        PositionEncodingKind, Range, TextEdit, Uri,
    },
};
use serde::Serialize;
use serde_json::Value;

const TIMEOUT: Duration = Duration::from_secs(5);

type Preset = fn() -> ClientCapabilities;

const PRESETS: [(&str, Preset); 4] = [
    ("vscode", capabilities::vscode::latest),
    ("helix", capabilities::helix::latest),
    ("neovim", capabilities::neovim::latest),
    ("emacs", capabilities::emacs::latest),
];

const CASES: &[&str] = &[
    "{}",
    "[]",
    r#"{"name": "café"}"#,
    r#"{"emoji": "😀", "music": "𝄞"}"#,
    r#"{"nested": {"list": [1, 2, 3], "empty": null}}"#,
    "{\n  \"中文\": \"∑\",\n  \"flags\": [true, false]\n}",
    r#"{"quote": "\"", "slash": "\\", "newline": "\n"}"#,
    r#"{"a": 1"#,
    r#"{"a": }"#,
    r#"{"a": "unterminated}"#,
    r"[1, 2,",
    r#"{"emoji": "😀"#,
    r#"{"a": truee}"#,
    r#"{"a": 01}"#,
    r#"{"a" 1}"#,
    r#"{"a": "é"#,
];

struct Server {
    label: &'static str,
    command: fn() -> Command,
    cases: usize,
    /// Known bug: some diagnostics land between the two UTF-16 code units
    /// of a non-BMP character, e.g. "Colon expected" on `{"😀`.
    splits_surrogate_pairs: bool,
    /// Known bug: in an unclosed container, the selection range for its
    /// contents ends a character early, as if before the missing bracket.
    nests_selection_ranges_only_when_closed: bool,
}

fn jjp() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_jjp"));
    command.arg("lsp");
    command
}

fn vscode_json() -> Command {
    let binary =
        std::env::var("VSCODE_JSON_BIN").unwrap_or_else(|_| "vscode-json-language-server".into());
    let mut command = Command::new(binary);
    command.arg("--stdio");
    command
}

fn light_json_lsp() -> Command {
    Command::new("light-json-lsp")
}

const SERVERS: [Server; 3] = [
    Server {
        label: "jjpwrgem",
        command: jjp,
        cases: CASES.len(),
        splits_surrogate_pairs: false,
        nests_selection_ranges_only_when_closed: false,
    },
    Server {
        label: "vscode-json",
        command: vscode_json,
        cases: 12,
        splits_surrogate_pairs: true,
        nests_selection_ranges_only_when_closed: true,
    },
    Server {
        label: "light-json-lsp",
        command: light_json_lsp,
        cases: 13,
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
        match client
            .server_capabilities()
            .and_then(|caps| caps.position_encoding.as_ref())
        {
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

fn line_count(text: &str) -> usize {
    (text.split_inclusive('\n').count() + usize::from(text.ends_with('\n'))).max(1)
}

/// The byte offset `position` names in `text`, or `None` if it isn't a valid
/// position: past the last line, or splitting a character. A character past
/// the end of its line means the end of the line.
fn byte_offset(text: &str, position: Position, encoding: Encoding) -> Option<usize> {
    let line = usize::try_from(position.line).unwrap();
    if line >= line_count(text) {
        return None;
    }
    let line_start: usize = text.split_inclusive('\n').take(line).map(str::len).sum();
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
fn edit_offset(text: &str, position: Position, encoding: Encoding) -> Option<usize> {
    if usize::try_from(position.line).unwrap() >= line_count(text) {
        return Some(text.len());
    }
    byte_offset(text, position, encoding)
}

/// The position of byte `offset` in `text`, which must be a char boundary.
fn position_of(text: &str, offset: usize, encoding: Encoding) -> Position {
    let before = &text[..offset];
    let line_start = before.rfind('\n').map_or(0, |newline| newline + 1);
    Position::new(
        u32::try_from(before.matches('\n').count()).unwrap(),
        before[line_start..]
            .chars()
            .map(|c| encoding.units(c))
            .sum(),
    )
}

/// Whether `position` falls between the two UTF-16 code units of a non-BMP
/// character.
fn splits_surrogate_pair(text: &str, position: Position) -> bool {
    let Some(before) = position.character.checked_sub(1) else {
        return false;
    };
    let before = Position::new(position.line, before);
    byte_offset(text, before, Encoding::Utf16)
        .and_then(|offset| text[offset..].chars().next())
        .is_some_and(|c| c.len_utf16() == 2)
}

fn apply(text: &str, edits: &[TextEdit], encoding: Encoding) -> Result<String, String> {
    let mut spans: Vec<_> = edits
        .iter()
        .map(|edit| {
            let start = edit_offset(text, edit.range.start, encoding)
                .ok_or_else(|| format!("edit starts at an invalid position: {edit:?}"))?;
            let end = edit_offset(text, edit.range.end, encoding)
                .ok_or_else(|| format!("edit ends at an invalid position: {edit:?}"))?;
            if start > end {
                return Err(format!("edit range is backwards: {edit:?}"));
            }
            Ok((start, end, edit.new_text.as_str()))
        })
        .collect::<Result<_, String>>()?;
    spans.sort_unstable_by_key(|&(start, end, _)| (start, end));
    for pair in spans.windows(2) {
        let [(_, earlier_end, _), (later_start, _, _)] = pair else {
            unreachable!("windows(2) yields pairs")
        };
        if earlier_end > later_start {
            return Err(format!("overlapping edits: {edits:?}"));
        }
    }
    let mut out = text.to_owned();
    for (start, end, new_text) in spans.into_iter().rev() {
        out.replace_range(start..end, new_text);
    }
    Ok(out)
}

fn probes(text: &str) -> Vec<usize> {
    let mut offsets = vec![0, text.len()];
    offsets.extend(
        text.char_indices()
            .filter(|(_, c)| !c.is_ascii())
            .flat_map(|(offset, c)| [offset, offset + c.len_utf8()]),
    );
    offsets.extend(
        text.match_indices('\n')
            .flat_map(|(offset, _)| [offset, offset + 1]),
    );
    offsets.sort_unstable();
    offsets.dedup();
    offsets
}

fn spawn_initialized(server: &Server, capabilities: ClientCapabilities) -> Option<Client> {
    let mut client = Client::spawn((server.command)())
        .ok()?
        .with_timeout(TIMEOUT);
    client.initialize(capabilities).ok()?;
    Some(client)
}

/// vscode-json advertises pull diagnostics to every client but only handles
/// the method when the client declares diagnostic support, so presets without
/// it use push instead.
#[derive(Clone, Copy)]
enum DiagnosticsMode {
    Pull,
    Push,
}

fn document_diagnostics(
    client: &mut Client,
    uri: &Uri,
    mode: DiagnosticsMode,
) -> Result<Vec<Diagnostic>, String> {
    match mode {
        DiagnosticsMode::Pull => client.diagnostics(uri),
        DiagnosticsMode::Push => client.next_diagnostics(uri),
    }
    .map_err(|err| format!("diagnostics failed: {err}"))
}

fn declares(client: &Client, capability: &str) -> bool {
    let capabilities = serde_json::to_value(client.server_capabilities()).unwrap();
    capabilities
        .get(capability)
        .is_some_and(|provider| !provider.is_null() && *provider != false)
}

fn range(value: &Value) -> Result<Range, String> {
    serde_json::from_value(value.clone()).map_err(|err| format!("invalid range {value}: {err}"))
}

/// The document as the server has it, to check the positions it answers with.
struct Seen<'a> {
    uri: &'a Uri,
    text: &'a str,
    valid: bool,
    encoding: Encoding,
    server: &'a Server,
}

impl Seen<'_> {
    /// The byte offset of `position`, reading a tolerated split surrogate
    /// pair as the start of its character.
    fn offset(&self, position: Position) -> Option<usize> {
        if self.server.splits_surrogate_pairs
            && matches!(self.encoding, Encoding::Utf16)
            && splits_surrogate_pair(self.text, position)
        {
            let before = Position::new(position.line, position.character - 1);
            return byte_offset(self.text, before, Encoding::Utf16);
        }
        byte_offset(self.text, position, self.encoding)
    }

    fn span(
        &self,
        range: Range,
        context: &dyn std::fmt::Display,
    ) -> Result<(usize, usize), String> {
        let (Some(start), Some(end)) = (self.offset(range.start), self.offset(range.end)) else {
            return Err(format!(
                "invalid {:?} range {range:?} in {context}",
                self.encoding
            ));
        };
        if start > end {
            return Err(format!("backwards range {range:?} in {context}"));
        }
        Ok((start, end))
    }

    /// Every range in `value` that's in this document: those under another
    /// document's `uri` or `targetUri` are skipped.
    fn ranges(&self, value: &Value, out: &mut Vec<Range>) {
        match value {
            Value::Array(items) => {
                for item in items {
                    self.ranges(item, out);
                }
            }
            Value::Object(fields) => {
                let elsewhere = ["uri", "targetUri"].into_iter().any(|key| {
                    fields
                        .get(key)
                        .and_then(Value::as_str)
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

    fn check_answer(
        &self,
        method: &str,
        answer: lspresso::Result<impl Serialize>,
    ) -> Result<Value, String> {
        let answer = answer.map_err(|err| format!("{method} failed: {err}"))?;
        let value = serde_json::to_value(answer).map_err(|err| format!("{method}: {err}"))?;
        let mut ranges = Vec::new();
        self.ranges(&value, &mut ranges);
        for range in ranges {
            self.span(range, &format_args!("{method}: {value}"))?;
        }
        Ok(value)
    }

    fn contains(
        &self,
        outer: Range,
        inner: Range,
        context: &dyn std::fmt::Display,
    ) -> Result<bool, String> {
        let (outer_start, outer_end) = self.span(outer, context)?;
        let (inner_start, inner_end) = self.span(inner, context)?;
        Ok(outer_start <= inner_start && inner_end <= outer_end)
    }
}

/// Calls each position and document feature the server declares, checking
/// that every range it answers with is valid, and that the ranges that must
/// nest do: a hover's around its position, a symbol's name within it, and
/// each selection range within its parent.
fn check_features(client: &mut Client, seen: &Seen<'_>) -> Result<(), String> {
    let positions: Vec<_> = probes(seen.text)
        .into_iter()
        .map(|offset| position_of(seen.text, offset, seen.encoding))
        .collect();

    for &position in &positions {
        if declares(client, "hoverProvider") {
            let hover = seen.check_answer("hover", client.hover(seen.uri, position))?;
            if let Some(around) = hover.get("range") {
                let at = Range::new(position, position);
                if !seen.contains(range(around)?, at, &hover)? {
                    return Err(format!("hover at {position:?} isn't around it: {hover}"));
                }
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
            if let (Some(whole), Some(name)) = (value.get("range"), value.get("selectionRange"))
                && !seen.contains(range(whole)?, range(name)?, &symbols)?
            {
                return Err(format!("a symbol's selection range is outside it: {value}"));
            }
            match value {
                Value::Array(items) => pending.extend(items),
                Value::Object(fields) => pending.extend(fields.values()),
                _ => {}
            }
        }
    }
    if declares(client, "foldingRangeProvider") {
        for fold in client
            .folding_range(seen.uri)
            .map_err(|err| format!("foldingRange failed: {err}"))?
            .unwrap_or_default()
        {
            if fold.start_line > fold.end_line
                || usize::try_from(fold.end_line).unwrap() >= line_count(seen.text)
            {
                return Err(format!("folding range outside the document: {fold:?}"));
            }
        }
    }
    if declares(client, "documentLinkProvider") {
        seen.check_answer("documentLink", client.document_link(seen.uri))?;
    }
    if declares(client, "colorProvider") {
        seen.check_answer("documentColor", client.document_color(seen.uri))?;
    }
    if declares(client, "selectionRangeProvider") {
        let answer = seen.check_answer(
            "selectionRange",
            client.selection_range(seen.uri, positions.clone()),
        )?;
        let chains: Vec<_> = answer.as_array().cloned().unwrap_or_default();
        if chains.len() != positions.len() {
            return Err(format!(
                "expected {} selection ranges, got {}: {answer}",
                positions.len(),
                chains.len()
            ));
        }
        for (chain, &position) in chains.iter().zip(&positions) {
            let mut inner = Range::new(position, position);
            let mut link = Some(chain);
            while let Some(selection) = link {
                let outer = range(&selection["range"])?;
                let tolerated = seen.server.nests_selection_ranges_only_when_closed && !seen.valid;
                if !seen.contains(outer, inner, &answer)? && !tolerated {
                    return Err(format!(
                        "selection range {outer:?} doesn't contain {inner:?}: {answer}"
                    ));
                }
                inner = outer;
                link = selection.get("parent");
            }
        }
    }
    Ok(())
}

fn check_document(
    client: &mut Client,
    server: &Server,
    encoding: Encoding,
    mode: DiagnosticsMode,
    uri: &Uri,
    text: &str,
) -> Result<(), String> {
    let original = serde_json::from_str::<Value>(text).ok();
    let seen = Seen {
        uri,
        text,
        valid: original.is_some(),
        encoding,
        server,
    };
    client
        .did_open(uri.clone(), text, LanguageKind::JSON)
        .map_err(|err| format!("didOpen failed: {err}"))?;
    let diagnostics = document_diagnostics(client, uri, mode)?;
    for diagnostic in &diagnostics {
        seen.span(diagnostic.range, &format_args!("diagnostic {diagnostic:?}"))?;
    }
    if diagnostics.is_empty() != original.is_some() {
        return Err(format!(
            "diagnostics disagree with whether the document is valid JSON: {diagnostics:?}"
        ));
    }

    check_features(client, &seen)?;

    if let Some(original) = original
        && declares(client, "documentFormattingProvider")
    {
        let edits = client
            .formatting(uri, FormattingOptions::default())
            .map_err(|err| format!("formatting failed: {err}"))?
            .unwrap_or_default();
        let formatted = apply(text, &edits, encoding)?;
        let reparsed = serde_json::from_str::<Value>(&formatted)
            .map_err(|err| format!("formatting broke the document: {err}: {formatted:?}"))?;
        if reparsed != original {
            return Err(format!("formatting changed the value: {formatted:?}"));
        }

        if !edits.is_empty() {
            client
                .did_change(uri, &edits)
                .map_err(|err| format!("syncing the edits failed: {err}"))?;
            if client.document_text(uri) != Some(formatted.as_str()) {
                return Err("lspresso applied the edits differently".into());
            }
            let diagnostics = document_diagnostics(client, uri, mode)?;
            if !diagnostics.is_empty() {
                return Err(format!(
                    "the formatted document, synced as its edits, has diagnostics: {diagnostics:?}"
                ));
            }
        }

        let again = client
            .formatting(uri, FormattingOptions::default())
            .map_err(|err| format!("second formatting failed: {err}"))?
            .unwrap_or_default();
        if apply(&formatted, &again, encoding)? != formatted {
            return Err("formatting didn't settle after one round".into());
        }
    }

    client
        .did_close(uri)
        .map_err(|err| format!("didClose failed: {err}"))?;
    Ok(())
}

fn check_server(server: &Server) {
    let required = std::env::var("LSP_E2E_REQUIRE").unwrap_or_default();
    for (preset_name, preset) in PRESETS {
        let capabilities = preset();
        let mode = if capabilities
            .text_document
            .as_ref()
            .and_then(|capabilities| capabilities.diagnostic.as_ref())
            .is_some()
        {
            DiagnosticsMode::Pull
        } else {
            DiagnosticsMode::Push
        };
        let Some(mut client) = spawn_initialized(server, capabilities) else {
            assert!(
                !required.split(',').any(|label| label == server.label),
                "{} is required but couldn't be spawned and initialized",
                server.label
            );
            return;
        };
        let encoding = Encoding::negotiated(&client);
        for (index, &text) in CASES[..server.cases].iter().enumerate() {
            let uri: Uri = format!("file:///e2e-{index}.json").parse().unwrap();
            if let Err(err) = check_document(&mut client, server, encoding, mode, &uri, text) {
                panic!(
                    "{} under the {preset_name} preset: {err}\ndocument: {text:?}",
                    server.label
                );
            }
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
    let at = |line, character| Position::new(line, character);
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
