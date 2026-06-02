use std::{
    fs,
    io::Write as _,
    process::{Command, Stdio},
};

use anyhow::bail;

const BYTES2CHARS_README_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../crates/bytes2chars/README.md"
);
const BYTES2CHARS_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/bytes2chars.template.md"
));
const JSON_SCHEMA_README_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../crates/json-schema/README.md"
);
const JSON_SCHEMA_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/json-schema.template.md"
));
const RDME_START: &str = "<!-- cargo-rdme start -->";

fn rdme_content(readme: &str) -> &str {
    readme
        .find(RDME_START)
        .map(|pos| &readme[pos..])
        .unwrap_or(readme)
}

fn inject_pre_rdme(readme: &str, pre_rdme: &str) -> String {
    format!("{}\n\n{}", pre_rdme.trim_end(), rdme_content(readme))
}

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
const JSON_BENCH_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/json-bench.template.md"
));
const CLI_BENCH_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/cli-formatter-bench.template.md"
));
const SHARED_FRAGMENT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/indeterminate_handling.md"
));
const BENCH_SUMMARY: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/bench_summary.md"
));
const BENCH_INPUTS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/bench_inputs.md"
));
const BENCH_HARDWARE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/bench_hardware.md"
));
const FIXTURE_CANADA: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/fixture_canada.md"
));
const FIXTURE_CITM_CATALOG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/fixture_citm_catalog.md"
));
const FIXTURE_TWITTER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/fixture_twitter.md"
));
const FIXTURE_SMALL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/fixture_small.md"
));
const LSP_BENCH_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/lsp-bench.template.md"
));

const LSP_BENCH_OUT_PATH_STR: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../benches/lsp/README.md");
const LSP_BENCH_RESULTS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../benches/lsp/results");

fn generate_lsp_bench_readme() -> Option<String> {
    let canada = fs::read_to_string(format!("{LSP_BENCH_RESULTS_DIR}/canada.md")).ok()?;
    let twitter = fs::read_to_string(format!("{LSP_BENCH_RESULTS_DIR}/twitter.md")).ok()?;
    let small = fs::read_to_string(format!("{LSP_BENCH_RESULTS_DIR}/small.md")).ok()?;
    let citm_catalog =
        fs::read_to_string(format!("{LSP_BENCH_RESULTS_DIR}/citm_catalog.md")).ok()?;

    let replacements = [
        ("{{LSP_BENCH_CANADA_TABLE}}", canada.trim()),
        ("{{LSP_BENCH_TWITTER_TABLE}}", twitter.trim()),
        ("{{LSP_BENCH_SMALL_TABLE}}", small.trim()),
        ("{{LSP_BENCH_CITM_CATALOG_TABLE}}", citm_catalog.trim()),
    ];

    render_template(LSP_BENCH_TEMPLATE, &replacements).ok()
}

const COVERAGE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../benches/output/coverage.txt"
));

fn coverage_badge(pct_str: &str) -> String {
    let pct: f64 = pct_str.trim().parse().unwrap_or(0.0);
    let color = crate::badge::badge_color(pct);
    format!(
        "![coverage: {:.1}%](https://img.shields.io/badge/coverage-{:.1}%25-{})",
        pct, pct, color
    )
}

const ROOT_OUT_PATH_STR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../readme.md");
const PARSE_OUT_PATH_STR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../crates/parse/readme.md");
const BENCH_OUT_PATH_STR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../benches/BENCHMARKS.md");
const BYTES2CHARS_BENCH_OUT_PATH_STR: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../benches/utf8.md");
const JSON_BENCH_OUT_PATH_STR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../benches/json.md");
const CLI_BENCH_OUT_PATH_STR: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../benches/cli-formatter.md");

const EXISTING_ROOT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../readme.md"));
const EXISTING_PARSE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../crates/parse/readme.md"
));
const EXISTING_BENCH: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../benches/BENCHMARKS.md"
));
const EXISTING_BYTES2CHARS_BENCH: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../benches/utf8.md"));
const EXISTING_JSON_BENCH: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../benches/json.md"));
const EXISTING_CLI_BENCH: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../benches/cli-formatter.md"
));

const BYTES2CHARS_BENCH_TABLE_REPLACEMENTS: [(&str, &str); 1] = [(
    "{{BYTES2CHARS_BENCH_TABLE}}",
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../benches/output/bytes2chars.md"
    )),
)];

const JSON_BENCH_TABLE_REPLACEMENTS: [(&str, &str); 3] = [
    (
        "{{JSON_DESER_BENCH_TABLE}}",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../benches/output/json_deser.md"
        )),
    ),
    (
        "{{JSON_PRETTIFY_BENCH_TABLE}}",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../benches/output/json_prettify.md"
        )),
    ),
    (
        "{{JSON_UGLIFY_BENCH_TABLE}}",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../benches/output/json_uglify.md"
        )),
    ),
];

const BENCH_TABLE_REPLACEMENTS: [(&str, &str); 6] = [
    (
        "{{PRETTY_CANADA_TABLE}}",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../benches/docker/output/pretty-canada.md"
        )),
    ),
    (
        "{{UGLY_CANADA_TABLE}}",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../benches/docker/output/ugly-canada.md"
        )),
    ),
    (
        "{{PRETTY_CITM_CATALOG_TABLE}}",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../benches/docker/output/pretty-citm_catalog.md"
        )),
    ),
    (
        "{{UGLY_CITM_CATALOG_TABLE}}",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../benches/docker/output/ugly-citm_catalog.md"
        )),
    ),
    (
        "{{PRETTY_TWITTER_TABLE}}",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../benches/docker/output/pretty-twitter.md"
        )),
    ),
    (
        "{{UGLY_TWITTER_TABLE}}",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../benches/docker/output/ugly-twitter.md"
        )),
    ),
];

fn oxfmt_format(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut child = Command::new("pnpm")
        .args(["--silent", "exec", "oxfmt", "--stdin-filepath", "file.md"])
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
        .replace("{{CHECK_EXAMPLE}}", strip_front_matter(CHECK_EXAMPLE))
        .replace("{{BENCH_SUMMARY}}", BENCH_SUMMARY)
        .replace("{{BENCH_INPUTS}}", BENCH_INPUTS)
        .replace("{{BENCH_HARDWARE}}", BENCH_HARDWARE)
        .replace("{{FIXTURE_CANADA}}", FIXTURE_CANADA)
        .replace("{{FIXTURE_CITM_CATALOG}}", FIXTURE_CITM_CATALOG)
        .replace("{{FIXTURE_TWITTER}}", FIXTURE_TWITTER)
        .replace("{{FIXTURE_SMALL}}", FIXTURE_SMALL);

    for (needle, replacement) in replacements {
        processed = processed.replace(needle, replacement.trim());
    }

    let formatted = oxfmt_format(&format!("{}{}", BANNER, processed))?;
    Ok(formatted)
}

pub fn write_readmes() {
    let badge = coverage_badge(COVERAGE);
    let jsonlines_badge = crate::spec_badges::badge_for("jsonlines");
    let utf8_badge = crate::spec_badges::badge_for("utf8");
    let root_rendered =
        render_template(JJPWREGEM_TEMPLATE, &[("{{COVERAGE_BADGE}}", &badge)]).unwrap();
    let parse_rendered = render_template(
        JJPWREGEM_PARSE_TEMPLATE,
        &[("{{JSONLINES_SPEC_BADGE}}", &jsonlines_badge)],
    )
    .unwrap();
    let bench_rendered = render_template(BENCH_TEMPLATE, &[]).unwrap();
    let cli_bench_rendered =
        render_template(CLI_BENCH_TEMPLATE, &BENCH_TABLE_REPLACEMENTS).unwrap();
    let bytes2chars_bench_rendered = render_template(
        BYTES2CHARS_BENCH_TEMPLATE,
        &BYTES2CHARS_BENCH_TABLE_REPLACEMENTS,
    )
    .unwrap();
    let json_bench_rendered =
        render_template(JSON_BENCH_TEMPLATE, &JSON_BENCH_TABLE_REPLACEMENTS).unwrap();

    fs::write(ROOT_OUT_PATH_STR, root_rendered).unwrap();
    fs::write(PARSE_OUT_PATH_STR, parse_rendered).unwrap();
    fs::write(BENCH_OUT_PATH_STR, bench_rendered).unwrap();
    fs::write(CLI_BENCH_OUT_PATH_STR, cli_bench_rendered).unwrap();
    fs::write(BYTES2CHARS_BENCH_OUT_PATH_STR, bytes2chars_bench_rendered).unwrap();
    fs::write(JSON_BENCH_OUT_PATH_STR, json_bench_rendered).unwrap();
    let bytes2chars_pre_rdme = BYTES2CHARS_TEMPLATE.replace("{{UTF8_SPEC_BADGE}}", &utf8_badge);
    let existing = fs::read_to_string(BYTES2CHARS_README_PATH).unwrap();
    fs::write(
        BYTES2CHARS_README_PATH,
        inject_pre_rdme(&existing, &bytes2chars_pre_rdme),
    )
    .unwrap();
    let json_schema_badge = crate::spec_badges::badge_for("json-schema-v7");
    let json_schema_pre_rdme =
        JSON_SCHEMA_TEMPLATE.replace("{{JSON_SCHEMA_V7_SPEC_BADGE}}", &json_schema_badge);
    let existing = fs::read_to_string(JSON_SCHEMA_README_PATH).unwrap();
    fs::write(
        JSON_SCHEMA_README_PATH,
        inject_pre_rdme(&existing, &json_schema_pre_rdme),
    )
    .unwrap();
    if let Some(lsp_bench_rendered) = generate_lsp_bench_readme() {
        fs::write(LSP_BENCH_OUT_PATH_STR, lsp_bench_rendered).unwrap();
    }
}

fn check_readme(path: &str, existing: &str, generated: &str) -> anyhow::Result<()> {
    if existing == generated {
        return Ok(());
    }
    let diff = existing
        .lines()
        .zip(generated.lines())
        .enumerate()
        .find(|(_, (a, b))| a != b);
    if let Some((i, (a, b))) = diff {
        bail!(
            "{path} out of date — first diff at line {}\n  existing:  {a:?}\n  generated: {b:?}",
            i + 1
        );
    }
    bail!(
        "{path} out of date — length differs: {} vs {} chars",
        existing.len(),
        generated.len()
    );
}

pub fn are_readmes_updated() -> anyhow::Result<()> {
    let badge = coverage_badge(COVERAGE);
    let jsonlines_badge = crate::spec_badges::badge_for("jsonlines");
    let utf8_badge = crate::spec_badges::badge_for("utf8");
    let root_rendered =
        render_template(JJPWREGEM_TEMPLATE, &[("{{COVERAGE_BADGE}}", &badge)]).unwrap();
    let parse_rendered = render_template(
        JJPWREGEM_PARSE_TEMPLATE,
        &[("{{JSONLINES_SPEC_BADGE}}", &jsonlines_badge)],
    )
    .unwrap();
    let bench_rendered = render_template(BENCH_TEMPLATE, &[]).unwrap();
    let cli_bench_rendered =
        render_template(CLI_BENCH_TEMPLATE, &BENCH_TABLE_REPLACEMENTS).unwrap();
    let bytes2chars_bench_rendered = render_template(
        BYTES2CHARS_BENCH_TEMPLATE,
        &BYTES2CHARS_BENCH_TABLE_REPLACEMENTS,
    )
    .unwrap();
    let json_bench_rendered =
        render_template(JSON_BENCH_TEMPLATE, &JSON_BENCH_TABLE_REPLACEMENTS).unwrap();

    check_readme("readme.md", EXISTING_ROOT, &root_rendered)?;
    check_readme("crates/parse/readme.md", EXISTING_PARSE, &parse_rendered)?;
    check_readme("benches/BENCHMARKS.md", EXISTING_BENCH, &bench_rendered)?;
    check_readme(
        "benches/cli-formatter.md",
        EXISTING_CLI_BENCH,
        &cli_bench_rendered,
    )?;
    check_readme(
        "benches/utf8.md",
        EXISTING_BYTES2CHARS_BENCH,
        &bytes2chars_bench_rendered,
    )?;
    check_readme("benches/json.md", EXISTING_JSON_BENCH, &json_bench_rendered)?;
    let bytes2chars_pre_rdme = BYTES2CHARS_TEMPLATE.replace("{{UTF8_SPEC_BADGE}}", &utf8_badge);
    let existing = fs::read_to_string(BYTES2CHARS_README_PATH).unwrap_or_default();
    let generated = inject_pre_rdme(&existing, &bytes2chars_pre_rdme);
    check_readme("crates/bytes2chars/README.md", &existing, &generated)?;
    let json_schema_badge = crate::spec_badges::badge_for("json-schema-v7");
    let json_schema_pre_rdme =
        JSON_SCHEMA_TEMPLATE.replace("{{JSON_SCHEMA_V7_SPEC_BADGE}}", &json_schema_badge);
    let existing = fs::read_to_string(JSON_SCHEMA_README_PATH).unwrap_or_default();
    let generated = inject_pre_rdme(&existing, &json_schema_pre_rdme);
    check_readme("crates/json-schema/README.md", &existing, &generated)?;
    Ok(())
}
