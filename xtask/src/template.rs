use std::{
    fs,
    io::Write as _,
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
    "/../tests/integration/snapshots/check_failure.snap"
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
const BYTES2CHARS_BENCH_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/bytes2chars-bench.template.md"
));
const SHARED_FRAGMENT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/indeterminate_handling.md"
));

const ROOT_OUT_PATH_STR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../readme.md");
const PARSE_OUT_PATH_STR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../crates/parse/readme.md");
const BENCH_OUT_PATH_STR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../benchmarks.md");
const BYTES2CHARS_BENCH_OUT_PATH_STR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../crates/bytes2chars/BENCHMARKS.md"
);

const EXISTING_ROOT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../readme.md"));
const EXISTING_PARSE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../crates/parse/readme.md"
));
const EXISTING_BENCH: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../benchmarks.md"));
const EXISTING_BYTES2CHARS_BENCH: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../crates/bytes2chars/BENCHMARKS.md"
));

const BYTES2CHARS_BENCH_TABLE_REPLACEMENTS: [(&str, &str); 1] = [(
    "{{BYTES2CHARS_BENCH_TABLE}}",
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../benches/output/bytes2chars.md"
    )),
)];

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

fn oxfmt_format(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut child = Command::new("pnpm")
        .args(["exec", "oxfmt", "--stdin-filepath", "file.md"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    {
        let stdin = child.stdin.as_mut().ok_or("failed to open oxfmt stdin")?;
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(format!("oxfmt failed: {}", output.status).into());
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

    let formatted = oxfmt_format(&format!("{}{}", BANNER, processed))?;
    Ok(formatted)
}

pub fn write_readmes() {
    let root_rendered = render_template(JJPWREGEM_TEMPLATE, &[]).unwrap();
    let parse_rendered = render_template(JJPWREGEM_PARSE_TEMPLATE, &[]).unwrap();
    let bench_rendered = render_template(BENCH_TEMPLATE, &BENCH_TABLE_REPLACEMENTS).unwrap();
    let bytes2chars_bench_rendered = render_template(
        BYTES2CHARS_BENCH_TEMPLATE,
        &BYTES2CHARS_BENCH_TABLE_REPLACEMENTS,
    )
    .unwrap();

    fs::write(ROOT_OUT_PATH_STR, root_rendered).unwrap();
    fs::write(PARSE_OUT_PATH_STR, parse_rendered).unwrap();
    fs::write(BENCH_OUT_PATH_STR, bench_rendered).unwrap();
    fs::write(BYTES2CHARS_BENCH_OUT_PATH_STR, bytes2chars_bench_rendered).unwrap();
}

pub fn are_readmes_updated() -> anyhow::Result<()> {
    let root_rendered = render_template(JJPWREGEM_TEMPLATE, &[]).unwrap();
    let parse_rendered = render_template(JJPWREGEM_PARSE_TEMPLATE, &[]).unwrap();
    let bench_rendered = render_template(BENCH_TEMPLATE, &BENCH_TABLE_REPLACEMENTS).unwrap();
    let bytes2chars_bench_rendered = render_template(
        BYTES2CHARS_BENCH_TEMPLATE,
        &BYTES2CHARS_BENCH_TABLE_REPLACEMENTS,
    )
    .unwrap();

    if EXISTING_ROOT != root_rendered {
        bail!("readme.md out of date (root)")
    } else if EXISTING_PARSE != parse_rendered {
        bail!("crates/parse/readme.md out of date")
    } else if EXISTING_BENCH != bench_rendered {
        bail!("xtask/bench/readme.md out of date")
    } else if EXISTING_BYTES2CHARS_BENCH != bytes2chars_bench_rendered {
        bail!("crates/bytes2chars/BENCHMARKS.md out of date")
    } else {
        Ok(())
    }
}
