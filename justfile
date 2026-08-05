run *args:
    # Run the program.
    cargo run -- {{args}}

test:
    # Run tests.
    cargo test

build:
    # Build debug package.
    cargo build

build-release:
    # Build release package.
    cargo build --release

fix: 
    # Run fix.
    cargo fix --allow-dirty --allow-staged --tests

release version:
    # Release a version.
    sed -i 's/^version = ".*"/version = "{{version}}"/' Cargo.toml
    sed -i 's/version = ".*";/version = "{{version}}";/' flake.nix
    cargo update --package injm
    git add Cargo.toml Cargo.lock flake.nix
    git commit -s -m "chore: release v{{version}}" || true
    git push origin main
    git tag -a v{{version}} -e 
    git push origin v{{version}}

toc:
  # Generate ToC in README.md
  markdown-toc -i README.md
  prettier --write README.md

lint:
    # Run lint. 
    cargo clippy
    cargo fmt --all -- --check

bench: 
    # Run benchmark.
    cargo bench
