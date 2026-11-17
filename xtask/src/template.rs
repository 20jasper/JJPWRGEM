use std::{
    fs,
    io::Write as _,
    process::{Command, Stdio},
};

use anyhow::{Result, anyhow, bail};

use crate::settings_docs::{helix_format_config, settings_reference};

const BYTES2CHARS_README_PATH: &str = repo_path!("crates/bytes2chars/README.md");
const BYTES2CHARS_TEMPLATE: &str = template!("bytes2chars.template.md");
const JSON_SCHEMA_README_PATH: &str = repo_path!("crates/json-schema/README.md");
const JSON_SCHEMA_TEMPLATE: &str = template!("json-schema.template.md");
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
const CHECK_EXAMPLE: &str =
    include_str!(repo_path!("tests/integration/snapshots/check_failure.snap"));

const BANNER: &str = "<!-- GENERATED FILE - update the templates in the xtask -->\n\n";

const INSTALL: &str = template!("install.md");
const LSP_PERF_CLAIM: &str = "[On large JSON files, diagnostics are up to ~5x faster and use ~20–25x less RAM than VSCode's LSP](https://github.com/20jasper/JJPWRGEM/blob/main/benches/lsp/lsp.md)";
const LSP_DEMO_ALT: &str = "animation of JJPWRGEM's LSP in a 60,000 line file. file changes are made quickly and feedback is shown quickly. code actions fix common issues like missing colons";
const LSP_DEMO_URL: &str = "https://raw.githubusercontent.com/20jasper/JJPWRGEM/main/npm-packages/jjpwrgem-vscode/vscode-jjp.gif";
const LSP_FEATURES: &str = template!("lsp_features.md");

const JJPWRGEM_TEMPLATE: &str = template!("root.template.md");
const JJPWRGEM_PARSE_TEMPLATE: &str = template!("parse.template.md");
const BENCH_TEMPLATE: &str = template!("bench.template.md");
const BYTES2CHARS_BENCH_TEMPLATE: &str = template!("bytes2chars-bench.template.md");
const JSON_BENCH_TEMPLATE: &str = template!("json-bench.template.md");
const CLI_BENCH_TEMPLATE: &str = template!("cli-formatter-bench.template.md");
const SHARED_FRAGMENT: &str = template!("indeterminate_handling.md");
const BENCH_SUMMARY: &str = template!("bench_summary.md");
const BENCH_INPUTS: &str = template!("bench_inputs.md");
const BENCH_HARDWARE: &str = template!("bench_hardware.md");
const FIXTURE_CANADA: &str = template!("fixture_canada.md");
const FIXTURE_CITM_CATALOG: &str = template!("fixture_citm_catalog.md");
const FIXTURE_TWITTER: &str = template!("fixture_twitter.md");
const FIXTURE_SMALL: &str = template!("fixture_small.md");
const LSP_BENCH_TEMPLATE: &str = template!("lsp-bench.template.md");
const VSCODE_TEMPLATE: &str = template!("vscode.template.md");
const VSCODE_OUT_PATH_STR: &str = repo_path!("npm-packages/jjpwrgem-vscode/README.md");
const LSP_CRATE_TEMPLATE: &str = template!("lsp_crate.template.md");
const LSP_CRATE_OUT_PATH_STR: &str = repo_path!("crates/lsp/readme.md");
const LSP_BENCH_OUT_PATH_STR: &str = repo_path!("benches/lsp/lsp.md");
const LSP_BENCH_RESULTS_DIR: &str = repo_path!("benches/lsp/results");

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

const COVERAGE: &str = include_str!(repo_path!("benches/output/coverage.txt"));

fn coverage_badge(pct_str: &str) -> String {
    let pct: f64 = pct_str.trim().parse().unwrap_or(0.0);
    let color = crate::badge::badge_color(pct);
    format!(
        "![coverage: {:.1}%](https://img.shields.io/badge/coverage-{:.1}%25-{})",
        pct, pct, color
    )
}

const ROOT_OUT_PATH_STR: &str = repo_path!("readme.md");
const PARSE_OUT_PATH_STR: &str = repo_path!("crates/parse/readme.md");
const BENCH_OUT_PATH_STR: &str = repo_path!("benches/BENCHMARKS.md");
const BYTES2CHARS_BENCH_OUT_PATH_STR: &str = repo_path!("benches/utf8.md");
const JSON_BENCH_OUT_PATH_STR: &str = repo_path!("benches/json.md");
const CLI_BENCH_OUT_PATH_STR: &str = repo_path!("benches/cli-formatter.md");

const BYTES2CHARS_BENCH_TABLE_REPLACEMENTS: [(&str, &str); 1] = [(
    "{{BYTES2CHARS_BENCH_TABLE}}",
    include_str!(repo_path!("benches/output/bytes2chars.md")),
)];

const JSON_BENCH_TABLE_REPLACEMENTS: [(&str, &str); 4] = [
    (
        "{{JSON_DESER_BENCH_TABLE}}",
        include_str!(repo_path!("benches/output/json_deser.md")),
    ),
    (
        "{{JSON_PRETTIFY_BENCH_TABLE}}",
        include_str!(repo_path!("benches/output/json_prettify.md")),
    ),
    (
        "{{JSON_UGLIFY_BENCH_TABLE}}",
        include_str!(repo_path!("benches/output/json_uglify.md")),
    ),
    (
        "{{JSON_SERIALIZE_BENCH_TABLE}}",
        include_str!(repo_path!("benches/output/json_serialize.md")),
    ),
];

const BENCH_TABLE_REPLACEMENTS: [(&str, &str); 6] = [
    (
        "{{PRETTY_CANADA_TABLE}}",
        include_str!(repo_path!("benches/docker/output/pretty-canada.md")),
    ),
    (
        "{{UGLY_CANADA_TABLE}}",
        include_str!(repo_path!("benches/docker/output/ugly-canada.md")),
    ),
    (
        "{{PRETTY_CITM_CATALOG_TABLE}}",
        include_str!(repo_path!("benches/docker/output/pretty-citm_catalog.md")),
    ),
    (
        "{{UGLY_CITM_CATALOG_TABLE}}",
        include_str!(repo_path!("benches/docker/output/ugly-citm_catalog.md")),
    ),
    (
        "{{PRETTY_TWITTER_TABLE}}",
        include_str!(repo_path!("benches/docker/output/pretty-twitter.md")),
    ),
    (
        "{{UGLY_TWITTER_TABLE}}",
        include_str!(repo_path!("benches/docker/output/ugly-twitter.md")),
    ),
];

fn oxfmt_format(input: &str) -> Result<String> {
    let mut child = Command::new("oxfmt")
        .args(["--stdin-filepath", "file.md"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow!("failed to open oxfmt stdin"))?;
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        bail!("oxfmt failed: {}", output.status);
    }

    let formatted = String::from_utf8(output.stdout)?;
    Ok(formatted)
}

fn render_template(template: &str, replacements: &[(&str, &str)]) -> Result<String> {
    let mut processed = template
        .replace("{{IND}}", SHARED_FRAGMENT)
        .replace("{{CHECK_EXAMPLE}}", strip_front_matter(CHECK_EXAMPLE))
        .replace(
            "{{INSTALL}}",
            &INSTALL
                .trim()
                .replace("{{AT_VERSION}}", "")
                .replace("{{CARGO_VERSION}}", ""),
        )
        .replace("{{LSP_PERF_CLAIM}}", LSP_PERF_CLAIM)
        .replace("{{LSP_DEMO_ALT}}", LSP_DEMO_ALT)
        .replace("{{LSP_DEMO_URL}}", LSP_DEMO_URL)
        .replace("{{LSP_FEATURES}}", LSP_FEATURES.trim())
        .replace("{{HELIX_FORMAT_CONFIG}}", helix_format_config().trim_end())
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

fn generated_readmes() -> Result<Vec<(&'static str, String)>> {
    let badge = coverage_badge(COVERAGE);
    let jsonlines_badge = crate::spec_badges::badge_for("jsonlines");
    let utf8_badge = crate::spec_badges::badge_for("utf8");
    let json_schema_badge = crate::spec_badges::badge_for("json-schema-v7");

    let bytes2chars_pre_rdme = BYTES2CHARS_TEMPLATE.replace("{{UTF8_SPEC_BADGE}}", &utf8_badge);
    let json_schema_pre_rdme =
        JSON_SCHEMA_TEMPLATE.replace("{{JSON_SCHEMA_V7_SPEC_BADGE}}", &json_schema_badge);

    let mut readmes = vec![
        (
            ROOT_OUT_PATH_STR,
            render_template(JJPWRGEM_TEMPLATE, &[("{{COVERAGE_BADGE}}", &badge)])?,
        ),
        (
            PARSE_OUT_PATH_STR,
            render_template(
                JJPWRGEM_PARSE_TEMPLATE,
                &[("{{JSONLINES_SPEC_BADGE}}", &jsonlines_badge)],
            )?,
        ),
        (BENCH_OUT_PATH_STR, render_template(BENCH_TEMPLATE, &[])?),
        (
            CLI_BENCH_OUT_PATH_STR,
            render_template(CLI_BENCH_TEMPLATE, &BENCH_TABLE_REPLACEMENTS)?,
        ),
        (
            BYTES2CHARS_BENCH_OUT_PATH_STR,
            render_template(
                BYTES2CHARS_BENCH_TEMPLATE,
                &BYTES2CHARS_BENCH_TABLE_REPLACEMENTS,
            )?,
        ),
        (
            JSON_BENCH_OUT_PATH_STR,
            render_template(JSON_BENCH_TEMPLATE, &JSON_BENCH_TABLE_REPLACEMENTS)?,
        ),
        (
            VSCODE_OUT_PATH_STR,
            render_template(
                VSCODE_TEMPLATE,
                &[(
                    "{{SETTINGS_REFERENCE}}",
                    &settings_reference("jjpwrgem.format."),
                )],
            )?,
        ),
        (
            LSP_CRATE_OUT_PATH_STR,
            render_template(
                LSP_CRATE_TEMPLATE,
                &[("{{SETTINGS_REFERENCE}}", &settings_reference(""))],
            )?,
        ),
        (
            BYTES2CHARS_README_PATH,
            inject_pre_rdme(
                &fs::read_to_string(BYTES2CHARS_README_PATH)?,
                &bytes2chars_pre_rdme,
            ),
        ),
        (
            JSON_SCHEMA_README_PATH,
            inject_pre_rdme(
                &fs::read_to_string(JSON_SCHEMA_README_PATH)?,
                &json_schema_pre_rdme,
            ),
        ),
    ];
    if let Some(lsp_bench_rendered) = generate_lsp_bench_readme() {
        readmes.push((LSP_BENCH_OUT_PATH_STR, lsp_bench_rendered));
    }
    Ok(readmes)
}

pub fn write_readmes() -> Result<()> {
    for (path, contents) in generated_readmes()? {
        fs::write(path, contents)?;
    }
    Ok(())
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

pub fn are_readmes_updated() -> Result<()> {
    for (path, generated) in generated_readmes()? {
        check_readme(path, &fs::read_to_string(path)?, &generated)?;
    }
    Ok(())
}
