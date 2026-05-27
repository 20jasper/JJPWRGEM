#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonKind {
    Json,
    JsonLines,
}

/// Matches both language IDs and file extensions — the strings are the same.
pub fn kind_from_str(s: &str) -> Option<JsonKind> {
    match s.to_ascii_lowercase().as_str() {
        "json" => Some(JsonKind::Json),
        "jsonlines" | "jsonl" | "ndjson" => Some(JsonKind::JsonLines),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case("json", JsonKind::Json)]
    #[case("JSON", JsonKind::Json)]
    #[case("jsonl", JsonKind::JsonLines)]
    #[case("JSONL", JsonKind::JsonLines)]
    #[case("jsonlines", JsonKind::JsonLines)]
    #[case("ndjson", JsonKind::JsonLines)]
    fn known_strings(#[case] s: &str, #[case] expected: JsonKind) {
        assert_eq!(kind_from_str(s), Some(expected));
    }

    #[rstest::rstest]
    #[case("txt")]
    #[case("")]
    #[case("toml")]
    fn unknown_strings(#[case] s: &str) {
        assert_eq!(kind_from_str(s), None);
    }
}
