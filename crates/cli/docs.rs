macro_rules! get_docs_snapshot {
    ($name:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/integration/snapshots/",
            $name,
            ".snap"
        ))
    };
}

pub(crate) use get_docs_snapshot;

pub fn strip_front_matter(raw: &str) -> &str {
    const FRONT_MATTER_SEP: &str = "\n---\n";
    raw.split_once(FRONT_MATTER_SEP)
        .expect("snapshots should always have a separator")
        .1
}
pub fn indent(s: &str) -> String {
    s.lines()
        .map(|x| format!("\t{x}"))
        .collect::<Vec<_>>()
        .join("\n")
}
