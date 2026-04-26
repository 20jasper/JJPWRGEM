set working-directory := "."
set quiet := true

default:
    just --list

# initialize and update all submodules
[group('dev')]
submodules:
    git submodule update --init --recursive

# install required devtools via cargo binstall
[group('dev')]
tools-install:
    cargo binstall cargo-watch@8.5 -y
    cargo binstall cargo-llvm-cov@0.8 -y
    cargo binstall cargo-insta@1.46 -y
    cargo binstall cargo-shear@1.11 -y
    cargo binstall cargo-diet@1.3.0 -y
    cargo binstall cargo-dist@0.31.0 -y
    cargo binstall release-plz@0.3 -y
    cargo binstall cargo-rdme@1.5 -y
    cargo binstall tracey@1.3.0 -y
    cargo binstall cargo-release@0.25 -y
    just tools-install-bench

# install bench devtools (also requires valgrind, kcachegrind, and callgrind from system package manager)
[group('dev')]
tools-install-bench:
    cargo binstall cargo-criterion@1.1.0 -y
    cargo binstall gungraun-runner@0.18.1 -y
    cargo binstall rustfilt@0.2.1 -y

prettier := "pnpm exec oxfmt"
prettier_glob := "./**/*.{md,yaml,yml,ts,js}"

# format rust, justfile, and markdown
[group('lint')]
format:
    cargo +nightly fmt --all
    just --fmt --unstable
    {{ prettier }} {{ prettier_glob }} --write

[group('lint')]
format-check:
    cargo +nightly fmt --all -- --check
    just --fmt --unstable --check
    {{ prettier }} {{ prettier_glob }} --check

[group('lint')]
lint:
    RUSTFLAGS=-Dwarnings cargo clippy -q --all-targets --all-features --workspace
    pnpm --if-present lint > /dev/null

test_flags := "--all-features --workspace --all-targets -q"

[group('test')]
test *args="":
    cargo test {{ test_flags }} {{ args }} > /dev/null
    echo "tests passed!"

# common flag: --open
[group('test')]
test-cov *args="":
    cargo llvm-cov --exclude xtask {{ test_flags }} --json --summary-only {{ args }} \
        | jq -r '(.data[0].totals.lines.percent * 10 | round) / 10' > benches/output/coverage.txt

# deletes snapshots locally and rejects in CI
[group('test')]
test-snapshot:
    cargo insta test {{ test_flags }} --unreferenced auto
    cargo insta review

[group('readmes')]
bytes2chars-readme:
    cargo +nightly fmt -p bytes2chars
    cargo rdme --workspace-project bytes2chars --force

[group('readmes')]
bytes2chars-readme-check:
    cargo rdme --workspace-project bytes2chars --check

# generate markdown files from templates
[group('readmes')]
readmes:
    cargo xtask generate-readmes
    just bytes2chars-readme

# verify markdown files match generated templates
[group('readmes')]
readmes-check:
    cargo xtask verify-readmes
    just bytes2chars-readme-check

[group('npm')]
npm-markdown:
    cp -f readme.md npm-template/README.md
    cp -f CHANGELOG.md npm-template/CHANGELOG.md

# updates everything related to the package.json
[group('npm')]
package-json: npm-markdown
    cargo xtask generate-npm-package
    cd ./npm-template && npm i --ignore-scripts --no-fund && npm shrinkwrap && git add npm-shrinkwrap.json

# regenerated npm package metadata and checks for changes
[group('npm')]
package-json-check: package-json
    git diff --exit-code -- npm-template/npm-shrinkwrap.json
    npm pack ./npm-template --dry-run --quiet

install := "install --path ."

# install `jjp` to your path in watch mode
[group('dev')]
install-watch:
    just install
    cargo watch -q -c -x "{{ install }} --offline"

# install `jjp` to your path
[group('dev')]
install:
    cargo {{ install }}

vscode-bin-location := "./npm-packages/jjpwrgem-vscode/bin"

# build release binary and copy to the vscode extension bin dir
[group('vscode')]
vscode-bin:
    cargo build --release
    mkdir -p {{ vscode-bin-location }}
    if [ -f target/release/jjp ]; then \
        cp target/release/jjp {{ vscode-bin-location }}/jjp; \
        chmod +x {{ vscode-bin-location }}/jjp; \
    elif [ -f target/release/jjp.exe ]; then \
        cp target/release/jjp.exe {{ vscode-bin-location }}/jjp.exe; \
    else \
        echo "No built jjp binary found in target/release"; exit 1; \
    fi
    echo "Copied binary into {{ vscode-bin-location }}"

[group('vscode')]
vscode-test-wsl: vscode-bin
    # ensure XDG_RUNTIME_DIR is available and try to start a session DBus (if possible)
    export XDG_RUNTIME_DIR="/tmp/runtime-$(id -u)"; \
    mkdir -p "$XDG_RUNTIME_DIR"; chmod 700 "$XDG_RUNTIME_DIR"; \
    if command -v dbus-launch >/dev/null 2>&1; then \
        echo "Starting dbus-launch"; \
        eval "$(dbus-launch --sh-syntax)" || true; \
    elif command -v dbus-daemon >/dev/null 2>&1; then \
        dbus-daemon --session --fork --print-address > "$XDG_RUNTIME_DIR/bus" 2>/dev/null || true; \
        echo "Started dbus-daemon (if available)"; \
    else \
        echo "dbus not found; continuing without session bus"; \
    fi; \
    if command -v Xvfb >/dev/null 2>&1; then \
        /usr/bin/Xvfb :99 -screen 0 1024x768x24 > /dev/null 2>&1 & \
        sleep 1; \
        echo "Started Xvfb"; \
    else \
        echo "Xvfb not found; attempting to run tests without X server"; \
    fi; \
    export DISPLAY=":99.0"; \
    pnpm --filter jjpwrgem-vscode test

# removes unnecessary files from crates before publishing
[group('lint')]
diet:
    for x in ./crates/* ./xtask ./benches .; do \
    	[ -f "$x/Cargo.toml" ] || continue; \
    	(cd $x && cargo diet -r -q); \
    done

# verify spec rules have version bumps for any changed rule text
[group('lint')]
tracey-check:
    tracey pre-commit

[group('release')]
prepublish:
    just format-check
    just lint
    just diet
    just tracey-check

[group('release')]
publish-dry-run crate:
    cargo publish -q --dry-run -p {{ crate }}

[group('release')]
release-binary:
    release-plz update --package jjpwrgem
    cargo release --no-publish --tag-prefix=jjpwrgem- --execute

[group('release')]
release-notes:
    dist host --steps=create --output-format=json | jq -r .announcement_github_body

json-benches := "json_deser json_prettify json_uglify"

# e.g. `just bench bytes2chars` or `just bench json_prettify --sample-size 50`
[group('bench')]
bench name *args="":
    mkdir -p benches/output
    tmp="$(mktemp)"; \
    trap 'rm -f "$tmp"' EXIT; \
    cargo criterion -p benches --bench {{ name }} --message-format=json {{ args }} > "$tmp"; \
    cargo xtask bench-table < "$tmp" > benches/output/{{ name }}.md
    just readmes

[group('bench')]
bench-json-raw benches=json-benches impl="" fixture="" *args="":
    just _bench-json-raw "{{ benches }}" "{{ impl }}" "{{ fixture }}" {{ args }}

# e.g. `just bench-jjp json_prettify canada --sample-size 50`
[group('bench')]
bench-jjp benches=json-benches fixture="" *args="":
    just _bench-json-raw "{{ benches }}" "jjpwrgem" "{{ fixture }}" {{ args }}

[private]
_bench-json-raw benches impl fixture *args="":
    for bench in {{ benches }}; do \
        env_args=""; \
        if [ -n "{{ impl }}" ]; then \
            env_args="$env_args JJP_JSON_IMPL={{ impl }}"; \
        fi; \
        if [ -n "{{ fixture }}" ]; then \
            env_args="$env_args JJP_JSON_INPUT={{ fixture }}"; \
        fi; \
        env $env_args cargo criterion -p benches --bench "$bench" {{ args }}; \
    done

# run the full json benchmark suite and regenerate readmes
[group('bench')]
bench-json *args="":
    just bench json_deser {{ args }}
    just bench json_prettify {{ args }}
    just bench json_uglify {{ args }}

# runs perf tests against 10+ cli tools and regenerates outputs and embeds in readmes
[group('bench')]
bench-docker:
    mkdir -p xtask/bench/output
    GITHUB_TOKEN="$(gh auth token 2>/dev/null || true)" \
        docker build -t jjp-benchmark \
            --secret id=gh_token,env=GITHUB_TOKEN \
            .
    docker run --rm \
        -u "$(id -u):$(id -g)" \
        -v "$(pwd)/xtask/bench/output:/benchmark/output" \
        jjp-benchmark
    {{ prettier }} './xtask/bench/output/*.md' --write
    just plot-bench
    just readmes

# plots docker benches
[group('bench')]
plot-bench:
    cargo xtask plot-benchmarks

[group('bench')]
bench-iai *args="":
    cargo bench -p benches --bench json_iai {{ args }}

[group('bench')]
kcachegrind bench="deser" fixture="citm":
    #!/usr/bin/env sh
    out="target/gungraun/benches/json_iai/{{ bench }}_group/bench_{{ bench }}.{{ fixture }}/callgrind.bench_{{ bench }}.{{ fixture }}.out"
    rustfilt < "$out" > "${out%.out}.demangled.out"
    kcachegrind "${out%.out}.demangled.out" &

[group('bench')]
iai-annotate bench="deser" fixture="citm":
    callgrind_annotate \
        target/gungraun/benches/json_iai/{{ bench }}_group/bench_{{ bench }}.{{ fixture }}/callgrind.bench_{{ bench }}.{{ fixture }}.out \
        2>/dev/null | rustfilt | head -60
