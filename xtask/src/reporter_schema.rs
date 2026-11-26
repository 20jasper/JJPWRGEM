use anyhow::Result;

use crate::generated::{pretty_json, write_or_check};

const SCHEMA_PATH: &str = repo_path!("schemas/reporter.schema.json");

pub fn reporter_schema(check: bool) -> Result<()> {
    let generated = pretty_json(&jjpwrgem_ui::reporter_schema())?;
    write_or_check(SCHEMA_PATH, &generated, "reporter-schema", check)
}
