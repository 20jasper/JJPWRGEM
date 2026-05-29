use crate::badge::{find_spec, spec_badge, tracey_status};

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
        .and_then(|s| find_spec(s, spec))
        .map(|i| spec_badge(&i.spec, i.both_count(), i.total_rules))
        .unwrap_or_default()
}
