use std::process::Command;

use anyhow::{Context, bail};
use serde::Deserialize;

#[derive(Deserialize)]
struct StatusOutput {
    impls: Vec<ImplStatus>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImplStatus {
    spec: String,
    total_rules: u32,
    covered_rules: u32,
}

fn spec_badge(spec: &str, covered: u32, total: u32) -> String {
    let pct = if total == 0 {
        0.0
    } else {
        covered as f64 / total as f64 * 100.0
    };
    let color = crate::badge::badge_color(pct);
    // shields.io static badge URL: literal `-` in label/message must be doubled to `--`
    let label = format!("spec:{}", spec.replace('-', "--"));
    let message = format!("{covered}%2F{total}");
    format!(
        "[![{spec}: {covered}/{total}](https://img.shields.io/badge/{label}-{message}-{color})](spec/{spec}.md)"
    )
}

pub fn write_spec_badges() -> anyhow::Result<()> {
    let output = Command::new("tracey")
        .args(["query", "--json", "status"])
        .output()
        .context("failed to run `tracey query --json status`")?;

    if !output.status.success() {
        bail!(
            "tracey query failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let status: StatusOutput =
        serde_json::from_slice(&output.stdout).context("failed to parse tracey status JSON")?;

    for s in &status.impls {
        println!("{}", spec_badge(&s.spec, s.covered_rules, s.total_rules));
    }

    Ok(())
}
