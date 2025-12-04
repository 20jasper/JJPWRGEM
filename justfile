dev-install:
    cargo install --locked cargo-binstall
    cargo binstall just -y
    cargo binstall cargo-watch -y
    cargo binstall cargo-llvm-cov -y
    cargo binstall cargo-insta -y
    cargo binstall cargo-shear -y

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

watch:
    cargo watch -q -c -x "install --path ."
