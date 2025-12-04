set working-directory := "."

dev-install:
    cargo binstall just -y
    cargo binstall cargo-watch -y
    cargo binstall cargo-llvm-cov -y
    cargo binstall cargo-insta -y
    cargo binstall cargo-shear -y
    cargo binstall cargo-diet -y
    cargo binstall cargo-dist -y

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
