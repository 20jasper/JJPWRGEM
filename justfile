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
    cargo llvm-cov

watch:
    cargo watch -q -c -x "install --path ."
