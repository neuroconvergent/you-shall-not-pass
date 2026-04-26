# Stack Research

**Domain:** Rust CLI wrapper over an existing core password-generation library
**Researched:** 2026-04-26
**Confidence:** HIGH (official docs + Context7 + Rust CLI Book verified)

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust (edition 2024) | stable 1.85+ | Language and toolchain | Already adopted by project; edition 2024 unlocks `impl Trait` in return position for `main()` and improved lifetime elision. Required by `Cargo.toml`. |
| clap (derive) | 4.5.53 | Argument parsing | Already in `Cargo.toml`. Derive API is the idiomatic choice for thin CLI wrappers: maps flags directly to struct fields, auto-generates `--help`, colored output, and suggested fixes. The builder API is more verbose for this use case. |
| thiserror | 2.0.17 | Library error types | Already in `Cargo.toml` and used by `src/lib.rs` for `PasswordError`. The `#[error("...")]` derive on the library's error enum gives human-readable error messages for free in the CLI. |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| rand (OsRng) | 0.9.2 | CSPRNG | Already in `Cargo.toml`. Used by `generate_password()` in the library. The CLI never touches RNG directly. |
| zeroize | 1.8.2 | Memory hygiene for secrets | Already in `Cargo.toml`. Used by the library to wrap password buffers. The CLI doesn't need to import this directly — the library handles it. |
| serde | 1.0.228 | Serialization (future config files) | Already in `Cargo.toml`. Not needed for current CLI phase but listed for completeness. |
| toml | 0.9.8 | TOML config parsing (future) | Already in `Cargo.toml`. Deferred to later phase. |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| `cargo build` | Compilation | Standard |
| `cargo test` | Test runner | Existing test infrastructure |
| `cargo clippy --all-targets --all-features` | Linting | Enforces AGENTS.md conventions |
| `cargo fmt` | Formatting | Rust edition 2024 style |
| `assert_cmd` (future) | CLI integration tests | Not needed now but recommended when CLI grows. Tests CLI as a binary: `assert_cmd::Command::cargo_bin("ysnp").unwrap().arg("--length").arg("20").assert().success()` |

## Installation

No new dependencies required. All CLI dependencies are already in `Cargo.toml`:

```toml
[dependencies]
clap = "4.5.53"          # CLI argument parsing (derive feature)
thiserror = "2.0.17"     # Error display (used in lib, consumed by CLI)
rand = "0.9.2"           # CSPRNG (used in lib only)
zeroize = "1.8.2"        # Secret zeroing (used in lib only)
```

The `clap` crate's `derive` feature is enabled by default in 4.x, so no explicit feature flag needed.

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| clap derive API | clap builder API | Builder API is better when: (a) args are dynamically generated, (b) you need runtime-conditional arg definitions, (c) you need fine-grained control over every aspect of arg configuration. For a thin wrapper mapping CLI flags to a known `Defaults` struct, derive is simpler and sufficient. |
| Manual `match` on `PasswordError` + `eprintln!` | `anyhow` / `eyre` | `anyhow` makes sense when: (a) error variants are heterogeneous and not worth a custom enum, (b) you're prototyping, (c) you need backtrace capture. Here, `PasswordError` is a well-defined enum with Display impls via `thiserror` — adding `anyhow` would be an unnecessary dependency. |
| `std::process::ExitCode` | `panic!` or bare `process::exit()` | `panic!` prints a noisy backtrace to users. `process::exit()` doesn't run drop handlers (bad for `Zeroizing` buffers). `ExitCode` is the idiomatic Rust 2024 way: `main() -> ExitCode` exits cleanly after running destructors. |
| `eprintln!` for user errors | `log` + `env_logger` | `log`/`env_logger` add two dependency crates for structured logging. The CLI has exactly one kind of failure (password generation returns an error) — a single `eprintln!` is sufficient. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `anyhow` | Adds an unnecessary dependency. `PasswordError` already has `Display` via `thiserror`. The CLI's error handling is simple: call `generate_password()`, match on the result, print the error to stderr, exit non-zero. `anyhow`'s context chains and backtraces add complexity without benefit here. | Manual `match` on `PasswordError` variants + `eprintln!("Error: {e}")` |
| `owo-colors` / `colored` / `ansi_term` | clap already handles colored output for `--help` and parse errors. The CLI output (password + entropy numbers) doesn't need terminal styling. Adding a color crate for the CLI output is over-engineering for a password generator. | Plain `println!` / `eprintln!` |
| `indicatif` (progress bars) | Password generation is near-instantaneous (microseconds). Progress bars would flash and disappear before the user sees them. | Nothing — no progress indication needed |
| `dialoguer` / `inquire` (interactive prompts) | The CLI is designed for non-interactive use (scriptable, pipeable). Interactive mode is a different interface paradigm and should be a separate feature flag if ever added. | Non-interactive: args only via clap flags |
| `clap_complete` (shell completions) | Nice-to-have but not in scope for the current CLI phase. Shell completions are a packaging concern, not a CLI architecture concern. Add later if needed — it's a single `#[command]` attribute and a `print_completions()` call. | Defer to packaging/deployment phase |
| GTK4 from CLI binary | `gtk4 = "0.10.3"` is in `Cargo.toml` but the CLI binary should NOT link against it. Use separate binary targets: `src/bin/cli.rs` for CLI, `src/bin/gui.rs` for GTK. The current `src/main.rs` should be renamed to `src/bin/cli.rs` when the GUI binary is introduced. | Keep CLI binary GTK-free. Use `src/bin/cli.rs` and `src/bin/gui.rs` as separate targets. |

## Stack Patterns by Variant

**For the current phase (CLI only, thin wrapper over existing lib):**
- Use clap derive with a struct that mirrors `Defaults` fields as clap args
- `main()` returns `std::process::ExitCode`
- One function: parse args → build `Defaults` → call `generate_password()` / `calc_entropy()` → print result
- Error path: `match` on `PasswordError`, `eprintln!`, return `ExitCode::FAILURE`

**If multiple subcommands are added later** (e.g., `ysnp password`, `ysnp passphrase`, `ysnp config`):
- Add `#[derive(Subcommand)]` enum under the main `Cli` struct
- Each subcommand gets its own `#[derive(Args)]` struct
- Use `#[command(subcommand_required = true)]` on the main command

**If machine-readable output is needed** (e.g., `--json`):
- Add an `--output` flag with `value_enum` (choices: `text`, `json`)
- Use `serde_json` for JSON serialization of output (already have `serde` in deps)
- Keep plain text as the default

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| `clap@4.5.53` | `clap@4.6.x` | 4.5.53 is a stable 4.x release. 4.6.x is backwards compatible. The derive API didn't change. `default_value_t`, `value_parser`, and `ArgAction` are identical. |
| `clap@4.5.53` + `thiserror@2.0.17` | Any Rust ≥ 1.74 | No known conflicts. They operate in different domains (arg parsing vs error display). |
| `thiserror@2.0.17` | `std::error::Error` | `PasswordError` derives `Error` which makes it compatible with any `Box<dyn Error>` signature if ever needed. |
| Rust edition 2024 | Rust 1.85+ | Required by `Cargo.toml` (`edition = "2024"`). The `unsafe_code = "forbid"` lint is edition 2024 syntax. |

## Sources

- [clap docs.rs 4.6.1](https://docs.rs/clap/latest/clap/index.html) — Derive API reference, ArgAction, ValueParser, CommandFactory traits. Verified: derive is the recommended approach for static CLI definitions. HIGH confidence.
- [Context7 / clap-rs/clap](https://context7.com/clap-rs/clap/llms.txt) — Custom value parsers, boolean flags with derive, default_value_t, error handling via Error::exit and Error::print. HIGH confidence.
- [Rust CLI Book — Nicer error reporting](https://rust-cli.github.io/book/tutorial/errors.html) — Patterns for `main() -> Result<()>`, `anyhow::Context`, custom error types. Verified: `thiserror`-based errors with manual matching is a valid alternative to `anyhow`. HIGH confidence.
- [Rust CLI Book — Output for humans](https://rust-cli.github.io/book/tutorial/output.html) — `println!` / `eprintln!` conventions, stdout vs stderr separation. HIGH confidence.
- [Rust CLI Book — Exit codes](https://rust-cli.github.io/book/in-depth/exit-code.html) — `std::process::ExitCode` and `ExitCode::SUCCESS` / `ExitCode::FAILURE`. HIGH confidence.
- Project's `Cargo.toml` and `src/lib.rs` — Verified existing dependency versions and library API surface (`Defaults`, `PasswordError`, `generate_password`, `calc_entropy`). HIGH confidence.

---

*Stack research for: You Shall Not Pass — CLI phase*
*Researched: 2026-04-26*
