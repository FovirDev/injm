# Roadmap

## Vision

`injm` injects content into marked regions in source files.

## v0.1.0

- [x] Use `clap` to create the CLI interface
- [x] Detect programming language using `treesitter`
- [x] Extract comments from a giving file using `treesitter`
- [x] Replace content between comments `injm begin` and `injm end` with stdin input
- [x] Write the result back to the file using `-o` or `--output` flag

The usage looks like:

```bash
cat src.txt | injm -o dest.rs
```

## v0.2.0

- [x] Support `--dry-run` to preview changes without writing
- [x] Skip binary files
- [x] Error messages when markers are missing or mismatched

## v0.3.0

- [x] Support multiple marker pairs inside a file with an `ID` identifier
- [x] Allow the user to specify which block to insert

The usage looks like:

`dest.rs`

```rust
fn main() {
    // injm begin :first
    // injm end :first

    // injm begin :second
    // injm end :second
}
```

Then,

```bash
cat src.txt | injm -o dest.rs --id first
```

## v0.4.0

Synchronize between files.

- [x] Read source content from `--input` or `-i`
- [x] Use `<id` and `>id` to specify input or output
- [x] Report missing `<id` IDs

Example:

`src.rs`

```rust
fn main() {
    // injm begin <hello
    println!("Hello, world!")
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

Then,

```bash
injm -i src.rs -o dest.rs
```

And `dest.rs` becomes:

```rust
fn main() {
    // injm begin >hello
    println!("Hello, world!")
    // injm end
}
```

## v0.5.0

Synchronize directories.

- [x] Accept glob as input and output
- [x] Scan supported source files recursively
- [x] Create `list` subcommand to list all marker regions

## v0.6.0

Improve preview and verification.

- [x] Add check mode
- [x] Improve error messages (Show line numbers of mismatched markers)
- [x] Return non-zero exit code when synchronization is needed
- [x] Show unified diff output

Example:

```bash
injm check src/ docs/
```

Perfect for CI.

## v0.7.0

Improve project configuration.

- [x] Support `injm.toml`
- [x] Configure include/exclude patterns
- [x] Integrate with `.gitignore`
- [x] Implement `injm` command

Example:

```toml
input = ["examples/*.md"]
output = ["docs/**"]
exclude = [
    "target/**",
    "vendor/**"
]
```

Then simply run:

```bash
injm
```

## v0.8.0

Improve marker region configuration.

- [x] Provide offset option (default is `0`)
- [x] Support trim space option (default is `false`)
- [x] Allow the user to specify minimum indentation (default is `None`)

Example:

**Offset Option**

```tex
% injm begin >id :offset=1
\begin{minted}{rust}
\end{minted}
% injm end
```

After injection, original `\begin{}` and `\end{}` will not be replaced.

**Trim Space Option**

`src.rs`

```rust
// injm begin <id
fn main() {
    println!("Hello injm");
}

// injm end
```

`dest.rs`

```rust
// injm begin >id :trim=true
// injm end
```

After injection, `dest.rs` becomes:

```rust
// injm begin >id :trim=true
fn main() {
    println!("Hello injm");
}
// injm end
```

**Minimum Indentation**

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
    if true {
        println!("Hello world");
    }
    // injm end
}
```

## v1.0.0

- [ ] Create GitHub actions
- [x] Optimize release binary
