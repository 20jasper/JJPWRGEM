mod metadata;

use anyhow::{Context, Ok, Result};
use itertools::Itertools as _;
use jjpwrgem_parse::format::LineEnding;
use package_json::PackageJson;
use std::{collections::HashMap, fs};

use crate::npm::metadata::{
    PackageMetadata, PlatformConfig, build_platforms_from_targets,
    targets::parse_dist_target_from_env,
};

// Key-name constants for JSON unknowns / package.json fields
const BIN_NAME: &str = "jjp";
const BIN_PATH: &str = "run-jjp.js";
const SCRIPTS_POSTINSTALL: &str = "postinstall";
const ARTIFACT_DOWNLOAD_URL_KEY: &str = "artifactDownloadUrl";
const GLIBC_MINIMUM_KEY: &str = "glibcMinimum";
const PREFER_UNPLUGGED_KEY: &str = "preferUnplugged";
const SUPPORTED_PLATFORMS_KEY: &str = "supportedPlatforms";
const ARTIFACT_NAME_KEY: &str = "artifactName";
const BINS_KEY: &str = "bins";
const ZIP_EXT_KEY: &str = "zipExt";

#[derive(Debug, Clone)]
struct GlibcMinimum {
    major: u32,
    series: u32,
}

impl Default for GlibcMinimum {
    fn default() -> Self {
        GlibcMinimum {
            major: 2,
            series: 35,
        }
    }
}

#[derive(Debug, Clone)]
struct NpmPackageConfig {
    bin_name: String,
    bin_path: String,
    dependencies: Vec<(String, String)>,
    dev_dependencies: Vec<(String, String)>,
    scripts: Vec<(String, String)>,
    engines: Vec<(String, String)>,
    glibc_minimum: GlibcMinimum,
    prefer_unplugged: bool,
}

impl Default for NpmPackageConfig {
    fn default() -> Self {
        NpmPackageConfig {
            bin_name: BIN_NAME.to_string(),
            bin_path: BIN_PATH.to_string(),
            dependencies: vec![
                ("axios".to_string(), "^1.12.2".to_string()),
                ("axios-proxy-builder".to_string(), "^0.1.2".to_string()),
                ("detect-libc".to_string(), "^2.1.2".to_string()),
                ("rimraf".to_string(), "^6.0.1".to_string()),
            ],
            dev_dependencies: vec![],
            scripts: vec![(
                SCRIPTS_POSTINSTALL.to_string(),
                "node ./install.js".to_string(),
            )],
            engines: vec![
                ("node".to_string(), ">=14".to_string()),
                ("npm".to_string(), ">=6".to_string()),
            ],
            glibc_minimum: GlibcMinimum::default(),
            prefer_unplugged: true,
        }
    }
}

pub fn write_package_json() -> Result<()> {
    let formatted_json = jjpwrgem_parse::format::serde::prettify_serializable(
        &package_json_from_env()?,
        80,
        LineEnding::Lf,
    )?;

    let static_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../npm-template/package.json");
    fs::write(static_path, formatted_json)?;

    println!("generated package json");

    Ok(())
}

fn package_json_from_env() -> Result<PackageJson> {
    let cargo_meta = PackageMetadata::from_cargo_env()?;
    let targets = parse_dist_target_from_env()?;
    let platforms = build_platforms_from_targets(&cargo_meta, targets)?;

    generate_package_json(&cargo_meta, platforms, NpmPackageConfig::default())
}

fn generate_package_json(
    cargo_meta: &PackageMetadata,
    platforms: Vec<PlatformConfig>,
    package_config: NpmPackageConfig,
) -> Result<PackageJson> {
    let repository_url = cargo_meta
        .package
        .repository
        .clone()
        .context("missing repo url")?
        .clone();

    let artifact_download_url = format!(
        "{}/releases/download/{}-v{}",
        repository_url, cargo_meta.package.name, cargo_meta.package.version
    );

    let pkg = PackageJson {
        name: cargo_meta.package.name.to_string(),
        version: cargo_meta.package.version.to_string(),
        private: false,
        description: cargo_meta.package.description.clone(),
        keywords: Some(
            cargo_meta
                .package
                .categories
                .iter()
                .cloned()
                .chain(cargo_meta.package.keywords.iter().cloned())
                .dedup()
                .collect(),
        ),
        homepage: cargo_meta
            .package
            .homepage
            .clone()
            .or(cargo_meta.package.repository.clone()),
        license: cargo_meta.package.license.clone(),
        author: cargo_meta
            .package
            .authors
            .first()
            .map(|a| package_json::PackagePeople::Literal(a.clone())),
        bin: Some(package_json::PackageBin::Record(HashMap::from([(
            package_config.bin_name.clone(),
            package_config.bin_path.clone(),
        )]))),
        dependencies: Some(package_config.dependencies.clone().into_iter().collect()),
        dev_dependencies: Some(
            package_config
                .dev_dependencies
                .clone()
                .into_iter()
                .collect(),
        ),
        scripts: package_config.scripts.clone().into_iter().collect(),
        engines: Some(package_config.engines.clone().into_iter().collect()),
        repository: Some(package_json::PackageRepository::Url(
            repository_url.to_string(),
        )),
        unknowns: {
            let mut m = HashMap::new();
            m.insert(
                ARTIFACT_DOWNLOAD_URL_KEY.to_string(),
                serde_json::Value::String(artifact_download_url),
            );
            m.insert(GLIBC_MINIMUM_KEY.to_string(), serde_json::json!({ "major": package_config.glibc_minimum.major, "series": package_config.glibc_minimum.series }));
            m.insert(
                PREFER_UNPLUGGED_KEY.to_string(),
                serde_json::Value::Bool(package_config.prefer_unplugged),
            );
            m.insert(SUPPORTED_PLATFORMS_KEY.to_string(), {
                serde_json::Value::Object(
                    platforms
                        .into_iter()
                        .map(|platform| {
                            (
                                platform.rust_target,
                                serde_json::json!({
                                    ARTIFACT_NAME_KEY: platform.artifact_name,
                                    BINS_KEY: {
                                        BIN_NAME: format!("jjp{}", platform.exe_suffix)
                                    },
                                    ZIP_EXT_KEY: platform.zip_ext,
                                }),
                            )
                        })
                        .collect(),
                )
            });
            m
        },
        ..Default::default()
    };

    Ok(pkg)
}

#[cfg(test)]
mod tests {

    use cargo_metadata::{
        PackageBuilder, PackageId, PackageName, camino::Utf8PathBuf, semver::Version,
    };
    use serde_json::json;

    use super::*;

    fn get_expected() -> serde_json::Value {
        json!(
        {
          "name": "jjpwrgem",
          "version": "0.5.1",
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
          "main": "",
          "bin": {
            "jjp": "run-jjp.js"
          },
          "repository": "https://github.com/20jasper/jjpwrgem",
          "scripts": {
            "postinstall": "node ./install.js"
          },
          "dependencies": {
            "axios-proxy-builder": "^0.1.2",
            "console.table": "^0.10.0",
            "axios": "^1.12.2",
            "rimraf": "^6.0.1",
            "detect-libc": "^2.1.2"
          },
          "devDependencies": {},
          "engines": {
            "node": ">=14",
            "npm": ">=6"
          },
          "private": false,
          "type": "",
          "preferUnplugged": true,
          "supportedPlatforms": {
            "aarch64-apple-darwin": {
              "artifactName": "jjpwrgem-aarch64-apple-darwin.tar.xz",
              "bins": {
                "jjp": "jjp"
              },
              "zipExt": ".tar.xz"
            },
            "aarch64-unknown-linux-gnu": {
              "artifactName": "jjpwrgem-aarch64-unknown-linux-gnu.tar.xz",
              "bins": {
                "jjp": "jjp"
              },
              "zipExt": ".tar.xz"
            },
            "x86_64-apple-darwin": {
              "artifactName": "jjpwrgem-x86_64-apple-darwin.tar.xz",
              "bins": {
                "jjp": "jjp"
              },
              "zipExt": ".tar.xz"
            },
            "x86_64-unknown-linux-gnu": {
              "artifactName": "jjpwrgem-x86_64-unknown-linux-gnu.tar.xz",
              "bins": {
                "jjp": "jjp"
              },
              "zipExt": ".tar.xz"
            },
            "x86_64-pc-windows-gnu": {
              "artifactName": "jjpwrgem-x86_64-pc-windows-msvc.zip",
              "bins": {
                "jjp": "jjp.exe"
              },
              "zipExt": ".zip"
            },
            "aarch64-pc-windows-msvc": {
              "artifactName": "jjpwrgem-x86_64-pc-windows-msvc.zip",
              "bins": {
                "jjp": "jjp.exe"
              },
              "zipExt": ".zip"
            },
            "x86_64-pc-windows-msvc": {
              "artifactName": "jjpwrgem-x86_64-pc-windows-msvc.zip",
              "bins": {
                "jjp": "jjp.exe"
              },
              "zipExt": ".zip"
            }
          },
          "glibcMinimum": {
            "major": 2,
            "series": 35
          },
          "artifactDownloadUrl": "https://github.com/20jasper/jjpwrgem/releases/download/jjpwrgem-v0.5.1"
        }

                )
    }

    #[test]
    fn build_manual_package_json_and_compare_with_generator() {
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

        let cargo_meta = PackageMetadata { package };

        let platforms = vec![
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
        ];

        let package_config = NpmPackageConfig {
            bin_name: BIN_NAME.to_string(),
            bin_path: BIN_PATH.to_string(),
            dependencies: vec![
                ("axios-proxy-builder".into(), "^0.1.2".into()),
                ("console.table".into(), "^0.10.0".into()),
                ("axios".into(), "^1.12.2".into()),
                ("rimraf".into(), "^6.0.1".into()),
                ("detect-libc".into(), "^2.1.2".into()),
            ],
            dev_dependencies: Vec::new(),
            scripts: vec![(SCRIPTS_POSTINSTALL.to_string(), "node ./install.js".into())],
            engines: vec![("node".into(), ">=14".into()), ("npm".into(), ">=6".into())],
            glibc_minimum: GlibcMinimum {
                major: 2,
                series: 35,
            },
            prefer_unplugged: true,
        };

        let actual = generate_package_json(&cargo_meta, platforms, package_config).unwrap();

        assert_eq!(serde_json::to_value(actual).unwrap(), get_expected());
    }
}
