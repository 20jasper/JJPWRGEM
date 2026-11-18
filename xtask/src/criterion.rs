use serde::Deserialize;

#[derive(Deserialize)]
pub struct Estimate {
    pub estimate: f64,
}

#[derive(Deserialize)]
pub struct ThroughputEntry {
    pub per_iteration: f64,
}

#[derive(Deserialize)]
#[serde(tag = "reason")]
pub enum Message {
    #[serde(rename = "benchmark-complete")]
    BenchmarkComplete {
        id: String,
        typical: Estimate,
        #[serde(default)]
        throughput: Vec<ThroughputEntry>,
    },
    #[serde(other)]
    Other,
}

pub fn format_time(ns: f64, decimals: usize, separator: &str, nanos: bool) -> String {
    let (value, unit) = if ns >= 1_000_000.0 {
        (ns / 1_000_000.0, "ms")
    } else if ns >= 1_000.0 || !nanos {
        (ns / 1_000.0, "µs")
    } else {
        (ns, "ns")
    };
    format!("{value:.decimals$}{separator}{unit}")
}

pub fn markdown_table(header: &[String], separator: &[&str], rows: &[Vec<String>]) -> String {
    let line = |cells: &[String]| {
        cells.iter().fold(String::from("|"), |mut line, cell| {
            if !cell.is_empty() {
                line.push(' ');
                line.push_str(cell);
            }
            line.push_str(" |");
            line
        })
    };
    let mut out = format!("{}\n|{}|\n", line(header), separator.join("|"));
    for row in rows {
        out.push_str(&line(row));
        out.push('\n');
    }
    out
}
