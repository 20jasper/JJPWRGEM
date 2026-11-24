use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{self, BufRead as _},
};

use anyhow::Context;
use serde::Deserialize;

use crate::{
    criterion::{Message, format_time},
    markdown::markdown_table,
};

const FIXTURES: &[&str] = &["small", "twitter", "citm_catalog", "canada"];

#[derive(Clone, Copy, Deserialize)]
struct Sums(f64, f64);

fn cell(ns: Option<f64>, sums: Option<Sums>, rss_bytes: Option<u64>) -> String {
    let Some(ns) = ns else {
        return "—".to_owned();
    };
    let timing = match sums {
        Some(Sums(server, total)) if server > 0.0 => {
            format!(
                "{} / {}",
                format_time(ns, 1, "", false),
                format_time(ns * total / server, 1, "", false)
            )
        }
        _ => format_time(ns, 1, "", false),
    };
    match rss_bytes {
        Some(bytes) => format!("{timing} ({:.1} MB)", bytes as f64 / (1024.0 * 1024.0)),
        None => timing,
    }
}

/// `times`, `sums` and `memory` are all keyed by criterion's benchmark id
fn table(
    fixture: &str,
    profiles: &BTreeSet<String>,
    times: &BTreeMap<String, f64>,
    sums: &BTreeMap<String, Sums>,
    memory: &BTreeMap<String, u64>,
) -> String {
    let method = |label: &str, id: &dyn Fn(&str) -> String| {
        std::iter::once(label.to_owned())
            .chain(profiles.iter().map(|p| {
                let id = id(p);
                cell(
                    times.get(&id).copied(),
                    sums.get(&id).copied(),
                    memory.get(&id).copied(),
                )
            }))
            .collect::<Vec<_>>()
    };

    let header: Vec<String> = std::iter::once("method".to_owned())
        .chain(profiles.iter().cloned())
        .collect();
    let separator = vec![" --- "; header.len()];
    let rows = vec![
        method("initialize", &|p| format!("initialize/{p}")),
        method("diagnostic", &|p| format!("diagnostics/{p}/{fixture}")),
        method("formatting", &|p| format!("formatting/{p}/{fixture}")),
        method("document symbols", &|p| {
            format!("document_symbols/{p}/{fixture}")
        }),
        method("folding ranges", &|p| {
            format!("folding_ranges/{p}/{fixture}")
        }),
        method("selection ranges", &|p| {
            format!("selection_ranges/{p}/{fixture}")
        }),
    ];
    format!(
        "Note: memory values are RSS (resident set size), shown in megabytes.\n\n{}",
        markdown_table(&header, &separator, &rows)
    )
}

/// Reads `cargo criterion --bench lsp --message-format=json` from stdin,
/// joins it with the RSS sidecar the bench wrote, and writes one
/// table per fixture to `benches/lsp/results/` for `readmes` to
/// stitch into `benches/lsp/lsp.md`.
pub fn write_lsp_bench_report() -> anyhow::Result<()> {
    let mut times = BTreeMap::new();
    for line in io::stdin().lock().lines() {
        if let Ok(Message::BenchmarkComplete { id, typical, .. }) = serde_json::from_str(&line?) {
            times.insert(id, typical.estimate);
        }
    }
    anyhow::ensure!(
        !times.is_empty(),
        "no benchmark-complete messages on stdin -- pipe in `cargo criterion --bench lsp --message-format=json`"
    );

    let sidecar_path = repo_path!("target/bench/lsp-memory.json");
    let memory: BTreeMap<String, u64> = serde_json::from_str(
        &fs::read_to_string(sidecar_path).with_context(|| format!("reading {sidecar_path}"))?,
    )
    .with_context(|| format!("parsing {sidecar_path}"))?;

    let timings_path = repo_path!("target/bench/lsp-timings.json");
    let sums: BTreeMap<String, Sums> = serde_json::from_str(
        &fs::read_to_string(timings_path).with_context(|| format!("reading {timings_path}"))?,
    )
    .with_context(|| format!("parsing {timings_path}"))?;

    let profiles: BTreeSet<String> = times
        .keys()
        .filter_map(|id| match id.split('/').collect::<Vec<_>>()[..] {
            [
                "initialize" | "diagnostics" | "formatting" | "document_symbols" | "folding_ranges"
                | "selection_ranges",
                profile,
                ..,
            ] => Some(profile.to_owned()),
            _ => None,
        })
        .collect();

    let results_dir = repo_path!("benches/lsp/results");
    fs::create_dir_all(results_dir).with_context(|| format!("creating {results_dir}"))?;
    for fixture in FIXTURES {
        let path = format!("{results_dir}/{fixture}.md");
        fs::write(&path, table(fixture, &profiles, &times, &sums, &memory))
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
        let sums = BTreeMap::from([("formatting/vscode-json/small".to_owned(), Sums(2.0, 6.0))]);
        let memory = BTreeMap::from([("initialize/jjpwrgem".to_owned(), 3 * 1024 * 1024)]);

        let out = table("small", &profiles, &times, &sums, &memory);

        assert!(out.contains("| method | jjpwrgem | vscode-json |"));
        assert!(out.contains("| initialize | 6.6ms (3.0 MB) | — |"));
        assert!(out.contains("| diagnostic | 200.0µs | — |"));
        assert!(out.contains("| formatting | — | 500.0µs / 1.5ms |"));
    }
}
