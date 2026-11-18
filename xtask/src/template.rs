use std::{fs, io, path::Path};

use anyhow::{Context as _, Result, anyhow};
use minijinja::{AutoEscape, Environment, UndefinedBehavior, Value, context, syntax::SyntaxConfig};

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
const COVERAGE: &str = include_str!(repo_path!("benches/output/coverage.txt"));

const BANNER: &str = "<!-- GENERATED FILE - update the templates in the xtask -->\n\n";

const TEMPLATES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/templates");
const REPO_ROOT: &str = repo_path!("");
const LSP_BENCH_TEMPLATE: &str = "lsp-bench.template.md";
const LSP_BENCH_OUT_PATH_STR: &str = repo_path!("benches/lsp/lsp.md");
const LSP_BENCH_RESULTS: &[&str] = &["canada", "twitter", "small", "citm_catalog"];

fn load_source(name: &str) -> Result<Option<String>, minijinja::Error> {
    for root in [TEMPLATES_DIR, REPO_ROOT] {
        match fs::read_to_string(Path::new(root).join(name)) {
            Ok(source) => return Ok(Some(source)),
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            Err(err) => {
                return Err(minijinja::Error::new(
                    minijinja::ErrorKind::InvalidOperation,
                    format!("could not read {name}"),
                )
                .with_source(err));
            }
        }
    }
    Ok(None)
}

fn environment() -> Environment<'static> {
    let mut env = Environment::new();
    env.set_undefined_behavior(UndefinedBehavior::Strict);
    env.set_auto_escape_callback(|_| AutoEscape::None);
    env.set_syntax(
        SyntaxConfig::builder()
            .keep_trailing_newline(true)
            .build()
            .expect("default delimiters are valid"),
    );
    env.set_loader(load_source);
    env
}

fn trimmed(text: &str) -> String {
    text.trim().to_string()
}

fn context() -> Result<Value> {
    Ok(context! {
        check_example => strip_front_matter(CHECK_EXAMPLE)?,
        coverage_badge => trimmed(&coverage_badge(COVERAGE)),
        utf8_spec_badge => trimmed(&badge_for("utf8")),
        json_schema_v7_spec_badge => trimmed(&badge_for("json-schema-v7")),
        jsonlines_spec_badge => trimmed(&badge_for("jsonlines")),
        helix_format_config => helix_format_config().trim_end(),
        vscode_settings_reference => trimmed(&settings_reference("jjpwrgem.format.")),
        lsp_settings_reference => trimmed(&settings_reference("")),
    })
}

fn render_template(env: &Environment, name: &str, ctx: &Value) -> Result<String> {
    let rendered = env
        .get_template(name)
        .and_then(|template| template.render(ctx))
        .map_err(|err| anyhow!("{err:#}"))?;
    oxfmt_format(&format!("{BANNER}{rendered}"), "file.md")
}

fn generate_lsp_bench_readme(env: &Environment, ctx: &Value) -> Result<Option<String>> {
    let all_present = LSP_BENCH_RESULTS.iter().all(|name| {
        Path::new(REPO_ROOT)
            .join(format!("benches/lsp/results/{name}.md"))
            .is_file()
    });
    if !all_present {
        return Ok(None);
    }
    render_template(env, LSP_BENCH_TEMPLATE, ctx).map(Some)
}

const GENERATED_READMES: &[(&str, &str)] = &[
    (repo_path!("readme.md"), "root.template.md"),
    (repo_path!("crates/parse/readme.md"), "parse.template.md"),
    (repo_path!("benches/BENCHMARKS.md"), "bench.template.md"),
    (
        repo_path!("benches/cli-formatter.md"),
        "cli-formatter-bench.template.md",
    ),
    (
        repo_path!("benches/utf8.md"),
        "bytes2chars-bench.template.md",
    ),
    (repo_path!("benches/json.md"), "json-bench.template.md"),
    (
        repo_path!("npm-packages/jjpwrgem-vscode/README.md"),
        "vscode.template.md",
    ),
    (repo_path!("crates/lsp/readme.md"), "lsp_crate.template.md"),
    (
        repo_path!("crates/bytes2chars/README.md"),
        "bytes2chars.template.md",
    ),
    (
        repo_path!("crates/json-schema/README.md"),
        "json-schema.template.md",
    ),
];

fn generated_readmes() -> Result<Vec<(&'static str, String)>> {
    let env = environment();
    let ctx = context()?;

    let mut readmes = Vec::new();
    for (path, template) in GENERATED_READMES {
        let mut rendered = render_template(&env, template, &ctx)?;
        if let Ok(existing) = fs::read_to_string(path)
            && let Some(pos) = existing.find(RDME_START)
        {
            rendered = format!("{}\n\n{}", rendered.trim_end(), &existing[pos..]);
        }
        readmes.push((*path, rendered));
    }

    if let Some(lsp_bench_rendered) = generate_lsp_bench_readme(&env, &ctx)? {
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
