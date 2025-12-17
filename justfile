set working-directory := "."

dev-install:
    cargo binstall cargo-watch -y
    cargo binstall cargo-llvm-cov -y
    cargo binstall cargo-insta -y
    cargo binstall cargo-shear -y
    cargo binstall cargo-diet -y
    cargo binstall cargo-dist -y
    cargo binstall release-plz -y

prettier_glob := "./**/*.{md,yaml,yml,ts,js}"

# format rust, justfile, and markdown
format:
    cargo fmt --all
    just --fmt --unstable
    npx -y prettier {{ prettier_glob }} --write

format-check:
    cargo fmt --all -- --check
    just --fmt --unstable --check
    npx -y prettier {{ prettier_glob }} --check

lint:
    RUSTFLAGS=-Dwarnings cargo clippy --all-targets --all-features --workspace

test_flags := "--all-features --workspace --all-targets"

test:
    cargo test {{ test_flags }}

test-cov:
    cargo llvm-cov {{ test_flags }}

test-cov-open:
    cargo llvm-cov {{ test_flags }} --open

# deletes snapshots locally and rejects in CI
test-snapshot:
    cargo insta test {{ test_flags }} --unreferenced auto 
    cargo insta review

xtask-command := "cargo run -p xtask -q --"

# generate markdown files from templates
readmes:
    {{ xtask-command }} generate-readmes

# verify markdown files match generated templates
readmes-check:
    {{ xtask-command }} verify-readmes

npm-markdown:
    cp -f readme.md npm-template/README.md
    cp -f CHANGELOG.md npm-template/CHANGELOG.md

# updates everything related to the package.json
package-json: npm-markdown
    {{ xtask-command }} generate-npm-package
    cd ./npm-template && npm i --ignore-scripts && npm shrinkwrap && git add npm-shrinkwrap.json

# regenerated npm package metadata and checks for changes
package-json-check: package-json
    git diff --exit-code -- npm-template/npm-shrinkwrap.json
    npm pack ./npm-template --dry-run

# install jjp into your path (watch)
install-watch:
    cargo watch -q -c -x "install --path ."

# install jjp into your path
install:
    cargo install --path .

vscode-bin-location := "./npm-packages/jjpwrgem-vscode/bin"

# build release binary and copy to the vscode extension bin dir
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
diet:
    for x in ./crates/* .; do \
    	echo "dieting $x"; \
    	(cd $x && cargo diet -r); \
    done

prepublish:
    just format-check
    just lint
    just diet

release-binary:
    release-plz update
    cargo release --no-publish --tag-prefix=jjpwrgem- --execute

# preview release notes
release-notes:
    dist host --steps=create --output-format=json | jq -r .announcement_github_body

# runs perf tests against 10+ cli tools and regenerates outputs and embeds in readmes
bench:
    mkdir -p xtask/bench/output
    docker build -t jjp-benchmark .
    docker run --rm \
        -u "$(id -u):$(id -g)" \
        -v "$(pwd)/xtask/bench/output:/benchmark/output" \
        jjp-benchmark
    npx -y prettier './xtask/bench/output/*.md' --write
    just plot-bench
    just readmes

plot-bench:
    cargo run -p xtask -- plot-benchmarks
