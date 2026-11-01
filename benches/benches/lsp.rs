use std::{
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use criterion::{
    BatchSize, BenchmarkGroup, Criterion, SamplingMode, criterion_group, criterion_main,
    measurement::WallTime,
};
use lspresso::Client;

/// A server this bench can spawn and drive the same way: `command` builds a
/// fresh, unstarted process each time, since criterion needs a new one per
/// iteration.
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

/// vscode-json takes over 100ms to initialize; criterion's default linear
/// sampling would run thousands of iterations of that. Flat sampling keeps
/// the iteration count at the sample size.
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
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3));
    targets = bench_initialize
}
criterion_main!(benches);
