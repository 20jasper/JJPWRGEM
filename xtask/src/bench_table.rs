use std::{
    collections::BTreeMap,
    io::{self, BufRead as _},
};

use serde::Deserialize;

#[derive(Deserialize)]
struct Estimate {
    estimate: f64,
}

#[derive(Deserialize)]
struct ThroughputEntry {
    per_iteration: f64,
}

#[derive(Deserialize)]
#[serde(tag = "reason")]
enum Message {
    #[serde(rename = "benchmark-complete")]
    BenchmarkComplete {
        id: String,
        typical: Estimate,
        throughput: Vec<ThroughputEntry>,
    },
    #[serde(other)]
    Other,
}

fn parse_id(id: &str) -> Option<(&str, &str, &str)> {
    let mut parts = id.splitn(3, '/');
    let group = parts.next()?;
    let bench = parts.next()?;
    let param = parts.next()?;
    Some((group, bench, param))
}

fn format_size(bytes: &str) -> String {
    match bytes.parse::<u64>() {
        Ok(b) if b >= 1024 * 1024 => format!("{} MiB", b / (1024 * 1024)),
        Ok(b) if b >= 1024 => format!("{} KiB", b / 1024),
        Ok(b) => format!("{b} B"),
        Err(_) => bytes.to_owned(),
    }
}

fn format_time(ns: f64) -> String {
    if ns >= 1_000_000.0 {
        format!("{:.2} ms", ns / 1_000_000.0)
    } else if ns >= 1_000.0 {
        format!("{:.2} µs", ns / 1_000.0)
    } else {
        format!("{:.2} ns", ns)
    }
}

fn format_throughput(bytes_per_iter: f64, ns: f64) -> String {
    let bytes_per_sec = bytes_per_iter / (ns / 1_000_000_000.0);
    let mib_per_sec = bytes_per_sec / (1024.0 * 1024.0);
    if mib_per_sec >= 1024.0 {
        format!("{:.2} GiB/s", mib_per_sec / 1024.0)
    } else {
        format!("{:.2} MiB/s", mib_per_sec)
    }
}

// group -> param -> bench -> (time_ns, bytes_per_iter)
type Data = BTreeMap<String, BTreeMap<String, BTreeMap<String, (f64, f64)>>>;

pub fn write_bench_table() -> anyhow::Result<()> {
    let mut data: Data = BTreeMap::new();

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        let Ok(msg) = serde_json::from_str::<Message>(&line) else {
            continue;
        };
        let Message::BenchmarkComplete {
            id,
            typical,
            throughput,
        } = msg
        else {
            continue;
        };
        let Some((group, bench, param)) = parse_id(&id) else {
            continue;
        };
        let bytes = throughput.first().map(|t| t.per_iteration).unwrap_or(0.0);
        data.entry(group.to_owned())
            .or_default()
            .entry(param.to_owned())
            .or_default()
            .insert(bench.to_owned(), (typical.estimate, bytes));
    }

    for (group, params) in &data {
        println!("\n## {group}");

        // collect bench names sorted slowest -> fastest by mean time across params
        let mut bench_names: Vec<String> = params
            .values()
            .next()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default();
        bench_names.sort_by(|a, b| {
            let mean = |name: &str| {
                let (sum, count) = params
                    .values()
                    .filter_map(|benches| benches.get(name).map(|&(ns, _)| ns))
                    .fold((0.0, 0usize), |(s, n), ns| (s + ns, n + 1));
                if count == 0 { 0.0 } else { sum / count as f64 }
            };
            mean(b)
                .partial_cmp(&mean(a))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut header = String::from("| |");
        let mut sep = String::from("|:--|");
        for bench in &bench_names {
            header.push_str(&format!(" `{bench}` |"));
            sep.push_str(":--:|");
        }

        println!("\n{header}");
        println!("{sep}");
        for (param, benches) in params {
            let size = format_size(param);
            let mut time_row = format!("| time ({size}) |");
            let mut throughput_row = format!("| throughput ({size}) |");
            for bench in &bench_names {
                if let Some(&(ns, bytes)) = benches.get(bench) {
                    time_row.push_str(&format!(" {} |", format_time(ns)));
                    throughput_row.push_str(&format!(" {} |", format_throughput(bytes, ns)));
                } else {
                    time_row.push_str(" — |");
                    throughput_row.push_str(" — |");
                }
            }
            println!("{time_row}");
            println!("{throughput_row}");
        }
    }

    Ok(())
}
