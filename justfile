dev-install:
    cargo install --locked cargo-binstall
    cargo binstall just -y
    cargo binstall cargo-watch -y
    cargo binstall cargo-llvm-cov -y
    cargo binstall cargo-insta -y

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
    cargo llvm-cov --lcov --output-path ./target/llvm-cov/lcov.info

watch:
    cargo watch -q -c -x "install --path ."
