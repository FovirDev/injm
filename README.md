# injm

A CLI tool that injects content into marked regions in source files.

## Table of Contents

<!-- toc -->

- [Installation](#installation)
  - [Cargo](#cargo)
  - [Nix](#nix)
    - [Dev shell](#dev-shell)
    - [Global (NixOS)](#global-nixos)
  - [Download Binary](#download-binary)
  - [GitHub Action](#github-action)
- [Usage](#usage)
  - [Quick Start](#quick-start)
  - [Basic Injection](#basic-injection)
  - [Inject into a Specific Region](#inject-into-a-specific-region)
  - [Sync Between Files](#sync-between-files)
  - [Marker Region Configuration](#marker-region-configuration)
  - [Multiple Files and Globs](#multiple-files-and-globs)
  - [Excluding Files](#excluding-files)
  - [`.gitignore` Integration](#gitignore-integration)
  - [Project Configuration](#project-configuration)
  - [List Markers](#list-markers)
  - [Dry Run](#dry-run)
  - [Check](#check)
- [Supported Languages](#supported-languages)
- [Contributing](#contributing)
- [Roadmap](#roadmap)
- [License](#license)
- [Acknowledgement](#acknowledgement)

<!-- tocstop -->

## Installation

### Cargo

```bash
cargo install injm
```

### Nix

Add `injm` as a flake input and install it declaratively in a dev shell or globally on NixOS:

```nix
# flake.nix of your project
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    injm.url = "github:FovirDev/injm";
  };
  # ...
}
```

#### Dev shell

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    injm.url = "github:FovirDev/injm";
  };

  outputs = { self, nixpkgs, injm }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      devShells.${system}.default = pkgs.mkShell {
        packages = [ injm.packages.${system}.default ];
      };
    };
}
```

Enter the shell with `nix develop`, or with `direnv`:

```bash
echo "use flake" >> .envrc && direnv allow
```

#### Global (NixOS)

With `injm` in your flake inputs (see above), add it to `environment.systemPackages` in your `configuration.nix`:

```nix
{ inputs, pkgs, ... }: {
  environment.systemPackages = [ inputs.injm.packages.${pkgs.system}.default ];
}
```

Then apply with `sudo nixos-rebuild switch`.

Alternatively, install imperatively with:

```bash
nix profile install github:FovirDev/injm
```

> Note: `injm` is not in the `nixpkgs` binary cache, so the first build compiles from source (needs network to fetch the tree-sitter parser bundle).

### Download Binary

Download the latest binary for your platform from [GitHub Releases](https://github.com/FovirDev/injm/releases/latest).

### GitHub Action

Use the [setup injm action](https://github.com/FovirDev/injm) to install `injm` in your GitHub Actions workflow:

```yaml
steps:
  - uses: FovirDev/injm@v1
    with:
      version: latest # optional, defaults to `latest`; use a specific tag like `v1.0.0` to pin
```

The action downloads the injm binary for the current platform and adds it to `PATH`, so you can use it in subsequent steps:

```yaml
steps:
  - uses: FovirDev/injm@v1

  - name: Verify marker regions are in sync
    run: injm check
```

It supports `ubuntu` (x64/arm64), `macOS` (x64/arm64) and `windows` (x64) runners.

## Usage

### Quick Start

The simplest way is to create an `injm.toml` in your project root and run `injm` without any subcommand:

```toml
input = ["src/**/*.rs"]
output = ["docs/"]
exclude = ["target/**"]
```

```bash
injm
```

This reads content from `src/**/*.rs`, finds all `<id` markers, and injects them into matching `>id` regions in `docs/`.

You can also use subcommands for one-off operations. The main one is `inject`:

### Basic Injection

Mark a region in your source file with `injm begin` and `injm end` comments:

`dest.rs`

```rust
fn main() {
    // injm begin
    // injm end
}
```

Then pipe content into `injm`:

```bash
echo -n 'println!("Hello, world!")' | injm inject --output dest.rs
```

Result:

`dest.rs`

```rust
fn main() {
    // injm begin
println!("Hello, world!");
    // injm end
}
```

Running `injm inject` again will replace the content between the markers:

```bash
cat src.txt | injm inject --output dest.rs
```

### Inject into a Specific Region

Give a region an output ID with `>id`, then target it with `--id`:

`dest.rs`

```rust
fn main() {
    // injm begin >greeting
    // injm end

    // injm begin >farewell
    // injm end
}
```

Inject into a specific region:

```bash
echo -n 'println!("Hello!")' | injm inject --output dest.rs --id greeting
```

Inject into multiple regions at once:

```bash
echo -n 'println!("Hello!")' | injm inject --output dest.rs --id greeting --id farewell
```

If `--id` is not specified, only regions **without** an ID are injected; regions with a `>id` are left untouched.

### Sync Between Files

Instead of piping from stdin, copy content between files with `--input`.
Mark the source region with `<id` (the content to read) and the destination
region with `>id` (where it goes):

`src.rs`

```rust
fn main() {
    // injm begin <hello
    println!("Hello, world!");
    // injm end
}
```

`dest.rs`

```rust
fn main() {
    // injm begin >hello
    // injm end
}
```

Then sync:

```bash
injm inject --input src.rs --output dest.rs
```

`dest.rs` becomes:

```rust
fn main() {
    // injm begin >hello
    println!("Hello, world!");
    // injm end
}
```

A region may read from several sources by listing multiple `<id` markers.
If a `>id` in the output has no matching `<id` in the input, `injm` reports
the missing ID and exits with an error.

### Marker Region Configuration

Per-region options can be appended to the `injm begin` marker as `:option=value`
tokens. They are set on the output region (the `>id` marker) and applied to
whatever content is injected into it. Multiple options may be combined in any
order:

```rust
// injm begin >id :offset=1 :trim=true :indent=4


// injm end
```

**`:offset=N`** (default `0`) — extend the region being replaced by `N` lines beyond the markers on each side, so lines just inside the markers survive injection. Useful for wrapper lines:

`dest.tex`

```latex
% injm begin >id :offset=1
\begin{minted}{rust}
\end{minted}
% injm end
```

Only the content between `\begin{minted}` and `\end{minted}` is replaced; the wrapper lines are kept. An offset that exceeds the block's own content reports an error.

**`:trim` / `:trim=true`** (default `false`) — remove leading and trailing blank lines from the injected content:

```rust
// injm begin >id :trim=true
// injm end
```

**`:indent=N`** (default unset) — rebase the minimum indentation of the injected
content to `N` columns, preserving the relative indentation of the other lines.
The example below injects an 8-column-indented region at 4 columns instead:

`src.rs`

```rust
fn main() {
    if true {
        // injm begin <id
        if true {
            println!("Hello world");
        }
        // injm end
    }
}
```

`dest.rs`

```rust
fn main() {
    // injm begin >id :indent=4
    // injm end
}
```

Becomes:

```rust
fn main() {
    // injm begin >id :indent=4
    if true {
        println!("Hello world");
    }
    // injm end
}
```

### Multiple Files and Globs

`--input` and `--output` accept multiple values and glob patterns:

```bash
# Multiple explicit files
injm inject --input src1.rs src2.rs --output dest.rs

# Sync to multiple outputs
injm inject --input src.rs --output out1.rs out2.rs

# Glob patterns
injm inject --input "src/**/*.rs" --output "docs/"

# Multiple globs
injm inject --input "mod_a/**/*.rs" "mod_b/**/*.rs" --output dest.rs
```

### Excluding Files

Skip files with `--exclude` (or `-e`). Accepts glob patterns matched against
absolute paths:

```bash
# Exclude a specific file
injm inject --input src/ --output dest/ --exclude src/legacy.rs

# Exclude with glob patterns
injm inject --input src/ --output dest/ --exclude "**/vendor/**" "**/*.generated.rs"

# Multiple flags
injm inject --input src/ --output dest/ -e "target/**" -e "node_modules/**"
```

Exclusion applies to both `--input` and `--output` files. The same patterns can also be set in `injm.toml` (see [Project Configuration](#project-configuration)).

### `.gitignore` Integration

By default, `injm` respects `.gitignore` rules — any file matched by `.gitignore` is skipped:

```bash
# Files in .gitignore are automatically excluded
injm inject --input src/ --output dest/
```

To include gitignored files, use `--no-gitignore`:

```bash
injm inject --input src/ --output dest/ --no-gitignore
```

### Project Configuration

Create an `injm.toml` in your project root to define input sources, output
destinations, and exclusion patterns:

```toml
input = ["examples/*.md"]
output = ["docs/**"]
exclude = [
    "target/**",
    "vendor/**"
]
```

When `injm.toml` is present, you can run `injm` **without any subcommand**:

```bash
injm
```

This reads the config, injects content from `input` into matching regions in
`output`, and writes the result. Equivalent to:

```bash
injm inject --input "examples/*.md" --output "docs/**" --exclude "target/**" --exclude "vendor/**"
```

You can also point to a different config file with `--config`:

```bash
injm --config path/to/injm.toml
```

Config values are merged with CLI flags — CLI arguments take precedence:

```bash
# Merges config's output with --exclude from CLI
injm --config injm.toml --exclude "temp/**"
```

### List Markers

Preview all marker regions across files:

```bash
injm list src/
```

Output:

```
+-------------+----------+--------+-------+
| File        | ID       | Type   | Lines |
+-------------+----------+--------+-------+
| src/main.rs | hello    | output | 6-7   |
+-------------+----------+--------+-------+
| src/main.rs | hello    | input  | 23-24 |
+-------------+----------+--------+-------+
| src/cli.rs  | greeting | input  | 1-2   |
+-------------+----------+--------+-------+
```

JSON output:

```bash
injm list src/ --format json
```

Accepts positional arguments (files, globs, or directories). Falls back to
current directory when no argument is given.

### Dry Run

Preview the result without writing to the file:

```bash
cat src.txt | injm inject --output dest.rs --dry-run
```

To see a unified diff of what would change instead of the full file, add `--diff`:

```bash
cat src.txt | injm inject --output dest.rs --dry-run --diff
```

These flags also work with the root command when using `injm.toml`:

```bash
# Preview all changes from config
injm --dry-run

# Show unified diff of what would change
injm --dry-run --diff
```

### Check

Verify that all output blocks (`>id`) contain the same content as their matching input blocks (`<id`):

```bash
injm check src/main.rs
```

If all blocks are synchronized, `injm check` exits 0 and prints:

```
all marker blocks are synchronized
```

If any are out of sync, it exits non-zero and lists each mismatch:

```
src/main.rs:12-14: output block `hello` is out of sync
```

To see a unified diff of what each out-of-sync block should contain, use
`--diff`:

```bash
injm check src/main.rs --diff
```

Accepts files, globs, or directories as arguments. Falls back to current directory when no argument is provided.

## Supported Languages

`injm` uses [tree-sitter](https://tree-sitter.github.io/tree-sitter/) to parse source files, so markers are detected from actual comment nodes — not from string literals or other non-comment content.

Supports any language recognized by [tree-sitter-language-pack](https://github.com/kreuzberg-dev/tree-sitter-language-pack), including:

- Rust, C, C++
- Python, Ruby
- JavaScript, TypeScript
- Go, Java
- And [300+ more](https://github.com/kreuzberg-dev/tree-sitter-language-pack)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Roadmap

See [ROADMAP.md](ROADMAP.md).

## License

MIT

## Acknowledgement

- [clap-rs/clap](https://github.com/clap-rs/clap): A full featured, fast Command Line Argument Parser for Rust.
- [ignore](https://github.com/BurntSushi/ripgrep/tree/master/crates/ignore): The ignore crate provides a fast recursive directory iterator that respects various filters such as globs, file types and .gitignore files. This crate also provides lower level direct access to gitignore and file type matchers.
- [mitsuhiko/similar](https://github.com/mitsuhiko/similar): A high level diffing library for rust based on diffs
- [rust-lang/glob](https://github.com/rust-lang/glob): Support for matching file paths against Unix shell style patterns.
- [serde-rs/serde](https://github.com/serde-rs/serde): Serialization framework for Rust.
- [xberg-io/tree-sitter-language-pack](https://github.com/xberg-io/tree-sitter-language-pack): Comprehensive tree-sitter grammar compilation with polyglot bindings — Rust, Python, Node.js, Go, Java, Ruby, Elixir, PHP, C#, WASM, Dart, Kotlin-Android, Swift, Zig, and CLI. 306+ languages.
- [zhiburt/tabled](https://github.com/zhiburt/tabled): An easy to use library for pretty print tables of Rust structs and enums.
