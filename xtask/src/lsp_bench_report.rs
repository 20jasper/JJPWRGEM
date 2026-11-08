use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{self, BufRead as _},
};

use anyhow::Context;
use serde::Deserialize;

const FIXTURES: &[&str] = &["small", "twitter", "citm_catalog", "canada"];

#[derive(Deserialize)]
struct Estimate {
    estimate: f64,
}

#[derive(Deserialize)]
#[serde(tag = "reason")]
enum Message {
    #[serde(rename = "benchmark-complete")]
    BenchmarkComplete { id: String, typical: Estimate },
    #[serde(other)]
    Other,
}

fn cell(ns: Option<f64>, rss_bytes: Option<u64>) -> String {
    let timing = match ns {
        Some(ns) if ns < 1_000_000.0 => format!("{:.1}µs", ns / 1_000.0),
        Some(ns) => format!("{:.1}ms", ns / 1_000_000.0),
        None => return "—".to_owned(),
    };
    match rss_bytes {
        Some(bytes) => format!("{timing} ({:.1} MB)", bytes as f64 / (1024.0 * 1024.0)),
        None => timing,
    }
}

fn row(label: &str, cells: impl IntoIterator<Item = String>) -> String {
    format!(
        "| {label} | {} |\n",
        cells.into_iter().collect::<Vec<_>>().join(" | ")
    )
}

/// `times` and `memory` are both keyed by criterion's benchmark id
fn table(
    fixture: &str,
    profiles: &BTreeSet<String>,
    times: &BTreeMap<String, f64>,
    memory: &BTreeMap<String, u64>,
) -> String {
    let method = |label: &str, id: &dyn Fn(&str) -> String| {
        row(
            label,
            profiles.iter().map(|p| {
                let id = id(p);
                cell(times.get(&id).copied(), memory.get(&id).copied())
            }),
        )
    };

    let mut out =
        String::from("Note: memory values are RSS (resident set size), shown in megabytes.\n\n");
    out.push_str(&row("method", profiles.iter().cloned()));
    out.push_str(&row("---", profiles.iter().map(|_| "---".to_owned())));
    out.push_str(&method("initialize", &|p| format!("initialize/{p}")));
    out.push_str(&method("diagnostic", &|p| {
        format!("diagnostics/{p}/{fixture}")
    }));
    out.push_str(&method("formatting", &|p| {
        format!("formatting/{p}/{fixture}")
    }));
    out
}

/// Reads `cargo criterion --bench lsp --message-format=json` from stdin,
/// joins it with the RSS sidecar the bench wrote, and writes one
/// table per fixture to `benches/lsp/results/` for `generate-readmes` to
/// stitch into `benches/lsp/lsp.md`.
pub fn write_lsp_bench_report() -> anyhow::Result<()> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let mut times = BTreeMap::new();
    for line in io::stdin().lock().lines() {
        if let Ok(Message::BenchmarkComplete { id, typical }) = serde_json::from_str(&line?) {
            times.insert(id, typical.estimate);
        }
    }
    anyhow::ensure!(
        !times.is_empty(),
        "no benchmark-complete messages on stdin -- pipe in `cargo criterion --bench lsp --message-format=json`"
    );

    let sidecar_path = format!("{manifest_dir}/../target/bench/lsp-memory.json");
    let memory: BTreeMap<String, u64> = serde_json::from_str(
        &fs::read_to_string(&sidecar_path).with_context(|| format!("reading {sidecar_path}"))?,
    )
    .with_context(|| format!("parsing {sidecar_path}"))?;

    let profiles: BTreeSet<String> = times
        .keys()
        .filter_map(|id| match id.split('/').collect::<Vec<_>>()[..] {
            ["initialize" | "diagnostics" | "formatting", profile, ..] => Some(profile.to_owned()),
            _ => None,
        })
        .collect();

    let results_dir = format!("{manifest_dir}/../benches/lsp/results");
    fs::create_dir_all(&results_dir).with_context(|| format!("creating {results_dir}"))?;
    for fixture in FIXTURES {
        let path = format!("{results_dir}/{fixture}.md");
        fs::write(&path, table(fixture, &profiles, &times, &memory))
            .with_context(|| format!("writing {path}"))?;
        println!("wrote {path}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_one_row_per_method_and_one_column_per_profile() {
        let profiles = BTreeSet::from(["jjpwrgem".to_owned(), "vscode-json".to_owned()]);
        let times = BTreeMap::from([
            ("initialize/jjpwrgem".to_owned(), 6_600_000.0),
            ("diagnostics/jjpwrgem/small".to_owned(), 200_000.0),
            ("formatting/vscode-json/small".to_owned(), 500_000.0),
        ]);
        let memory = BTreeMap::from([("initialize/jjpwrgem".to_owned(), 3 * 1024 * 1024)]);

        let out = table("small", &profiles, &times, &memory);

        assert!(out.contains("| method | jjpwrgem | vscode-json |"));
        assert!(out.contains("| initialize | 6.6ms (3.0 MB) | — |"));
        assert!(out.contains("| diagnostic | 200.0µs | — |"));
        assert!(out.contains("| formatting | — | 500.0µs |"));
    }
}
