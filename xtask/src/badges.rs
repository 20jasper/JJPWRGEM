use std::process::Command;

use anyhow::Context;
use serde::Deserialize;

fn badge_color(pct: f64) -> &'static str {
    match pct as u32 {
        90..=100 => "brightgreen",
        80..=89 => "green",
        70..=79 => "yellowgreen",
        60..=69 => "yellow",
        50..=59 => "orange",
        _ => "red",
    }
}

#[derive(Deserialize)]
struct TraceyStatus {
    impls: Vec<ImplStatus>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImplStatus {
    spec: String,
    total_rules: u32,
    covered_rules: u32,
    verified_rules: u32,
}

impl ImplStatus {
    /// Rules that have both an implementation reference and a verification reference.
    fn both_count(&self) -> u32 {
        self.covered_rules.min(self.verified_rules)
    }
}

fn tracey_status() -> anyhow::Result<Option<TraceyStatus>> {
    let output = match Command::new("tracey")
        .args(["query", "--json", "status"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return Ok(None),
    };
    if !output.status.success() {
        return Ok(None);
    }
    let status =
        serde_json::from_slice(&output.stdout).context("failed to parse tracey status JSON")?;
    Ok(Some(status))
}

fn spec_badge(spec: &str, covered: u32, total: u32) -> String {
    let pct = if total == 0 {
        0.0
    } else {
        covered as f64 / total as f64 * 100.0
    };
    let color = badge_color(pct);
    let pct_int = pct.round() as u32;
    // shields.io static badge URL: literal `-` in label/message must be doubled to `--`
    // `%25` is URL-encoded `%`
    let label = format!("spec:{}", spec.replace('-', "--"));
    format!(
        "[![{spec}: {pct_int}%](https://img.shields.io/badge/{label}-{pct_int}%25-{color})](https://github.com/20jasper/JJPWRGEM/blob/main/spec/{spec}.md)"
    )
}

pub fn coverage_badge(pct_str: &str) -> String {
    let pct: f64 = pct_str.trim().parse().unwrap_or(0.0);
    let color = badge_color(pct);
    format!(
        "![coverage: {:.1}%](https://img.shields.io/badge/coverage-{:.1}%25-{})",
        pct, pct, color
    )
}

pub fn write_spec_badges() -> anyhow::Result<()> {
    let status =
        tracey_status()?.ok_or_else(|| anyhow::anyhow!("tracey not available or failed"))?;

    for s in &status.impls {
        println!("{}", spec_badge(&s.spec, s.both_count(), s.total_rules));
    }

    Ok(())
}

/// Badge markdown for a single spec, or empty string if tracey unavailable.
pub fn badge_for(spec: &str) -> String {
    tracey_status()
        .ok()
        .flatten()
        .as_ref()
        .and_then(|status| status.impls.iter().find(|i| i.spec == spec))
        .map(|i| spec_badge(&i.spec, i.both_count(), i.total_rules))
        .unwrap_or_default()
}
