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
