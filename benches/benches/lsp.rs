use std::{
    collections::HashMap,
    path::PathBuf,
    process::Command,
    sync::{Mutex, OnceLock},
};

use criterion::{Criterion, criterion_group, criterion_main};
use lspresso::{Client, capabilities};
use serde::Serialize;

/// RSS-only sidecar: time comes from criterion's stored estimates.
#[derive(Default, Serialize)]
struct Sidecar {
    initialize: HashMap<String, u64>,
}

static SIDECAR: OnceLock<Mutex<Sidecar>> = OnceLock::new();

fn get_sidecar() -> &'static Mutex<Sidecar> {
    SIDECAR.get_or_init(|| Mutex::new(Sidecar::default()))
}

fn flush_sidecar() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../target/bench/lsp-memory.json");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create sidecar dir");
    }
    let json = serde_json::to_string_pretty(&*get_sidecar().lock().unwrap())
        .expect("serialize sidecar metrics");
    std::fs::write(&path, json).expect("write sidecar");
}

/// A server configuration that can produce a fresh `Command` on demand.
struct Profile {
    label: &'static str,
    bin: PathBuf,
    args: Vec<&'static str>,
}

impl Profile {
    fn command(&self) -> Command {
        let mut cmd = Command::new(&self.bin);
        cmd.args(&self.args);
        cmd
    }
}

fn jjp_profile() -> Option<Profile> {
    let bin = if let Ok(path) = std::env::var("JJP_BIN") {
        PathBuf::from(path)
    } else {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../target/release/jjp");
        if path.exists() {
            path
        } else {
            return None;
        }
    };
    Some(Profile {
        label: "jjpwrgem",
        bin,
        args: vec!["lsp"],
    })
}

fn light_json_lsp_profile() -> Profile {
    let bin = match std::env::var("LIGHT_JSON_LSP_BIN") {
        Ok(path) if !path.is_empty() => PathBuf::from(path),
        _ => PathBuf::from("light-json-lsp"),
    };
    Profile {
        label: "light-json-lsp",
        bin,
        args: vec![],
    }
}

fn vscode_json_profile() -> Profile {
    Profile {
        label: "vscode-json",
        bin: PathBuf::from("vscode-json-language-server"),
        args: vec!["--stdio"],
    }
}

/// True if the server can be spawned and successfully initializes.
fn available(profile: &Profile) -> bool {
    let Ok(mut client) = Client::spawn(profile.command()) else {
        return false;
    };
    client.initialize(capabilities::vscode()).is_ok()
}

fn profiles() -> Vec<Profile> {
    let mut profiles = Vec::new();
    if let Some(jjp) = jjp_profile() {
        profiles.push(jjp);
    }
    for profile in [vscode_json_profile(), light_json_lsp_profile()] {
        if available(&profile) {
            profiles.push(profile);
        }
    }

    match std::env::var("LSP_PROFILE") {
        Ok(filter) if !filter.is_empty() => {
            profiles.retain(|p| p.label == filter);
            profiles
        }
        _ => profiles,
    }
}

fn bench_initialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("lsp/initialize");
    for profile in &profiles() {
        group.bench_function(profile.label, |b| {
            b.iter(|| {
                let mut client = Client::spawn(profile.command()).expect("spawn");
                client
                    .initialize(capabilities::vscode())
                    .expect("initialize");
                client.shutdown().expect("shutdown");
            });
        });
        // Spawn one post-criterion client to capture peak RSS after initialization.
        let mut client = Client::spawn(profile.command()).expect("spawn");
        client
            .initialize(capabilities::vscode())
            .expect("initialize");
        if let Some(rss) = client.rss_bytes() {
            get_sidecar()
                .lock()
                .unwrap()
                .initialize
                .insert(profile.label.to_owned(), rss);
        }
    }
    group.finish();
    flush_sidecar();
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_initialize
}
criterion_main!(benches);
