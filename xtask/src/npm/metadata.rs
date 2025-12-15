use anyhow::{Context, Result};
use cargo_metadata::{MetadataCommand, Package};
#[derive(Debug, Clone)]
pub struct PackageMetadata {
    pub package: Package,
}

impl PackageMetadata {
    pub fn from_cargo_env() -> Result<Self> {
        let manifest_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../Cargo.toml");
        let package = MetadataCommand::new()
            .manifest_path(manifest_path)
            .no_deps()
            .exec()?
            .root_package()
            .cloned()
            .context("No root package found in cargo metadata")?;

        Ok(PackageMetadata { package })
    }
}

pub const AARCH64_PC_WINDOWS_MSVC: &str = "aarch64-pc-windows-msvc";
pub const X86_64_PC_WINDOWS_GNU: &str = "x86_64-pc-windows-gnu";
pub const X86_64_PC_WINDOWS_MSVC: &str = "x86_64-pc-windows-msvc";

pub const MSVC_ALIAS_FOR_X86_64: [&str; 2] = [X86_64_PC_WINDOWS_GNU, AARCH64_PC_WINDOWS_MSVC];

#[derive(Debug, Clone)]
pub struct PlatformConfig {
    pub rust_target: String,
    pub artifact_name: String,
    pub exe_suffix: String,
    pub zip_ext: String,
}

pub mod targets {
    use anyhow::Result;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct Workspace {
        dist: Dist,
    }

    #[derive(Debug, Deserialize)]
    struct Dist {
        targets: Vec<String>,
    }

    pub fn parse_dist_target_from_env() -> Result<Vec<String>> {
        parse_dist_targets(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../dist-workspace.toml"
        )))
    }
    fn parse_dist_targets(s: &str) -> Result<Vec<String>> {
        Ok(toml::from_str::<Workspace>(s)?.dist.targets)
    }
}

pub fn build_platforms_from_targets(
    meta: &PackageMetadata,
    targets: Vec<String>,
) -> Result<Vec<PlatformConfig>> {
    let mut platforms = Vec::new();

    for target in targets {
        let (zip_ext, exe_suffix) = if target.contains("windows") {
            (".zip", ".exe")
        } else {
            (".tar.xz", "")
        };

        let artifact_name = format!("{}-{}{}", meta.package.name, target, zip_ext);

        let base = PlatformConfig {
            rust_target: target.clone(),
            artifact_name: artifact_name.clone(),
            exe_suffix: exe_suffix.to_string(),
            zip_ext: zip_ext.to_string(),
        };

        if target == X86_64_PC_WINDOWS_MSVC {
            for &alias in MSVC_ALIAS_FOR_X86_64.iter() {
                let mut alias_cfg = base.clone();
                alias_cfg.rust_target = alias.to_string();
                platforms.push(alias_cfg);
            }
        }

        platforms.push(base);
    }

    Ok(platforms)
}
