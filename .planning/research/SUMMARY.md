# Project Research Summary

**Project:** You Shall Not Pass — Password/Passphrase Generator
**Domain:** Rust CLI frontend wrapping a CSPRNG password-generation core library with quantum entropy modelling
**Researched:** 2026-04-26
**Confidence:** HIGH

## Executive Summary

This project is a **Rust CLI password generator** built as a thin presentation layer over an existing core library (`src/lib.rs`). Experts build this architecture by keeping the CLI strictly responsible for argument parsing, configuration mapping, library invocation, and output formatting — never containing entropy math, RNG calls, or generation logic. The core library handles all business logic so it can be shared with the future GTK GUI frontend without code duplication.

The recommended approach is a **5-step sequential build** within the CLI phase: define the clap derive struct, implement a `defaults_from_cli()` mapping function, wire the library's `generate_password()` and `calc_entropy()` calls, polish TTY-aware output, and cover with integration tests. Four high-severity security prerequisites must be addressed before or during wiring: the library must return `Zeroizing<String>` instead of plain `String` (Pitfall 1), `OsRng` fallibility must produce a graceful `PasswordError::RngFailure` rather than a panic (Pitfall 3), core dumps must be disabled at startup via `RLIMIT_CORE=0` (Pitfall 7), and `#[derive(Debug)]` must be audited to prevent secret leakage through panic backtraces (Pitfall 5).

The project's key competitive differentiator — **quantum entropy modelling** via Grover-adjusted brute-force estimation — is unique in the market. No existing password generator (pwgen, pass, xkcdpass) displays entropy values at all, let alone quantum-adjusted estimates. The CLI must prominently surface both classical and quantum entropy to establish this differentiator from launch. The primary risk is security hygiene: memory-side-channel leakage through non-zeroized return strings, shell history exposure through secret-bearing CLI arguments, and terminal scrollback persistence. All of these have clear prevention strategies documented in the pitfalls research and must be enforced at implementation time, not retrofitted later.

## Key Findings

### Recommended Stack

The stack is already established by `Cargo.toml` and requires **zero new dependencies** for the CLI phase. The CLI exclusively uses `clap` 4.5.53 (derive API) for argument parsing, `thiserror` 2.0.17 (via the existing `PasswordError` enum in `lib.rs`) for error display, and Rust's standard library (`std::process::ExitCode`, `std::io::IsTerminal`) for process lifecycle and output mode detection. The core library already provides `rand` 0.9.2 (OsRng CSPRNG), `zeroize` 1.8.2 (secret memory hygiene), and `serde`/`toml` (deferred for config phase).

**Key decisions:**
- **clap derive, not builder:** The derive API maps CLI flags directly to a struct — the idiomatic choice for a thin wrapper with static arguments. Builder is for dynamically generated args.
- **No `anyhow`:** `PasswordError` already has `Display` via `thiserror`. Adding `anyhow` would be an unnecessary dependency. Manual `match` on `PasswordError` variants with `eprintln!` is sufficient.
- **`src/main.rs`, not `src/bin/cli.rs` yet:** The multi-binary refactor (`cli.rs` + `gui.rs`) is deferred until the GTK GUI binary is added. Single binary in `src/main.rs` keeps `cargo run` simple.
- **No color crates, no progress bars, no interactive prompts:** Password generation is near-instantaneous and the CLI is designed for non-interactive/scriptable use. clap handles `--help` coloring natively.

### Expected Features

**Must have for launch (P1 — wired from existing library capabilities):**
- `-l/--length <N>` — Password length control (default 12, from library's `DEFAULTS`)
- `--no-alpha/--no-caps/--no-numbers/--no-symbols` — Charset group toggles
- `-q/--quantum` — Quantum entropy display toggle (the primary differentiator)
- Classical + quantum entropy display alongside every generated password
- `--help` with usage examples, `--version`
- Proper exit codes: 0 on success, 1 on bad args, 2 on internal errors
- Pipe mode: password-only on stdout, entropy/diagnostics on stderr

**Should have for v1.1 (P2 — additional wiring or small additions):**
- `--avoid/-x <CHARS>` — Explicit character exclusion (already functional in core, just needs wiring)
- `--json` — Structured output for scripting consumers (requires `serde::Serialize` on output struct; `serde` is already a dependency)
- `--verbose/-v` — Per-group character counts, pool size breakdown (requires minor `generate_password()` instrumentation in the library)

**Deferred to v2+ (P3 — explicitly excluded from CLI phase):**
- Multiple passwords (`--count N`) — deferred for output complexity and TTY detection requirements
- Passphrase generation (XKCD-style) — separate generation subsystem per PLAN.md
- Config file support (XDG TOML) — deferred to config phase
- Clipboard integration — deferred to dedicated clipboard phase
- Interactive mode, pronounceable passwords, reproducible (sha1-seeded) passwords — never implement

### Architecture Approach

The architecture follows the **Thin Frontend, Thick Library** pattern. The CLI binary (`src/main.rs`) has exactly three responsibilities: parse user input via clap, translate CLI arguments into the library's `Defaults` config struct via a dedicated `defaults_from_cli()` mapping function, and format/display results from `generate_password()` and `calc_entropy()`. All business logic — charset pool construction, unbiased OsRng sampling, entropy computation, and error definition — lives exclusively in `src/lib.rs`.

**Major components:**
1. **CLI Arg Definition** (`src/main.rs`, clap derive) — `#[derive(Parser)] struct Cli` with fields mirroring library's `Defaults` as user-facing flags
2. **Configuration Mapping Layer** (`src/main.rs`, `defaults_from_cli()`) — Translates inverted clap booleans (`--no-caps`) to library booleans (`caps_alphabet_flag: true`), applies `DEFAULTS` fallthrough, and handles the `chars_to_avoid` string-to-`&'static str` conversion
3. **Core Library** (`src/lib.rs`, `password_generator` module) — `Defaults` config struct, `PasswordError` enum, `generate_password()`, `calc_entropy()`, and private charset/RNG helpers
4. **Output Formatting** (`src/main.rs`) — `IsTerminal`-aware printing: TTY mode shows formatted password + entropy, pipe mode prints raw password to stdout and diagnostics to stderr

**Critical anti-patterns to avoid:**
- **NEVER** inline generation logic or RNG calls in `main.rs` — always call through `lib.rs`
- **NEVER** make library types depend on clap — library must remain UI-framework-agnostic
- **NEVER** split `src/lib.rs` prematurely — the current ~200-line module is well within maintainability limits
- **NEVER** log, `dbg!()`, or `eprintln!` the password value — even in debug builds

### Critical Pitfalls

The top 5 pitfalls requiring immediate attention during CLI development, ordered by severity:

1. **Non-zeroized return `String`** — `generate_password()` returns plain `String`, leaking password material into freed heap. **Fix:** Change signature to `Result<Zeroizing<String>>`. This is the single highest-impact security fix and must happen before any CLI wiring touches the password value.

2. **RNG panic on `OsRng` failure** — With `rand` 0.9, `OsRng` uses `TryRngCore` which can fail (early boot, restricted containers). Current code panics via implicit unwrap. **Fix:** Add `PasswordError::RngFailure` variant; propagate gracefully; CLI maps to user-friendly stderr message and non-zero exit code.

3. **Shell history leakage via command-line arguments** — Any flag accepting secret material (seed, key, custom charset) gets stored in `~/.bash_history`. **Fix:** Never define clap arguments for secrets. The current CLI phase only accepts non-secret parameters (length, booleans, char filter strings). Audit all `#[arg(...)]` attributes.

4. **Core dump leaks process memory** — On SIGSEGV/SIGABRT, kernel writes full process memory to disk, including any password buffers. **Fix:** Call `setrlimit(RLIMIT_CORE, 0)` at the top of `main()`. The `nix` crate is already in `Cargo.toml`.

5. **Entropy calculation inconsistency with generation** — `calc_entropy()` lacks the `length >= groups.len()` validation that `generate_password()` enforces, and the `log2(pool_size) * length` formula slightly overestimates entropy for short passwords with the one-per-group constraint. **Fix:** Add the same validation to `calc_entropy()`; document the formula as a conservative lower bound; defer precise combinatorial model to a later phase.

**Additional pitfalls** (TTY detection, panic handler, scrollback persistence, stdout buffering) are all CLI-specific output formatting concerns addressable during Step 4 (output polish) without blocking the core functionality pipeline.

## Implications for Roadmap

The CLI phase (Phase 4 of the project roadmap) decomposes into five sequential implementation steps with two parallel hardening tracks. This structure emerges directly from the architecture's dependency graph, the feature prioritization matrix, and the pitfalls' prevent-before-proceed requirements.

### Implementation Steps (Sequential)

#### Step 1: Library Hardening (Pre-CLI)

**Rationale:** Three critical pitfalls require library-level changes that the CLI wiring depends on. Attempting to wire the CLI against the current library signatures would build on a foundation that must be torn out.

**Delivers:**
- `generate_password()` returns `Zeroizing<String>` instead of `String` (Pitfall 1 fix)
- `PasswordError::RngFailure` variant added and propagated from `OsRng` calls (Pitfall 3 fix)
- `calc_entropy()` receives the same `length >= groups.len()` validation as `generate_password()` (Pitfall 6 fix)
- Library tests updated for new signatures and error paths

**Addresses:** Pitfalls 1, 3, 6 from PITFALLS.md
**Avoids:** Building CLI on signatures that will change; prevents the "works but insecure" false-positive milestone

#### Step 2: CLI Argument Struct (clap derive)

**Rationale:** Everything downstream depends on having parsed, validated user input. This step is testable immediately: `cargo run -- --help` produces complete auto-generated help.

**Delivers:**
- `#[derive(Parser)] struct Cli` in `src/main.rs` with all P1 flags: `--length`, `--no-caps`, `--no-numbers`, `--no-symbols`, `--quantum`, `--avoid`, `--help` (with `after_help` examples), `--version`
- Proper value parsers: `value_parser!(usize).range(1..)` on length, `default_value_t = DEFAULTS.length` for defaults
- Audit: no `--seed`, `--key`, `--password`, or secret-bearing args (Pitfall 4 prevention)

**Addresses:** All P1 features from FEATURES.md (flag definitions only, not behavior)
**Uses:** `clap` 4.5.53 derive API (`#[arg(short, long)]`, `#[command(version, after_help)]`)

#### Step 3: Configuration Mapping (defaults_from_cli)

**Rationale:** The glue between CLI types and library types. Isolated and testable as a unit before any library calls are made.

**Delivers:**
- `fn defaults_from_cli(args: &Cli) -> Defaults` — maps inverted clap booleans to library booleans, handles `chars_to_avoid` string conversion, applies `password_generator::DEFAULTS` fallthrough
- Unit tests: construct `Cli` instances, call mapping, assert `Defaults` field values

**Addresses:** Enables all P1 features by producing valid `Defaults` configs
**Avoids:** Library coupling to clap types (Anti-Pattern 2)

#### Step 4: Core Wiring + Output (generate → display)

**Rationale:** The main deliverable. Once Steps 1-3 are complete, this is a straightforward function call chain.

**Delivers:**
- `main()` calls `generate_password(&config)` and `calc_entropy(&config)` (twice: once with `quantum_check: false` for classical, once with `quantum_check: true` for quantum)
- TTY detection via `std::io::stdout().is_terminal()`:
  - **TTY mode:** Formatted output with password on its own line, classical + quantum entropy below
  - **Pipe mode:** Raw password only on stdout; entropy and warnings on stderr
- Explicit `stdout.flush()` after password print (Pitfall 8 mitigation)
- Startup hardening: `setrlimit(RLIMIT_CORE, 0)` call at top of `main()` (Pitfall 7)
- Custom panic handler (e.g., `human-panic`) to suppress backtrace secrets (Pitfall 5)
- Error propagation: `match` on `PasswordError` variants → user-friendly `eprintln!` messages → `ExitCode::FAILURE`
- `#[derive(Debug)]` audit on CLI struct — ensure no type transitively contains secret material

**Addresses:** All P1 features (functional), Pitfalls 2, 4, 5, 7, 8
**Uses:** `std::io::IsTerminal`, `std::process::ExitCode`, `nix::sys::resource::setrlimit`

#### Step 5: Output Polish + Integration Tests

**Rationale:** Separated from core wiring to prevent perfectionism blocking the functional milestone. Output polish and test coverage can proceed in parallel.

**Delivers:**
- Entropy values rounded to 1 decimal place: `"Classical entropy: 92.4 bits"`
- Quantum entropy clearly labeled: `"Quantum-adjusted (Grover): 46.2 bits"`
- `after_help` examples: `ysnp`, `ysnp -l 24`, `ysnp --no-symbols -q`
- Integration tests: argument parsing edge cases, defaults mapping correctness, TTY vs pipe output validation
- P2 feature wiring (optional, can be deferred): `--avoid` flag, `--json` output, `--verbose` breakdown

**Addresses:** UX pitfalls (raw f64 display, unclear labels, missing examples)
**Uses:** `assert_cmd` (recommended for integration tests), `serde_json` (for `--json` output)

### Dependency Graph

```
Step 1 (Library Hardening)  ──┐
                               ├──► Step 2 (Cli Struct) ──► Step 3 (Config Mapping)
                               │                                      │
                               │                                      ▼
                               └──────────────────────► Step 4 (Core Wiring + Output)
                                                                      │
                                                                      ▼
                                                              Step 5 (Polish + Tests)
```

Steps 1-3 are sequential dependencies. Step 4 depends on Steps 1-3. Step 5 can begin once Step 4 produces functional output. Steps 4 and 5 can be partially parallelized (formatting fine-tuning vs test authoring).

### Phase Ordering Rationale

1. **Library hardening precedes all CLI work** because the CLI wiring code will touch the password `String` directly. If the library returns plain `String`, every CLI code path becomes a security violation. Changing the signature later would require rewriting every call site. The hardening is a one-time, library-only change with no CLI dependencies.

2. **Argument parsing precedes generation** because every generation call depends on having a validated `Defaults` config. clap's parsing and validation layer catches user errors before they reach the library, reducing the error surface the library must handle.

3. **Configuration mapping is isolated** because the impedance mismatch between clap's inverted booleans (`--no-caps` → library's `caps_alphabet_flag: true`) is a pure translation concern. Testing it in isolation catches mapping bugs before they manifest as wrong passwords.

4. **Output polish is deferred to the end** to avoid the trap of spending hours on column alignment while the functional pipeline is untested. A working but ugly CLI that generates correct passwords is infinitely better than a beautiful CLI that doesn't work.

### Grouping Rationale for v1.1 (Post-Launch)

P2 features (`--avoid`, `--json`, `--verbose`) group naturally:
- `--avoid` and `--json` are pure CLI additions (no library changes) → low risk, can ship in a point release
- `--verbose` requires library instrumentation (`generate_password()` tracking per-group counts) → should be bundled with `--avoid` for a single library change cycle

### Research Flags

**Phases needing deeper research during planning:**
- **Step 1 (Library Hardening):** The `rand` 0.9 API migration (from `RngCore` to `TryRngCore`) is flagged in CONCERNS.md as current code not compiling. Needs focused API research on the exact `rand` 0.9 `OsRng` usage pattern and whether `rand::rng()` (thread-local) is a viable substitute. Also needs research on `Zeroizing<String>` API surface — what `Deref`/`Display` implementations are available for CLI printing.
- **Step 4 (Core Wiring):** The `--json` structured output format (when added) needs research on whether to print the entire JSON object to stdout (including password) or split password to stdout and JSON metadata to stderr. Two competing security philosophies: single JSON stream for scriptability vs. never mixing secrets with structured data.

**Phases with well-documented patterns (skip research-phase):**
- **Step 2 (Cli Struct):** clap derive API is exhaustively documented in official docs and the Rust CLI Book. Standard pattern with zero ambiguity.
- **Step 3 (Config Mapping):** A pure data translation function with no external dependencies. Trivially testable. No research needed.
- **Step 5 (Output Polish + Tests):** Formatting conventions are well-established in the Rust CLI Book's "Output for humans" and "Output for machines" chapters. Integration testing patterns with `assert_cmd` are standard.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Verified against official clap 4.6.1 docs, Rust CLI Book, project's own `Cargo.toml`. All dependencies already present. No new crates needed. |
| Features | HIGH | Core library API analyzed directly (`src/lib.rs`). Competitor analysis verified against pwgen man page, pass docs, xkcdpass README. Feature dependencies mapped. MVP scope clear. |
| Architecture | HIGH | Pattern drawn from project's own codebase analysis (`ARCHITECTURE.md`), AGENTS.md constraints, and Rust CLI Book architecture guidance. Thin-frontend/thick-library is the standard Rust crate pattern. |
| Pitfalls | HIGH | Each pitfall verified against official crate docs (`zeroize`, `rand`, `clap`), Rust CLI Book security guidance, real-world post-mortems (pwgen `-H` warning), and project codebase CONCERNS.md audit. Recovery costs assessed. |

**Overall confidence:** HIGH — All research areas are backed by official documentation, direct source code inspection, and project-specific constraints (AGENTS.md, PLAN.md, PROJECT.md). No significant gaps remain for the current CLI phase.

### Gaps to Address

No critical gaps exist for Phase 4 (CLI development). Two minor areas warrant attention during implementation:

- **`rand` 0.9 API migration details:** The CONCERNS.md flags that the current code doesn't compile against `rand` 0.9 due to `OsRng` API changes. The exact migration path (whether to use `OsRng.unwrap_mut()`, switch to `rand::rng()`, or adopt the `TryRngCore` trait directly) needs a concrete decision during Step 1 implementation. This is a focused API question, not a research gap — the `rand` 0.9 CHANGELOG and docs.rs provide the answer.

- **`Zeroizing<String>` display ergonomics:** The `zeroize` crate's `Zeroizing<T>` implements `Deref<Target=T>` and `Display` (when `T: Display`), so printing via `println!("{}", *password)` is straightforward. However, the exact ergonomics for `format!()` and string interpolation should be verified during Step 1 to ensure the CLI output code is clean. This is a 5-minute docs.rs check, not a research gap.

Both gaps are implementation decisions, not unknowns. They are documented here for transparency but do not block roadmap creation.

## Sources

### Primary (HIGH confidence)
- [clap 4.6.1 official docs (docs.rs)](https://docs.rs/clap/latest/clap/index.html) — Derive API reference, `ArgAction`, `ValueParser`, `CommandFactory` traits
- [Context7 / clap-rs/clap](https://context7.com/clap-rs/clap/llms.txt) — Custom value parsers, boolean flags, `default_value_t`, error handling via `Error::exit()` and `Error::print()`
- [Rust CLI Book — Nicer error reporting](https://rust-cli.github.io/book/tutorial/errors.html) — Patterns for `main() -> Result<()>`, custom error types, `anyhow::Context`
- [Rust CLI Book — Output for humans](https://rust-cli.github.io/book/tutorial/output.html) — `println!`/`eprintln!` conventions, stdout vs stderr separation
- [Rust CLI Book — Exit codes](https://rust-cli.github.io/book/in-depth/exit-code.html) — `std::process::ExitCode` and `ExitCode::SUCCESS`/`ExitCode::FAILURE`
- [Rust CLI Book — Signal handling](https://rust-cli.github.io/book/in-depth/signals.html) — Ctrl+C handling, graceful shutdown patterns
- [Rust CLI Book — Human communication](https://rust-cli.github.io/book/in-depth/human-communication.html) — Output formatting for terminal users
- [Rust CLI Book — Machine communication](https://rust-cli.github.io/book/in-depth/machine-communication.html) — Pipe-safe output patterns
- [Cargo Book: Cargo Targets](https://doc.rust-lang.org/cargo/reference/cargo-targets.html) — Binary/library target auto-discovery
- [zeroize 1.8.2 official docs (docs.rs)](https://docs.rs/zeroize/1.8.2/zeroize/) — `Zeroizing<T>`, `Zeroize` trait, `ZeroizeOnDrop`, `Debug` implementation behavior
- [pwgen(1) man page — Arch Linux](https://man.archlinux.org/man/pwgen.1.en) — Full feature inventory, security warnings about shell history and `-H` flag
- Project's `Cargo.toml`, `src/lib.rs`, AGENTS.md — Existing dependency versions, library API surface, security rules

### Secondary (MEDIUM confidence)
- [clap Derive Tutorial](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html) — `#[derive(Parser)]`, `#[arg(short, long)]`, `#[command(flatten)]`
- [clap Value Parsers](https://docs.rs/clap/latest/clap/) — Custom validation with `value_parser`, range constraints
- Codebase analysis at `.planning/codebase/ARCHITECTURE.md` — Existing component map, data flow diagrams, anti-patterns
- Codebase analysis at `.planning/codebase/CONCERNS.md` — Codebase-specific audit, `rand` 0.9 compilation issue, entropy validation gap
- password-store (`pass`) documentation — `zx2c4/password-store` repository and passwordstore.org
- XKCD-password-generator (`xkcdpass` v1.30.0) — Repository README and CLI help output

### Tertiary (LOW confidence)
- crates.io search for existing Rust password generator CLIs (`password-generator`, `rpg-util`) — Limited search depth; none have quantum entropy modeling or significant adoption

---

*Research completed: 2026-04-26*
*Ready for roadmap: yes*
