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
    RUSTFLAGS=-Dwarnings cargo clippy --all-targets --all-features 

test-cov:
    cargo llvm-cov --workspace

test-cov-open:
    cargo llvm-cov --workspace --open

# generate README files from templates
generate-readmes:
    cargo run -p xtask -q -- generate-readmes

# verify that README.md files match generated templates
verify-readmes:
    cargo run -p xtask -q -- verify-readmes

watch:
    cargo watch -q -c -x "install --path ."

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
