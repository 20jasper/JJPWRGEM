use std::fs;

use anyhow::{Context as _, Result};

use crate::{
    badges::{badge_for, coverage_badge},
    generated::{oxfmt_format, write_or_check},
    settings_docs::{helix_format_config, settings_reference},
};

const RDME_START: &str = "<!-- cargo-rdme start -->";

fn strip_front_matter(raw: &str) -> Result<&str> {
    const FRONT_MATTER_SEP: &str = "\n---\n";
    raw.split_once(FRONT_MATTER_SEP)
        .map(|(_, body)| body)
        .context("snapshots should always have a separator")
}
const CHECK_EXAMPLE: &str =
    include_str!(repo_path!("tests/integration/snapshots/check_failure.snap"));

const BANNER: &str = "<!-- GENERATED FILE - update the templates in the xtask -->\n\n";

const INSTALL_QUICK: &str = template!("install_quick.md");
const LSP_PERF_CLAIM: &str = "[On large JSON files, diagnostics are up to ~5x faster and use ~20–25x less RAM than VSCode's LSP](https://github.com/20jasper/JJPWRGEM/blob/main/benches/lsp/lsp.md)";
const LSP_DEMO_ALT: &str = "animation of JJPWRGEM's LSP in a 60,000 line file. file changes are made quickly and feedback is shown quickly. code actions fix common issues like missing colons";
const LSP_DEMO_URL: &str = "https://raw.githubusercontent.com/20jasper/JJPWRGEM/main/npm-packages/jjpwrgem-vscode/vscode-jjp.gif";
const LSP_FEATURES: &str = template!("lsp_features.md");

const SHARED_FRAGMENT: &str = template!("indeterminate_handling.md");
const BENCH_SUMMARY: &str = template!("bench_summary.md");
const BENCH_INPUTS: &str = template!("bench_inputs.md");
const BENCH_HARDWARE: &str = template!("bench_hardware.md");
const FIXTURE_CANADA: &str = template!("fixture_canada.md");
const FIXTURE_CITM_CATALOG: &str = template!("fixture_citm_catalog.md");
const FIXTURE_TWITTER: &str = template!("fixture_twitter.md");
const FIXTURE_SMALL: &str = template!("fixture_small.md");
const LSP_BENCH_TEMPLATE: &str = template!("lsp-bench.template.md");
const LSP_BENCH_OUT_PATH_STR: &str = repo_path!("benches/lsp/lsp.md");
const LSP_BENCH_RESULTS_DIR: &str = repo_path!("benches/lsp/results");

fn generate_lsp_bench_readme() -> Result<Option<String>> {
    let read = |name: &str| fs::read_to_string(format!("{LSP_BENCH_RESULTS_DIR}/{name}.md")).ok();
    let (Some(canada), Some(twitter), Some(small), Some(citm_catalog)) = (
        read("canada"),
        read("twitter"),
        read("small"),
        read("citm_catalog"),
    ) else {
        return Ok(None);
    };

    let mut replacements = shared_replacements()?;
    replacements.extend([
        ("{{LSP_BENCH_CANADA_TABLE}}", canada.trim().to_string()),
        ("{{LSP_BENCH_TWITTER_TABLE}}", twitter.trim().to_string()),
        ("{{LSP_BENCH_SMALL_TABLE}}", small.trim().to_string()),
        (
            "{{LSP_BENCH_CITM_CATALOG_TABLE}}",
            citm_catalog.trim().to_string(),
        ),
    ]);

    render_template(LSP_BENCH_TEMPLATE, &replacements).map(Some)
}

const COVERAGE: &str = include_str!(repo_path!("benches/output/coverage.txt"));

fn shared_replacements() -> Result<Vec<(&'static str, String)>> {
    let trimmed = |text: &str| text.trim().to_string();
    Ok(vec![
        ("{{IND}}", SHARED_FRAGMENT.to_string()),
        (
            "{{CHECK_EXAMPLE}}",
            strip_front_matter(CHECK_EXAMPLE)?.to_string(),
        ),
        ("{{INSTALL_QUICK}}", INSTALL_QUICK.trim().to_string()),
        ("{{UTF8_SPEC_BADGE}}", trimmed(&badge_for("utf8"))),
        (
            "{{JSON_SCHEMA_V7_SPEC_BADGE}}",
            trimmed(&badge_for("json-schema-v7")),
        ),
        ("{{LSP_PERF_CLAIM}}", LSP_PERF_CLAIM.to_string()),
        ("{{LSP_DEMO_ALT}}", LSP_DEMO_ALT.to_string()),
        ("{{LSP_DEMO_URL}}", LSP_DEMO_URL.to_string()),
        ("{{LSP_FEATURES}}", LSP_FEATURES.trim().to_string()),
        (
            "{{HELIX_FORMAT_CONFIG}}",
            helix_format_config().trim_end().to_string(),
        ),
        ("{{BENCH_SUMMARY}}", BENCH_SUMMARY.to_string()),
        ("{{BENCH_INPUTS}}", BENCH_INPUTS.to_string()),
        ("{{BENCH_HARDWARE}}", BENCH_HARDWARE.to_string()),
        ("{{FIXTURE_CANADA}}", FIXTURE_CANADA.to_string()),
        ("{{FIXTURE_CITM_CATALOG}}", FIXTURE_CITM_CATALOG.to_string()),
        ("{{FIXTURE_TWITTER}}", FIXTURE_TWITTER.to_string()),
        ("{{FIXTURE_SMALL}}", FIXTURE_SMALL.to_string()),
        ("{{COVERAGE_BADGE}}", trimmed(&coverage_badge(COVERAGE))),
        ("{{JSONLINES_SPEC_BADGE}}", trimmed(&badge_for("jsonlines"))),
        (
            "{{BYTES2CHARS_BENCH_TABLE}}",
            trimmed(include_str!(repo_path!("benches/output/bytes2chars.md"))),
        ),
        (
            "{{JSON_DESER_BENCH_TABLE}}",
            trimmed(include_str!(repo_path!("benches/output/json_deser.md"))),
        ),
        (
            "{{JSON_PRETTIFY_BENCH_TABLE}}",
            trimmed(include_str!(repo_path!("benches/output/json_prettify.md"))),
        ),
        (
            "{{JSON_UGLIFY_BENCH_TABLE}}",
            trimmed(include_str!(repo_path!("benches/output/json_uglify.md"))),
        ),
        (
            "{{JSON_SERIALIZE_BENCH_TABLE}}",
            trimmed(include_str!(repo_path!("benches/output/json_serialize.md"))),
        ),
        (
            "{{PRETTY_CANADA_TABLE}}",
            trimmed(include_str!(repo_path!(
                "benches/docker/output/pretty-canada.md"
            ))),
        ),
        (
            "{{UGLY_CANADA_TABLE}}",
            trimmed(include_str!(repo_path!(
                "benches/docker/output/ugly-canada.md"
            ))),
        ),
        (
            "{{PRETTY_CITM_CATALOG_TABLE}}",
            trimmed(include_str!(repo_path!(
                "benches/docker/output/pretty-citm_catalog.md"
            ))),
        ),
        (
            "{{UGLY_CITM_CATALOG_TABLE}}",
            trimmed(include_str!(repo_path!(
                "benches/docker/output/ugly-citm_catalog.md"
            ))),
        ),
        (
            "{{PRETTY_TWITTER_TABLE}}",
            trimmed(include_str!(repo_path!(
                "benches/docker/output/pretty-twitter.md"
            ))),
        ),
        (
            "{{UGLY_TWITTER_TABLE}}",
            trimmed(include_str!(repo_path!(
                "benches/docker/output/ugly-twitter.md"
            ))),
        ),
        (
            "{{VSCODE_SETTINGS_REFERENCE}}",
            trimmed(&settings_reference("jjpwrgem.format.")),
        ),
        (
            "{{LSP_SETTINGS_REFERENCE}}",
            trimmed(&settings_reference("")),
        ),
    ])
}

fn render_template(template: &str, replacements: &[(&str, String)]) -> Result<String> {
    let mut processed = template.to_string();
    for (needle, replacement) in replacements {
        processed = processed.replace(needle, replacement);
    }

    oxfmt_format(&format!("{}{}", BANNER, processed), "file.md")
}

const GENERATED_READMES: &[(&str, &str)] = &[
    (repo_path!("readme.md"), template!("root.template.md")),
    (
        repo_path!("crates/parse/readme.md"),
        template!("parse.template.md"),
    ),
    (
        repo_path!("benches/BENCHMARKS.md"),
        template!("bench.template.md"),
    ),
    (
        repo_path!("benches/cli-formatter.md"),
        template!("cli-formatter-bench.template.md"),
    ),
    (
        repo_path!("benches/utf8.md"),
        template!("bytes2chars-bench.template.md"),
    ),
    (
        repo_path!("benches/json.md"),
        template!("json-bench.template.md"),
    ),
    (
        repo_path!("npm-packages/jjpwrgem-vscode/README.md"),
        template!("vscode.template.md"),
    ),
    (
        repo_path!("crates/lsp/readme.md"),
        template!("lsp_crate.template.md"),
    ),
    (
        repo_path!("crates/bytes2chars/README.md"),
        template!("bytes2chars.template.md"),
    ),
    (
        repo_path!("crates/json-schema/README.md"),
        template!("json-schema.template.md"),
    ),
];

fn generated_readmes() -> Result<Vec<(&'static str, String)>> {
    let replacements = shared_replacements()?;

    let mut readmes = Vec::new();
    for (path, template) in GENERATED_READMES {
        let mut rendered = render_template(template, &replacements)?;
        if let Ok(existing) = fs::read_to_string(path)
            && let Some(pos) = existing.find(RDME_START)
        {
            rendered = format!("{}\n\n{}", rendered.trim_end(), &existing[pos..]);
        }
        readmes.push((*path, rendered));
    }

    if let Some(lsp_bench_rendered) = generate_lsp_bench_readme()? {
        readmes.push((LSP_BENCH_OUT_PATH_STR, lsp_bench_rendered));
    }
    Ok(readmes)
}

pub fn readmes(check: bool) -> Result<()> {
    for (path, generated) in generated_readmes()? {
        write_or_check(path, &generated, "readmes", check)?;
    }
    Ok(())
}
