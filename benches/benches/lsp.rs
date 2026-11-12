use std::{
    collections::BTreeMap,
    hint::black_box,
    path::Path,
    process::Command,
    sync::{LazyLock, Mutex},
    time::Duration,
};

use criterion::{
    BatchSize, Bencher, BenchmarkGroup, BenchmarkId, Criterion, SamplingMode, criterion_group,
    criterion_main, measurement::WallTime,
};
use lspresso::{Client, capabilities, gen_lsp_types::*, server_command};

const TIMEOUT: Duration = Duration::from_secs(5);
const FIXTURES: [&str; 4] = ["small", "twitter", "citm_catalog", "canada"];
const JJP: &str = "jjpwrgem";

/// A server to benchmark. `command` builds a fresh process each time, since
/// startup is timed once per process.
struct Profile {
    label: &'static str,
    command: fn() -> Command,
}

const SERVERS: [Profile; 3] = [
    Profile {
        label: JJP,
        command: || {
            server_command(
                "JJP_BIN",
                concat!(env!("CARGO_MANIFEST_DIR"), "/../target/dist/jjp"),
                ["lsp"],
            )
        },
    },
    Profile {
        label: "vscode-json",
        command: || {
            server_command(
                "VSCODE_JSON_BIN",
                "vscode-json-language-server",
                ["--stdio"],
            )
        },
    },
    Profile {
        label: "light-json-lsp",
        command: || Command::new("light-json-lsp"),
    },
];

impl Profile {
    fn spawn(&self) -> Client {
        Client::spawn((self.command)())
            .unwrap()
            .with_timeout(TIMEOUT)
    }

    fn initialized(&self) -> Client {
        let mut client = self.spawn();
        client.initialize(capabilities::vscode::latest()).unwrap();
        client
    }

    fn warm(&self, uri: &Uri, text: impl Into<String>) -> (Client, Vec<Diagnostic>) {
        let mut client = self.initialized();
        client
            .did_open(uri.clone(), text, LanguageKind::JSON)
            .unwrap();
        let diagnostics = client.diagnostics(uri).unwrap();
        (client, diagnostics)
    }
}

/// Servers that can't start are skipped, so a run without the comparison
/// servers installed still benchmarks jjp
static PROFILES: LazyLock<Vec<&Profile>> = LazyLock::new(|| {
    SERVERS
        .iter()
        .filter(|server| {
            Client::spawn_initialized((server.command)(), capabilities::vscode::latest()).is_ok()
        })
        .collect()
});

fn jjp() -> Option<&'static Profile> {
    PROFILES
        .iter()
        .copied()
        .find(|profile| profile.label == JJP)
}

/// Peak RSS in bytes, keyed like the criterion benchmark id
/// (`diagnostics/jjpwrgem/small`). `cargo xtask lsp-bench-report` joins this with
/// criterion's timings
static MEMORY: Mutex<BTreeMap<String, u64>> = Mutex::new(BTreeMap::new());

fn record_memory(id: &str, client: &mut Client) {
    let Some(rss) = client.rss_bytes() else {
        return;
    };
    let mut memory = MEMORY.lock().unwrap();
    memory.insert(id.to_owned(), rss);
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/bench/lsp-memory.json");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, serde_json::to_string_pretty(&*memory).unwrap()).unwrap();
}

fn fixture(name: &str) -> (Uri, String) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("data/{name}.json"));
    let uri = format!("file://{}", path.display()).parse().unwrap();
    (uri, std::fs::read_to_string(path).unwrap())
}

fn end_position(text: &str) -> Position {
    let line = text.bytes().filter(|&byte| byte == b'\n').count();
    let character: usize = text
        .rsplit_once('\n')
        .map_or(text, |(_, after)| after)
        .chars()
        .map(char::len_utf16)
        .sum();
    Position {
        line: line.try_into().expect("fixture line count fits u32"),
        character: character.try_into().expect("fixture line length fits u32"),
    }
}

/// vscode-json takes over 100ms to initialize and can take over a second to
/// format canada; criterion's default linear sampling would run thousands of
/// iterations of that
fn slow_group<'a>(c: &'a mut Criterion, name: &str) -> BenchmarkGroup<'a, WallTime> {
    let mut group = c.benchmark_group(name);
    group.sampling_mode(SamplingMode::Flat);
    group
}

fn iter_server_time<T>(
    b: &mut Bencher,
    client: &mut Client,
    mut round_trip: impl FnMut(&mut Client) -> T,
) {
    b.iter_custom(|iters| {
        let before = client.timings();
        for _ in 0..iters {
            black_box(round_trip(client));
        }
        client.timings().since(before).server
    });
}

fn bench_fixtures(
    c: &mut Criterion,
    name: &str,
    mut bench: impl FnMut(&mut BenchmarkGroup<WallTime>, &Profile, &str),
) {
    let mut group = slow_group(c, name);
    for profile in PROFILES.iter() {
        for fixture in FIXTURES {
            bench(&mut group, profile, fixture);
        }
    }
}

/// Spawn until the initialize response. The client is dropped after the timer
/// stops, so teardown isn't counted
fn bench_initialize(c: &mut Criterion) {
    let mut group = slow_group(c, "initialize");
    for profile in PROFILES.iter() {
        group.bench_function(profile.label, |b| {
            b.iter_batched(|| (), |()| profile.initialized(), BatchSize::PerIteration);
        });
        record_memory(
            &format!("initialize/{}", profile.label),
            &mut profile.initialized(),
        );
    }
}

/// Edit, then wait for diagnostics: the editor hot path. The document alternates
/// between two texts so every edit is a real change a server can't serve from cache
fn bench_diagnostics(c: &mut Criterion) {
    bench_fixtures(c, "diagnostics", |group, profile, name| {
        let (uri, valid) = fixture(name);
        let broken = format!("{valid}\nnot json");
        let edited = format!("{broken}\n");
        let end = end_position(&broken);
        let (mut client, _) = profile.warm(&uri, broken.clone());
        let incremental = client.supports_incremental_sync();
        let mut insert = false;
        group.bench_function(BenchmarkId::new(profile.label, name), |b| {
            iter_server_time(b, &mut client, |client| {
                insert = !insert;
                if incremental {
                    let range = if insert {
                        Range { start: end, end }
                    } else {
                        Range {
                            start: end,
                            end: Position {
                                line: end.line,
                                character: end.character + 1,
                            },
                        }
                    };
                    client
                        .did_change_incremental(&uri, range, if insert { "\n" } else { "" })
                        .unwrap();
                } else {
                    let text = if insert { &edited } else { &broken };
                    client.did_change_full(&uri, text.clone()).unwrap();
                }
                client.diagnostics(black_box(&uri)).unwrap();
            });
        });
        record_memory(
            &format!("diagnostics/{}/{name}", profile.label),
            &mut client,
        );
    });
}

fn bench_formatting(c: &mut Criterion) {
    bench_fixtures(c, "formatting", |group, profile, name| {
        let (uri, text) = fixture(name);
        let (mut client, _) = profile.warm(&uri, text);
        group.bench_function(BenchmarkId::new(profile.label, name), |b| {
            iter_server_time(b, &mut client, |client| {
                client
                    .formatting(black_box(&uri), FormattingOptions::default())
                    .unwrap()
            });
        });
        record_memory(&format!("formatting/{}/{name}", profile.label), &mut client);
    });
}

/// jjp only: open, then wait for the first diagnostics of a document the server
/// hasn't seen. Each iteration opens a fresh URI and closes it untimed
fn bench_open(c: &mut Criterion) {
    let Some(profile) = jjp() else {
        return;
    };
    let mut group = slow_group(c, "open");
    let mut client = profile.initialized();
    let mut opened = 0;
    for name in FIXTURES {
        let (_, text) = fixture(name);
        group.bench_function(name, |b| {
            b.iter_custom(|iters| {
                (0..iters)
                    .map(|_| {
                        opened += 1;
                        let uri: Uri = format!("file:///open-{opened}.json").parse().unwrap();
                        let before = client.timings();
                        client
                            .did_open(uri.clone(), text.clone(), LanguageKind::JSON)
                            .unwrap();
                        client.diagnostics(black_box(&uri)).unwrap();
                        let waited = client.timings().since(before).server;
                        client.did_close(&uri).unwrap();
                        waited
                    })
                    .sum()
            });
        });
    }
}

/// jjp only: vscode-json has no code action that fixes a JSON syntax error, so
/// there is nothing to compare against
fn bench_code_action(c: &mut Criterion) {
    let Some(profile) = jjp() else {
        return;
    };
    let uri = "file:///code-action.json".parse().unwrap();
    let range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 2,
            character: 1,
        },
    };
    // A missing colon
    let (mut client, diagnostics) = profile.warm(&uri, "{\n  \"key\" 1\n}\n");

    c.bench_function("code_action", |b| {
        iter_server_time(b, &mut client, |client| {
            client
                .code_action(black_box(&uri), range, diagnostics.clone())
                .unwrap()
        });
    });
    record_memory(&format!("code_action/{}", profile.label), &mut client);
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3));
    targets = bench_initialize, bench_diagnostics, bench_formatting, bench_open, bench_code_action
}
criterion_main!(benches);
