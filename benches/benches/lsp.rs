use std::{
    collections::HashMap,
    hint::black_box,
    path::{Path, PathBuf},
    process::Command,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

use criterion::{
    BatchSize, BenchmarkGroup, BenchmarkId, Criterion, SamplingMode, criterion_group,
    criterion_main, measurement::WallTime,
};
use lspresso::Client;
use serde::Serialize;

const TIMEOUT: Duration = Duration::from_secs(5);
const FIXTURES: &[&str] = &["small", "twitter", "citm_catalog", "canada"];

/// Everything criterion doesn't measure itself: peak RSS in bytes per
/// profile (and fixture), and how many edits each server's formatting
/// response contained. `cargo xtask lsp-bench-report` joins this with
/// criterion's timing output.
#[derive(Default, Serialize)]
struct Sidecar {
    initialize: HashMap<String, u64>,
    diagnostics: HashMap<String, HashMap<String, u64>>,
    formatting: HashMap<String, HashMap<String, u64>>,
    formatting_edits: HashMap<String, HashMap<String, usize>>,
    code_action: HashMap<String, u64>,
}

static SIDECAR: OnceLock<Mutex<Sidecar>> = OnceLock::new();

fn get_sidecar() -> &'static Mutex<Sidecar> {
    SIDECAR.get_or_init(|| Mutex::new(Sidecar::default()))
}

fn flush_sidecar() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/bench/lsp-memory.json");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create target/bench");
    }
    let json =
        serde_json::to_string_pretty(&*get_sidecar().lock().unwrap()).expect("serialize sidecar");
    std::fs::write(&path, json).expect("write sidecar");
}

/// A server this bench can spawn and drive the same way: `command` builds a
/// fresh, unstarted process each time (criterion needs a new one per
/// iteration for `bench_initialize`, and a new one per profile elsewhere).
struct Profile {
    label: &'static str,
    command: fn() -> Command,
}

fn jjp_binary_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("JJP_BIN") {
        return Some(PathBuf::from(path));
    }
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/dist/jjp");
    path.exists().then_some(path)
}

fn jjp_lsp_command() -> Command {
    let bin = jjp_binary_path().expect(
        "jjp binary not found at ../target/dist/jjp -- run `cargo build --profile dist -p jjpwrgem` \
         first, or set JJP_BIN to an explicit path",
    );
    let mut command = Command::new(bin);
    command.arg("lsp");
    command
}

fn vscode_json_command() -> Command {
    let mut command = Command::new("vscode-json-language-server");
    command.arg("--stdio");
    command
}

fn light_json_lsp_command() -> Command {
    Command::new("light-json-lsp")
}

/// True if the server can be spawned and successfully initializes --
/// `vscode-json-language-server` is an optional comparison point, not a
/// hard dependency, so a bench run without it installed just skips it
/// rather than failing outright.
fn available(command: fn() -> Command) -> bool {
    let Ok(mut client) = Client::spawn(command()) else {
        return false;
    };
    client.initialize(lspresso::capabilities::vscode()).is_ok()
}

fn jjp_profile() -> Option<Profile> {
    jjp_binary_path().is_some().then_some(Profile {
        label: "jjpwrgem",
        command: jjp_lsp_command,
    })
}

fn profiles() -> Vec<Profile> {
    let mut profiles = Vec::new();
    profiles.extend(jjp_profile());
    let comparisons: [Profile; 2] = [
        Profile {
            label: "vscode-json",
            command: vscode_json_command,
        },
        Profile {
            label: "light-json-lsp",
            command: light_json_lsp_command,
        },
    ];
    profiles.extend(
        comparisons
            .into_iter()
            .filter(|profile| available(profile.command)),
    );
    profiles
}

fn warm_client(profile: &Profile, uri: &lspresso::Uri, text: impl Into<String>) -> Client {
    let mut client = Client::spawn((profile.command)()).expect("spawn");
    client
        .initialize(lspresso::capabilities::vscode())
        .expect("initialize");
    client
        .did_open(uri.clone(), text, lspresso::LanguageKind::JSON)
        .expect("did_open");
    client
}

/// vscode-json takes ~0.5s per diagnostics round trip and over a second to
/// format canada; criterion's default linear sampling would run thousands
/// of iterations of that. Flat sampling keeps the iteration count at the
/// sample size.
fn slow_group<'a>(c: &'a mut Criterion, name: &str) -> BenchmarkGroup<'a, WallTime> {
    let mut group = c.benchmark_group(name);
    group.sampling_mode(SamplingMode::Flat);
    group
}

fn bench_initialize(c: &mut Criterion) {
    let mut group = slow_group(c, "initialize");
    for profile in profiles() {
        // Startup: spawn until the initialize response. Returning the client
        // makes criterion drop (kill) it after the timer stops, so teardown
        // isn't counted.
        group.bench_function(profile.label, |b| {
            b.iter_batched(
                || (),
                |()| {
                    let mut client = Client::spawn((profile.command)()).unwrap();
                    client.initialize(lspresso::capabilities::vscode()).unwrap();
                    client
                },
                BatchSize::PerIteration,
            );
        });

        // Spawn one post-criterion client to capture peak RSS after init.
        let mut client = Client::spawn((profile.command)()).unwrap();
        client.initialize(lspresso::capabilities::vscode()).unwrap();
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

fn data_path(name: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "data", &format!("{name}.json")]
        .iter()
        .collect()
}

fn fixture_uri(name: &str) -> lspresso::Uri {
    lspresso::parse_uri(&format!("file://{}", data_path(name).display()))
}

/// A copy of `name`'s fixture with a trailing syntax error, to exercise
/// diagnostics reporting.
fn invalid_fixture(name: &str) -> String {
    let valid = std::fs::read_to_string(data_path(name)).expect("read fixture");
    format!("{valid}\nnot json")
}

/// Time-to-diagnostics: edit -> diagnostics available, the editor hot path.
/// Both profiles are driven push-only (neither client declares pull
/// diagnostic support), so this also captures `vscode-json-language-server`'s
/// real debounce delay rather than hiding it behind a pull request.
fn bench_diagnostics(c: &mut Criterion) {
    let mut group = slow_group(c, "diagnostics");
    for profile in profiles() {
        for name in FIXTURES {
            let text = invalid_fixture(name);
            let uri = fixture_uri(name);
            let mut client = warm_client(&profile, &uri, text.clone());
            // Settle diagnostics for the initial didOpen before timing.
            client.next_diagnostics(&uri, TIMEOUT).unwrap();

            group.bench_with_input(BenchmarkId::new(profile.label, name), &uri, |b, uri| {
                b.iter(|| {
                    client.did_change_full(uri, text.clone()).unwrap();
                    client.next_diagnostics(black_box(uri), TIMEOUT).unwrap();
                });
            });

            // Peak RSS across all criterion iterations for this profile+fixture.
            if let Some(rss) = client.rss_bytes() {
                get_sidecar()
                    .lock()
                    .unwrap()
                    .diagnostics
                    .entry(profile.label.to_owned())
                    .or_default()
                    .insert((*name).to_owned(), rss);
            }
        }
    }
    group.finish();
    flush_sidecar();
}

fn bench_formatting(c: &mut Criterion) {
    let mut group = slow_group(c, "formatting");
    for profile in profiles() {
        for name in FIXTURES {
            let text = std::fs::read_to_string(data_path(name)).expect("read fixture");
            let uri = fixture_uri(name);
            let mut client = warm_client(&profile, &uri, text);
            client.next_diagnostics(&uri, TIMEOUT).unwrap();
            let edits = client
                .formatting(&uri)
                .unwrap()
                .map_or(0, |edits| edits.len());
            get_sidecar()
                .lock()
                .unwrap()
                .formatting_edits
                .entry(profile.label.to_owned())
                .or_default()
                .insert((*name).to_owned(), edits);

            group.bench_with_input(BenchmarkId::new(profile.label, name), &uri, |b, uri| {
                b.iter(|| client.formatting(black_box(uri)).unwrap());
            });

            if let Some(rss) = client.rss_bytes() {
                get_sidecar()
                    .lock()
                    .unwrap()
                    .formatting
                    .entry(profile.label.to_owned())
                    .or_default()
                    .insert((*name).to_owned(), rss);
            }
        }
    }
    group.finish();
    flush_sidecar();
}

/// `jjp`-only: open -> first diagnostics for a document the server hasn't
/// seen, then close it untimed. Each iteration opens a fresh URI because
/// jjp publishes without a version: the clear it sends on close would
/// otherwise satisfy the next open's wait.
fn bench_open(c: &mut Criterion) {
    let Some(profile) = jjp_profile() else {
        return;
    };
    let mut group = slow_group(c, "open");
    let mut client = Client::spawn((profile.command)()).expect("spawn");
    client
        .initialize(lspresso::capabilities::vscode())
        .expect("initialize");
    let mut opened = 0u64;
    for name in FIXTURES {
        let text = std::fs::read_to_string(data_path(name)).expect("read fixture");
        group.bench_function(*name, |b| {
            b.iter_custom(|iters| {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iters {
                    opened += 1;
                    let uri =
                        lspresso::parse_uri(&format!("file:///lspresso-bench-open-{opened}.json"));
                    let text = text.clone();
                    let start = Instant::now();
                    client
                        .did_open(uri.clone(), text, lspresso::LanguageKind::JSON)
                        .unwrap();
                    client.next_diagnostics(black_box(&uri), TIMEOUT).unwrap();
                    elapsed += start.elapsed();
                    client.did_close(&uri).unwrap();
                }
                elapsed
            });
        });
    }
    group.finish();
}

/// A document with a missing colon, used to exercise code actions.
const INVALID_JSON: &str = "{\n  \"key\" 1\n}\n";

/// `jjp`-only: `vscode-json-language-server` has no diagnostic-driven code
/// action for JSON syntax errors at all -- its only code action is an
/// unconditional "Sort JSON" command, not a fix for anything, so there's
/// no equivalent operation on its side to compare against.
fn bench_code_action(c: &mut Criterion) {
    let Some(profile) = jjp_profile() else {
        return;
    };
    let uri = lspresso::parse_uri("file:///lspresso-bench-code-action.json");
    let range = lspresso::Range::new(lspresso::Position::new(0, 0), lspresso::Position::new(2, 1));

    let mut client = warm_client(&profile, &uri, INVALID_JSON);
    let diagnostics = client.next_diagnostics(&uri, TIMEOUT).unwrap();
    client
        .code_action(&uri, range, diagnostics.clone())
        .unwrap();

    c.bench_function("code_action", |b| {
        b.iter(|| {
            client
                .code_action(black_box(&uri), range, diagnostics.clone())
                .unwrap()
        });
    });

    if let Some(rss) = client.rss_bytes() {
        get_sidecar()
            .lock()
            .unwrap()
            .code_action
            .insert(profile.label.to_owned(), rss);
    }
    flush_sidecar();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3));
    targets = bench_initialize, bench_diagnostics, bench_formatting, bench_open,
        bench_code_action
}
criterion_main!(benches);
