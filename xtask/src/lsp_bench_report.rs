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

/// Mirrors the sidecar `benches/benches/lsp.rs` writes; fields it doesn't
/// report on (e.g. `code_action`) are ignored.
#[derive(Deserialize, Default)]
#[serde(default)]
struct Sidecar {
    initialize: BTreeMap<String, u64>,
    diagnostics: BTreeMap<String, BTreeMap<String, u64>>,
    formatting: BTreeMap<String, BTreeMap<String, u64>>,
    formatting_edits: BTreeMap<String, BTreeMap<String, usize>>,
}

fn cell(ns: Option<f64>, rss_bytes: Option<u64>) -> String {
    match (ns, rss_bytes) {
        (Some(ns), Some(bytes)) => format!(
            "{:.1}ms ({:.1} MB)",
            ns / 1_000_000.0,
            bytes as f64 / (1024.0 * 1024.0)
        ),
        (Some(ns), None) => format!("{:.1}ms", ns / 1_000_000.0),
        _ => "—".to_owned(),
    }
}

fn edit_count(edits: Option<usize>) -> String {
    match edits {
        Some(1) => "1 edit".to_owned(),
        Some(n) => format!("{n} edits"),
        None => "—".to_owned(),
    }
}

fn row(label: &str, cells: impl IntoIterator<Item = String>) -> String {
    format!(
        "| {label} | {} |\n",
        cells.into_iter().collect::<Vec<_>>().join(" | ")
    )
}

fn table(
    fixture: &str,
    profiles: &BTreeSet<String>,
    times: &BTreeMap<String, f64>,
    sidecar: &Sidecar,
) -> String {
    let time = |id: String| times.get(&id).copied();
    let rss = |map: &BTreeMap<String, BTreeMap<String, u64>>, profile: &str| {
        map.get(profile).and_then(|m| m.get(fixture)).copied()
    };

    let mut out =
        String::from("Note: memory values are RSS (resident set size), shown in megabytes.\n\n");
    out.push_str(&row("method", profiles.iter().cloned()));
    out.push_str(&row("---", profiles.iter().map(|_| "---".to_owned())));
    out.push_str(&row(
        "initialize",
        profiles.iter().map(|p| {
            cell(
                time(format!("initialize/{p}")),
                sidecar.initialize.get(p).copied(),
            )
        }),
    ));
    out.push_str(&row(
        "diagnostic",
        profiles.iter().map(|p| {
            cell(
                time(format!("diagnostics/{p}/{fixture}")),
                rss(&sidecar.diagnostics, p),
            )
        }),
    ));
    out.push_str(&row(
        "formatting",
        profiles.iter().map(|p| {
            cell(
                time(format!("formatting/{p}/{fixture}")),
                rss(&sidecar.formatting, p),
            )
        }),
    ));
    out.push_str(&row(
        "formatting edits",
        profiles.iter().map(|p| {
            edit_count(
                sidecar
                    .formatting_edits
                    .get(p)
                    .and_then(|m| m.get(fixture))
                    .copied(),
            )
        }),
    ));
    out
}

/// Reads `cargo criterion --bench lsp --message-format=json` from stdin,
/// joins it with the RSS/edit-count sidecar the bench wrote, and writes one
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
    let sidecar: Sidecar = serde_json::from_str(
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
        fs::write(&path, table(fixture, &profiles, &times, &sidecar))
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
        let sidecar = Sidecar {
            initialize: BTreeMap::from([("jjpwrgem".to_owned(), 3 * 1024 * 1024)]),
            formatting_edits: BTreeMap::from([(
                "vscode-json".to_owned(),
                BTreeMap::from([("small".to_owned(), 21)]),
            )]),
            ..Sidecar::default()
        };

        let out = table("small", &profiles, &times, &sidecar);

        assert!(out.contains("| method | jjpwrgem | vscode-json |"));
        assert!(out.contains("| initialize | 6.6ms (3.0 MB) | — |"));
        assert!(out.contains("| diagnostic | 0.2ms | — |"));
        assert!(out.contains("| formatting | — | 0.5ms |"));
        assert!(out.contains("| formatting edits | — | 21 edits |"));
    }
}
