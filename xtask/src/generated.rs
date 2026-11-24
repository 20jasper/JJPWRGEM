use std::{
    fs,
    io::Write as _,
    process::{Command, Stdio},
};

use anyhow::{Result, anyhow, bail};
use jjpwrgem_parse::format::{DEFAULT_PREFERRED_WIDTH, LineEnding, prettify_str};

pub fn oxfmt_format(input: &str, filepath: &str) -> Result<String> {
    let mut child = Command::new("oxfmt")
        .args(["--stdin-filepath", filepath])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    child
        .stdin
        .as_mut()
        .ok_or_else(|| anyhow!("failed to open oxfmt stdin"))?
        .write_all(input.as_bytes())?;

    let output = child.wait_with_output()?;
    if !output.status.success() {
        bail!("oxfmt failed: {}", output.status);
    }

    Ok(String::from_utf8(output.stdout)?)
}

pub fn write_or_check(path: &str, generated: &str, command: &str, check: bool) -> Result<()> {
    if !check {
        return Ok(fs::write(path, generated)?);
    }
    let existing = fs::read_to_string(path)?;
    if existing == generated {
        return Ok(());
    }
    let diff = existing
        .lines()
        .zip(generated.lines())
        .enumerate()
        .find(|(_, (a, b))| a != b);
    let detail = match diff {
        Some((i, (a, b))) => format!(
            "first diff at line {}\n  existing:  {a:?}\n  generated: {b:?}",
            i + 1
        ),
        None => format!(
            "length differs: {} vs {} chars",
            existing.len(),
            generated.len()
        ),
    };
    bail!("{path} out of date, {detail}\nrun `cargo xtask {command}`")
}

pub fn pretty_json(value: &impl serde::Serialize) -> Result<String> {
    let compact = jjpwrgem_parse::format::serde::uglify_serializable(value)
        .map_err(|error| anyhow!("{error}"))?;
    let pretty = prettify_str(&compact, DEFAULT_PREFERRED_WIDTH, LineEnding::Lf)
        .map_err(|error| anyhow!("{error}"))?;
    Ok(pretty + "\n")
}
