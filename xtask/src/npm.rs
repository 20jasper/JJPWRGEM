mod metadata;

use std::fs;

use anyhow::{Context, Result, anyhow};
use itertools::Itertools as _;
use jjpwrgem_parse::{ast::Document, format::LineEnding};
use serde_json::Value;

use crate::npm::metadata::{
    PackageMetadata, PlatformConfig, build_platforms_from_targets,
    targets::parse_dist_target_from_env,
};

const BIN_NAME: &str = "jjp";
const ARTIFACT_DOWNLOAD_URL_KEY: &str = "artifactDownloadUrl";
const SUPPORTED_PLATFORMS_KEY: &str = "supportedPlatforms";
const ARTIFACT_NAME_KEY: &str = "artifactName";
const BINS_KEY: &str = "bins";
const ZIP_EXT_KEY: &str = "zipExt";

const TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../npm-template/package.template.json"
));

pub fn write_package_json() -> Result<()> {
    let patched = patch_package_json()?;
    let compact = jjpwrgem_parse::format::serde::uglify_serializable(&patched)?;
    let ast = Document::parse(compact.as_str()).map_err(|err| anyhow!(err.to_string()))?;
    let formatted = jjpwrgem_parse::format::prettify_document(&ast, 80, LineEnding::Lf);

    let out_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../npm-template/package.json");
    fs::write(out_path, formatted)?;

    println!("generated package json");
    Ok(())
}

fn patch_package_json() -> Result<Value> {
    let cargo_meta = PackageMetadata::from_cargo_env()?;
    let targets = parse_dist_target_from_env()?;
    let platforms = build_platforms_from_targets(&cargo_meta, targets)?;
    apply_patches(serde_json::from_str(TEMPLATE)?, &cargo_meta, platforms)
}

fn apply_patches(
    mut pkg: Value,
    cargo_meta: &PackageMetadata,
    platforms: Vec<PlatformConfig>,
) -> Result<Value> {
    let repository_url = cargo_meta
        .package
        .repository
        .clone()
        .context("missing repo url")?;

    let artifact_download_url = format!(
        "{}/releases/download/{}-v{}",
        repository_url, cargo_meta.package.name, cargo_meta.package.version
    );

    let map = pkg
        .as_object_mut()
        .context("template is not a JSON object")?;

    map.insert(
        "version".into(),
        Value::String(cargo_meta.package.version.to_string()),
    );
    map.insert(
        "description".into(),
        Value::String(cargo_meta.package.description.clone().unwrap_or_default()),
    );
    map.insert(
        "keywords".into(),
        Value::Array(
            cargo_meta
                .package
                .categories
                .iter()
                .cloned()
                .chain(cargo_meta.package.keywords.iter().cloned())
                .dedup()
                .map(Value::String)
                .collect(),
        ),
    );
    map.insert(
        "homepage".into(),
        Value::String(
            cargo_meta
                .package
                .homepage
                .clone()
                .or(cargo_meta.package.repository.clone())
                .unwrap_or_default(),
        ),
    );
    map.insert(
        "author".into(),
        Value::String(
            cargo_meta
                .package
                .authors
                .first()
                .cloned()
                .unwrap_or_default(),
        ),
    );
    map.insert(
        ARTIFACT_DOWNLOAD_URL_KEY.into(),
        Value::String(artifact_download_url),
    );
    map.insert(
        SUPPORTED_PLATFORMS_KEY.into(),
        Value::Object(
            platforms
                .into_iter()
                .map(|platform| {
                    (
                        platform.rust_target,
                        serde_json::json!({
                            ARTIFACT_NAME_KEY: platform.artifact_name,
                            BINS_KEY: { BIN_NAME: format!("jjp{}", platform.exe_suffix) },
                            ZIP_EXT_KEY: platform.zip_ext,
                        }),
                    )
                })
                .collect(),
        ),
    );

    Ok(pkg)
}

#[cfg(test)]
mod tests {
    use cargo_metadata::{
        PackageBuilder, PackageId, PackageName, camino::Utf8PathBuf, semver::Version,
    };
    use serde_json::json;

    use super::*;

    fn make_cargo_meta() -> PackageMetadata {
        let package = PackageBuilder::new(
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
        .expect("package builds");

        PackageMetadata { package }
    }

    fn make_platforms() -> Vec<PlatformConfig> {
        vec![
            PlatformConfig {
                rust_target: "aarch64-apple-darwin".into(),
                artifact_name: "jjpwrgem-aarch64-apple-darwin.tar.xz".into(),
                exe_suffix: String::new(),
                zip_ext: ".tar.xz".into(),
            },
            PlatformConfig {
                rust_target: "aarch64-unknown-linux-gnu".into(),
                artifact_name: "jjpwrgem-aarch64-unknown-linux-gnu.tar.xz".into(),
                exe_suffix: String::new(),
                zip_ext: ".tar.xz".into(),
            },
            PlatformConfig {
                rust_target: "x86_64-apple-darwin".into(),
                artifact_name: "jjpwrgem-x86_64-apple-darwin.tar.xz".into(),
                exe_suffix: String::new(),
                zip_ext: ".tar.xz".into(),
            },
            PlatformConfig {
                rust_target: "x86_64-unknown-linux-gnu".into(),
                artifact_name: "jjpwrgem-x86_64-unknown-linux-gnu.tar.xz".into(),
                exe_suffix: String::new(),
                zip_ext: ".tar.xz".into(),
            },
            PlatformConfig {
                rust_target: "x86_64-pc-windows-gnu".into(),
                artifact_name: "jjpwrgem-x86_64-pc-windows-msvc.zip".into(),
                exe_suffix: ".exe".into(),
                zip_ext: ".zip".into(),
            },
            PlatformConfig {
                rust_target: "aarch64-pc-windows-msvc".into(),
                artifact_name: "jjpwrgem-x86_64-pc-windows-msvc.zip".into(),
                exe_suffix: ".exe".into(),
                zip_ext: ".zip".into(),
            },
            PlatformConfig {
                rust_target: "x86_64-pc-windows-msvc".into(),
                artifact_name: "jjpwrgem-x86_64-pc-windows-msvc.zip".into(),
                exe_suffix: ".exe".into(),
                zip_ext: ".zip".into(),
            },
        ]
    }

    #[test]
    fn patch_matches_expected() {
        let cargo_meta = make_cargo_meta();
        let platforms = make_platforms();
        let template: Value = serde_json::from_str(TEMPLATE).unwrap();
        let actual = apply_patches(template, &cargo_meta, platforms).unwrap();

        let expected = json!({
          "name": "jjpwrgem",
          "version": "0.5.1",
          "private": false,
          "description": "jjpwrgem json parser with really good error messages",
          "keywords": [
            "command-line-utilities",
            "text-processing",
            "encoding",
            "parser",
            "formatter",
            "json",
            "linter"
          ],
          "homepage": "https://github.com/20jasper/jjpwrgem",
          "license": "MIT",
          "author": "Jacob Asper <jacobasper191@gmail.com>",
          "repository": "https://github.com/20jasper/jjpwrgem",
          "bin": { "jjp": "run-jjp.js" },
          "files": ["*.js", "README.md", "CHANGELOG.md", "LICENSE"],
          "type": "",
          "main": "",
          "scripts": { "postinstall": "node ./install.js" },
          "dependencies": { "detect-libc": "^2.1.2" },
          "devDependencies": {},
          "engines": { "node": ">=14", "npm": ">=6" },
          "artifactDownloadUrl": "https://github.com/20jasper/jjpwrgem/releases/download/jjpwrgem-v0.5.1",
          "glibcMinimum": { "major": 2, "series": 35 },
          "preferUnplugged": true,
          "supportedPlatforms": {
            "aarch64-apple-darwin": {
              "artifactName": "jjpwrgem-aarch64-apple-darwin.tar.xz",
              "bins": { "jjp": "jjp" },
              "zipExt": ".tar.xz"
            },
            "aarch64-unknown-linux-gnu": {
              "artifactName": "jjpwrgem-aarch64-unknown-linux-gnu.tar.xz",
              "bins": { "jjp": "jjp" },
              "zipExt": ".tar.xz"
            },
            "x86_64-apple-darwin": {
              "artifactName": "jjpwrgem-x86_64-apple-darwin.tar.xz",
              "bins": { "jjp": "jjp" },
              "zipExt": ".tar.xz"
            },
            "x86_64-unknown-linux-gnu": {
              "artifactName": "jjpwrgem-x86_64-unknown-linux-gnu.tar.xz",
              "bins": { "jjp": "jjp" },
              "zipExt": ".tar.xz"
            },
            "x86_64-pc-windows-gnu": {
              "artifactName": "jjpwrgem-x86_64-pc-windows-msvc.zip",
              "bins": { "jjp": "jjp.exe" },
              "zipExt": ".zip"
            },
            "aarch64-pc-windows-msvc": {
              "artifactName": "jjpwrgem-x86_64-pc-windows-msvc.zip",
              "bins": { "jjp": "jjp.exe" },
              "zipExt": ".zip"
            },
            "x86_64-pc-windows-msvc": {
              "artifactName": "jjpwrgem-x86_64-pc-windows-msvc.zip",
              "bins": { "jjp": "jjp.exe" },
              "zipExt": ".zip"
            }
          }
        });

        assert_eq!(actual, expected);
    }
}
