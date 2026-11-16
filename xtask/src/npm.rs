use std::fs;

use anyhow::{Context, Result, anyhow, bail};
use cargo_metadata::{MetadataCommand, Package};
use itertools::Itertools as _;
use jjpwrgem_parse::{ast::Document, format::LineEnding};
use serde::Deserialize;
use serde_json::{Value, json};

const BIN_NAME: &str = "jjp";

const TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../npm-packages/jjpwrgem/package.template.json"
));

#[derive(Deserialize)]
struct ReleaseTarget {
    target: String,
    archive: String,
}

fn release_targets() -> Result<Vec<ReleaseTarget>> {
    Ok(serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../.github/release-targets.json"
    )))?)
}

fn root_package() -> Result<Package> {
    MetadataCommand::new()
        .manifest_path(concat!(env!("CARGO_MANIFEST_DIR"), "/../Cargo.toml"))
        .no_deps()
        .exec()?
        .root_package()
        .cloned()
        .context("No root package found in cargo metadata")
}

pub fn write_package_json() -> Result<()> {
    let patched = patch_package_json(&root_package()?, &release_targets()?)?;
    let compact = jjpwrgem_parse::format::serde::uglify_serializable(&patched)?;
    let ast = Document::parse(compact.as_str()).map_err(|err| anyhow!(err.to_string()))?;
    let formatted = jjpwrgem_parse::format::prettify_document(&ast, 80, LineEnding::Lf);

    let out_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../npm-packages/jjpwrgem/package.json"
    );
    fs::write(out_path, formatted)?;

    println!("generated package json");
    Ok(())
}

fn supported_platform(archive: &str) -> Result<Value> {
    let (zip_ext, exe_suffix) = if archive.ends_with(".zip") {
        (".zip", ".exe")
    } else if archive.ends_with(".tar.xz") {
        (".tar.xz", "")
    } else {
        bail!("unsupported archive type: {archive}");
    };

    Ok(json!({
        "artifactName": archive,
        "bins": { BIN_NAME: format!("jjp{exe_suffix}") },
        "zipExt": zip_ext,
    }))
}

fn patch_package_json(package: &Package, targets: &[ReleaseTarget]) -> Result<Value> {
    let repository_url = package.repository.clone().context("missing repo url")?;

    let mut pkg: Value = serde_json::from_str(TEMPLATE)?;
    let map = pkg
        .as_object_mut()
        .context("template is not a JSON object")?;

    map.insert("version".into(), package.version.to_string().into());
    map.insert(
        "description".into(),
        package.description.clone().unwrap_or_default().into(),
    );
    map.insert(
        "keywords".into(),
        package
            .categories
            .iter()
            .chain(&package.keywords)
            .dedup()
            .cloned()
            .collect::<Vec<_>>()
            .into(),
    );
    map.insert(
        "homepage".into(),
        package
            .homepage
            .clone()
            .or(package.repository.clone())
            .unwrap_or_default()
            .into(),
    );
    map.insert(
        "author".into(),
        package.authors.first().cloned().unwrap_or_default().into(),
    );
    map.insert(
        "artifactDownloadUrl".into(),
        format!(
            "{repository_url}/releases/download/{}-v{}",
            package.name, package.version
        )
        .into(),
    );
    map.insert(
        "supportedPlatforms".into(),
        targets
            .iter()
            .map(|t| Ok((t.target.clone(), supported_platform(&t.archive)?)))
            .collect::<Result<serde_json::Map<_, _>>>()?
            .into(),
    );

    Ok(pkg)
}

#[cfg(test)]
mod tests {
    use cargo_metadata::{
        PackageBuilder, PackageId, PackageName, camino::Utf8PathBuf, semver::Version,
    };

    use super::*;

    fn fixture_package() -> Package {
        PackageBuilder::new(
            PackageName::new("jjpwrgem".to_owned()),
            Version::parse("0.5.1").unwrap(),
            PackageId {
                repr: "jjpwrgem 0.5.1 (path+https://github.com/20jasper/jjpwrgem)".into(),
            },
            Utf8PathBuf::from("/workspace/jjpwrgem/Cargo.toml"),
        )
        .authors(vec!["Jacob Asper <jacobasper191@gmail.com>".into()])
        .description(Some(
            "jjpwrgem json parser with really good error messages".into(),
        ))
        .license(Some("MIT".into()))
        .categories(vec![
            "command-line-utilities".into(),
            "text-processing".into(),
        ])
        .keywords(vec![
            "encoding".into(),
            "parser".into(),
            "formatter".into(),
            "json".into(),
            "linter".into(),
        ])
        .repository(Some("https://github.com/20jasper/jjpwrgem".into()))
        .homepage(Some("https://github.com/20jasper/jjpwrgem".into()))
        .build()
        .expect("package builds")
    }

    #[test]
    fn package_json_snapshot() {
        let actual = patch_package_json(&fixture_package(), &release_targets().unwrap()).unwrap();
        insta::assert_json_snapshot!(actual);
    }
}
