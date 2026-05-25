use std::fs;

use anyhow::Context;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct Results {
    benchmarks: Vec<Benchmark>,
}

#[derive(Deserialize)]
struct Benchmark {
    name: String,
    servers: Vec<ServerResult>,
}

#[derive(Deserialize)]
struct ServerResult {
    server: String,
    status: String,
    p50_ms: Option<f64>,
    rss_kb: Option<u64>,
    response: Option<Value>,
}

fn format_edit_count(response: &Option<Value>) -> String {
    match response {
        Some(Value::Array(edits)) => {
            let n = edits.len();
            if n == 1 {
                "1 edit".to_string()
            } else {
                format!("{n} edits")
            }
        }
        _ => "—".to_string(),
    }
}

pub fn write_lsp_bench_report(name: &str) -> anyhow::Result<()> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let input_path = format!("{manifest_dir}/../benches/lsp/benchmarks/results.json");
    let input = fs::read_to_string(&input_path).with_context(|| format!("reading {input_path}"))?;
    let results: Results = serde_json::from_str(&input)?;

    let mut out = String::new();
    out.push_str("Note: memory values are RSS (resident set size), shown in megabytes.\n\n");

    let servers: Vec<&str> = results.benchmarks[0]
        .servers
        .iter()
        .map(|s| s.server.as_str())
        .collect();

    let header = servers.join(" | ");
    out.push_str(&format!("| method | {header} |\n"));
    out.push_str(&format!(
        "|--------|{}|\n",
        servers.iter().map(|_| "---").collect::<Vec<_>>().join("|")
    ));

    for bench in &results.benchmarks {
        let bname = bench.name.replace("textDocument/", "");
        let cells: Vec<String> = bench
            .servers
            .iter()
            .map(|s| {
                if s.status != "ok" {
                    return "—".into();
                }
                match (s.p50_ms, s.rss_kb) {
                    (Some(ms), Some(kb)) if kb > 0 => {
                        format!("{:.1}ms ({:.1} MB)", ms, kb as f64 / 1024.0)
                    }
                    (Some(ms), _) => format!("{:.1}ms", ms),
                    _ => "—".into(),
                }
            })
            .collect();
        out.push_str(&format!("| {bname} | {} |\n", cells.join(" | ")));
    }

    if let Some(formatting) = results
        .benchmarks
        .iter()
        .find(|b| b.name == "textDocument/formatting")
    {
        let edit_cells: Vec<String> = formatting
            .servers
            .iter()
            .map(|s| {
                if s.status != "ok" {
                    return "—".to_string();
                }
                format_edit_count(&s.response)
            })
            .collect();
        out.push_str(&format!(
            "| formatting edits | {} |\n",
            edit_cells.join(" | ")
        ));
    }

    let output_path = format!("{manifest_dir}/../benches/lsp/results/{name}.md");
    fs::write(&output_path, &out).with_context(|| format!("writing {output_path}"))?;
    println!("wrote {output_path}");

    let benchmarks_dir = format!("{manifest_dir}/../benches/lsp/benchmarks");
    fs::remove_dir_all(&benchmarks_dir).with_context(|| format!("removing {benchmarks_dir}"))?;

    Ok(())
}
