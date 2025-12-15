set working-directory := "."

dev-install:
    cargo binstall cargo-watch -y
    cargo binstall cargo-llvm-cov -y
    cargo binstall cargo-insta -y
    cargo binstall cargo-shear -y
    cargo binstall cargo-diet -y
    cargo binstall cargo-dist -y
    cargo binstall release-plz -y

# format rust, justfile, and markdown
format:
    cargo fmt --all
    just --fmt --unstable
    npx -y prettier './**/*.{md,yaml,yml}' --write

format-check:
    cargo fmt --all -- --check
    just --fmt --unstable --check
    npx -y prettier './**/*.{md,yaml,yml}' --check

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
