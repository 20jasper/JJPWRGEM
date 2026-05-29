use std::process::Command;

use anyhow::Context;
use serde::Deserialize;

pub fn badge_color(pct: f64) -> &'static str {
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
pub struct TraceyStatus {
    pub impls: Vec<ImplStatus>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImplStatus {
    pub spec: String,
    pub total_rules: u32,
    pub covered_rules: u32,
    pub verified_rules: u32,
}

impl ImplStatus {
    /// Rules that have both an implementation reference and a verification reference.
    pub fn both_count(&self) -> u32 {
        self.covered_rules.min(self.verified_rules)
    }
}

pub fn tracey_status() -> anyhow::Result<Option<TraceyStatus>> {
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

pub fn spec_badge(spec: &str, covered: u32, total: u32) -> String {
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
        "[![{spec}: {pct_int}%](https://img.shields.io/badge/{label}-{pct_int}%25-{color})](spec/{spec}.md)"
    )
}

pub fn find_spec<'a>(status: &'a TraceyStatus, spec: &str) -> Option<&'a ImplStatus> {
    status.impls.iter().find(|i| i.spec == spec)
}
