const FILES: &[&str] = &["canada", "citm_catalog", "twitter"];

fn data_path(name: &str) -> std::path::PathBuf {
    let manifest = env!("CARGO_MANIFEST_DIR");
    [manifest, "data", &format!("{name}.json")].iter().collect()
}

pub fn include_impl(name: &str) -> bool {
    match std::env::var("JJP_JSON_IMPL") {
        Ok(filter) if !filter.is_empty() => filter == name,
        _ => true,
    }
}

pub fn load_inputs() -> Vec<(&'static str, String)> {
    FILES
        .iter()
        .copied()
        .filter(|name| match std::env::var("JJP_JSON_INPUT") {
            Ok(filter) if !filter.is_empty() => filter == *name,
            _ => true,
        })
        .map(|name| (name, std::fs::read_to_string(data_path(name)).unwrap()))
        .collect()
}
