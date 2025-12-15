use std::io::Write as _;
use std::{
    fs,
    process::{Command, Stdio},
};

use anyhow::bail;

fn strip_front_matter(raw: &str) -> &str {
    const FRONT_MATTER_SEP: &str = "\n---\n";
    raw.split_once(FRONT_MATTER_SEP)
        .expect("snapshots should always have a separator")
        .1
}
const CHECK_EXAMPLE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../tests/integration/commands/docs/snapshots/check_failure.snap"
));

const BANNER: &str = "<!-- GENERATED FILE - update the templates in the xtask -->\n\n";

const JJPWREGEM_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/root.template.md"
));
const JJPWREGEM_PARSE_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/parse.template.md"
));
const BENCH_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/bench.template.md"
));
const SHARED_FRAGMENT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/indeterminate_handling.md"
));

const ROOT_OUT_PATH_STR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../readme.md");
const PARSE_OUT_PATH_STR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../crates/parse/readme.md");
const BENCH_OUT_PATH_STR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../benchmarks.md");

const EXISTING_ROOT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../readme.md"));
const EXISTING_PARSE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../crates/parse/readme.md"
));
const EXISTING_BENCH: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../benchmarks.md"));

const BENCH_TABLE_REPLACEMENTS: [(&str, &str); 6] = [
    (
        "{{PRETTY_CANADA_TABLE}}",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/bench/output/pretty-canada.md"
        )),
    ),
    (
        "{{UGLY_CANADA_TABLE}}",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/bench/output/ugly-canada.md"
        )),
    ),
    (
        "{{PRETTY_CITM_CATALOG_TABLE}}",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/bench/output/pretty-citm_catalog.md"
        )),
    ),
    (
        "{{UGLY_CITM_CATALOG_TABLE}}",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/bench/output/ugly-citm_catalog.md"
        )),
    ),
    (
        "{{PRETTY_TWITTER_TABLE}}",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/bench/output/pretty-twitter.md"
        )),
    ),
    (
        "{{UGLY_TWITTER_TABLE}}",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/bench/output/ugly-twitter.md"
        )),
    ),
];

fn prettier_format(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut child = Command::new("npx")
        .arg("prettier")
        .arg("--parser")
        .arg("markdown")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or("failed to open prettier stdin")?;
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(format!("prettier failed: {}", output.status).into());
    }

    let formatted = String::from_utf8(output.stdout)?;
    Ok(formatted)
}

fn render_template(
    template: &str,
    replacements: &[(&str, &str)],
) -> Result<String, Box<dyn std::error::Error>> {
    let mut processed = template
        .replace("{{IND}}", SHARED_FRAGMENT)
        .replace("{{CHECK_EXAMPLE}}", strip_front_matter(CHECK_EXAMPLE));

    for (needle, replacement) in replacements {
        processed = processed.replace(needle, replacement.trim());
    }

    let with_banner = format!("{}{}", BANNER, processed);
    let formatted = prettier_format(&with_banner)?;
    Ok(formatted)
}

pub fn write_readmes() {
    let root_rendered = render_template(JJPWREGEM_TEMPLATE, &[]).unwrap();
    let parse_rendered = render_template(JJPWREGEM_PARSE_TEMPLATE, &[]).unwrap();
    let bench_rendered = render_template(BENCH_TEMPLATE, &BENCH_TABLE_REPLACEMENTS).unwrap();

    fs::write(ROOT_OUT_PATH_STR, root_rendered).unwrap();
    fs::write(PARSE_OUT_PATH_STR, parse_rendered).unwrap();
    fs::write(BENCH_OUT_PATH_STR, bench_rendered).unwrap();
}

pub fn are_readmes_updated() -> anyhow::Result<()> {
    let root_rendered = render_template(JJPWREGEM_TEMPLATE, &[]).unwrap();
    let parse_rendered = render_template(JJPWREGEM_PARSE_TEMPLATE, &[]).unwrap();
    let bench_rendered = render_template(BENCH_TEMPLATE, &BENCH_TABLE_REPLACEMENTS).unwrap();

    if EXISTING_ROOT != root_rendered {
        bail!("readme.md out of date (root)")
    } else if EXISTING_PARSE != parse_rendered {
        bail!("crates/parse/readme.md out of date")
    } else if EXISTING_BENCH != bench_rendered {
        bail!("xtask/bench/readme.md out of date")
    } else {
        Ok(())
    }
}
