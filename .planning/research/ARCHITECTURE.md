# Architecture Research

**Domain:** Rust CLI wrapping a core library (password/passphrase generator)
**Researched:** 2026-04-26
**Confidence:** HIGH

## Standard Architecture

### System Overview

```
┌──────────────────────────────────────────────────────────────┐
│                    Presentation Layer                          │
│  src/main.rs (CLI binary)                                     │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  ┌──────────────────┐   ┌───────────────────────────┐   │ │
│  │  │  #[derive(Parser)]│   │  output formatting +      │   │ │
│  │  │  struct Cli       │   │  terminal presentation    │   │ │
│  │  └────────┬─────────┘   └───────────────────────────┘   │ │
│  │           │ maps to                                       │ │
│  │           ▼                                               │ │
│  │  ┌──────────────────────────────────────────────────┐    │ │
│  │  │  defaults_from_cli(&Cli) -> Defaults               │    │ │
│  │  │  Maps validated CLI args to library config struct  │    │ │
│  │  └──────────────────────────────────────────────────┘    │ │
│  │                                                           │ │
│  │  main() { parse → map → generate → print }               │ │
│  └─────────────────────────┬─────────────────────────────────┘ │
│                            │ calls                              │
├────────────────────────────┼───────────────────────────────────┤
│                    Core Library Layer                           │
│  src/lib.rs                                                    │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │  pub mod password_generator {                             │ │
│  │    pub struct Defaults {                                  │ │
│  │      length, charset_groups, target_entropy,              │ │
│  │      chars_to_avoid, utf8_flag, alphabet_flag,            │ │
│  │      caps_alphabet_flag, num_flag, symbol_flag,           │ │
│  │      quantum_check                                        │ │
│  │    }                                                       │ │
│  │    pub enum PasswordError { EmptyLength, NoCharsetGroups,  │ │
│  │      LengthTooShort, ExhaustedGroup, EmptyCharacterPool }  │ │
│  │    pub fn generate_password(&Defaults) -> Result<String>   │ │
│  │    pub fn calc_entropy(&Defaults) -> Result<f64>           │ │
│  │  }                                                         │ │
│  │  // Private: enabled_groups, build_avoid_set,              │ │
│  │  // filtered_groups, build_character_pool, random_index    │ │
│  └──────────────────────────────────────────────────────────┘ │
└──────────────────────────────┬───────────────────────────────┘
                               │ uses
                               ▼
┌──────────────────────────────────────────────────────────────┐
│  External Dependencies                                        │
│  ┌─────────────┐ ┌───────────┐ ┌──────────┐ ┌───────────┐   │
│  │ rand::OsRng │ │ zeroize   │ │ thiserror │ │ clap 4.5  │   │
│  │ (CSPRNG)    │ │ (secret   │ │ (error    │ │ (CLI args) │   │
│  │             │ │  cleanup) │ │  derive)  │ │            │   │
│  └─────────────┘ └───────────┘ └──────────┘ └───────────┘   │
└──────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | File | Typical Implementation |
|-----------|----------------|------|------------------------|
| CLI Arg Definition | Define and parse command-line flags (`--length`, `--no-caps`, `--quantum`) | `src/main.rs` | `#[derive(Parser)] struct Cli` via `clap` derive API |
| CLI-to-Library Mapping | Translate `Cli` struct fields into `Defaults` config | `src/main.rs` | Standalone `fn defaults_from_cli()` or `impl Defaults { fn from_cli() }` |
| CLI Output Formatting | Display generated password and entropy metrics to stdout | `src/main.rs` | Direct `println!` (no formatting library needed for phase 4) |
| Password Generation | Produce unbiased random password via rejection-sampled OsRng | `src/lib.rs` | `pub fn generate_password(&Defaults) -> Result<String>` |
| Entropy Calculation | Compute classical and quantum-adjusted entropy bits | `src/lib.rs` | `pub fn calc_entropy(&Defaults) -> Result<f64>` |
| Charset Management | Filter, deduplicate, and pool character groups | `src/lib.rs` (private) | `enabled_groups()`, `filtered_groups()`, `build_character_pool()` |
| Unbiased Sampling | Draw random indices without modulo bias | `src/lib.rs` (private) | `fn random_index(len: usize, rng: &mut OsRng) -> usize` (rejection sampling) |

## Recommended Project Structure

```
src/
├── lib.rs              # Core library: password_generator module
│                       # - pub: Defaults, PasswordError, generate_password, calc_entropy
│                       # - private: enabled_groups, build_avoid_set, filtered_groups,
│                       #            build_character_pool, random_index
├── main.rs             # CLI binary entry point (target of this phase)
│                       # - #[derive(Parser)] struct Cli (clap args)
│                       # - fn defaults_from_cli(&Cli) -> Defaults (mapping)
│                       # - fn main() { parse → map → generate → print }
├── bin/                # Future: additional binary targets
│   └── gui.rs          # Future: GTK GUI frontend (deferred)
├── config/             # Future: XDG config parsing (deferred)
│   └── mod.rs
├── entropy/            # Future: modular entropy math (deferred)
│   └── mod.rs
└── clipboard/          # Future: clipboard wrappers (deferred)
    └── mod.rs

tests/
├── cli_tests.rs        # CLI integration tests (test clap parsing, defaults mapping)
└── generator_tests.rs  # Library unit tests (entropy math, generation correctness)
```

### Structure Rationale

- **`src/main.rs` as single CLI binary (not `src/bin/cli.rs` yet):** The AGENTS.md target (`src/bin/cli.rs`) is for when multiple binaries exist (CLI + GUI). At this phase, only one binary exists. Using `src/main.rs` avoids unnecessary `Cargo.toml` `[[bin]]` config, keeps the workflow simple (`cargo run` without `--bin`), and defers the multi-binary refactor to when `gui.rs` is added. This follows the AGENTS.md principle: "Minimize new abstractions. Avoid speculative features."

- **`src/lib.rs` stays monolithic for now:** The codebase analysis correctly identifies that splitting into `src/entropy/`, `src/config/`, etc. is premature at ~200 lines. The lib.rs module boundary is clear and adequate for the current feature set. Splitting is deferred to when passphrase generation or config loading increase the file beyond maintainability.

- **`tests/` directory for integration tests:** Library-internal logic (entropy math) should be in `#[cfg(test)]` blocks within `src/lib.rs` for access to private functions. CLI argument parsing and mapping should be integration tests in `tests/` that exercise the public API.

## Architectural Patterns

### Pattern 1: Thin Frontend, Thick Library

**What:** The CLI binary (main.rs) is exclusively responsible for three operations: parse user input, call library functions, format output. All business logic, computation, and error handling live in the library.

**When to use:** Whenever you have a core library that multiple frontends need to share (CLI, GUI, future API). This is the standard Rust pattern for any crate that has both a `lib.rs` and binaries.

**Trade-offs:**
- Pro: Multiple binaries share the same logic without duplication. GUI wraps the exact same `generate_password()` call as CLI. Testing is easier — library tested independently of UI.
- Pro: Guarantees the GUI won't accidentally include RNG or entropy math in UI code (AGENTS.md requirement).
- Con: Adds one layer of indirection (CLI args → Defaults mapping). For very simple tools, this can feel like ceremony.

**Example — the anti-pattern to avoid:**

```rust
// DO NOT: inlining library logic in main.rs
fn main() {
    let mut rng = OsRng;
    let pool = "abcdefghijklmnopqrstuvwxyz...";
    // ... generation logic duplicated here ...
    println!("{password}");
}
```

**Example — the correct pattern:**

```rust
// main.rs: thin wrapper
use clap::Parser;
use you_shall_not_pass::password_generator::{self, Defaults};

#[derive(Parser)]
#[command(version, about = "Generate a quantum-resistant password")]
struct Cli {
    #[arg(short, long, default_value_t = 12)]
    length: usize,
    #[arg(long = "no-caps", default_value_t = false)]
    no_caps: bool,
    #[arg(long, default_value_t = false)]
    no_numbers: bool,
    #[arg(long, default_value_t = false)]
    no_symbols: bool,
    #[arg(long, default_value_t = false)]
    no_quantum: bool,
    #[arg(long, default_value = "")]
    avoid: String,
}

fn main() {
    let args = Cli::parse();
    let config = Defaults {
        length: args.length,
        caps_alphabet_flag: !args.no_caps,
        num_flag: !args.no_numbers,
        symbol_flag: !args.no_symbols,
        quantum_check: !args.no_quantum,
        chars_to_avoid: Box::leak(args.avoid.into_boxed_str()), // safe: process lifetime
        ..password_generator::DEFAULTS
    };

    let password = password_generator::generate_password(&config)
        .expect("password generation failed");
    let classical = password_generator::calc_entropy(&Defaults {
        quantum_check: false,
        ..config.clone()
    }).expect("entropy calculation failed");
    let quantum = password_generator::calc_entropy(&config)
        .expect("quantum entropy calculation failed");

    println!("Password:   {password}");
    println!("Classical entropy: {classical:.1} bits");
    println!("Quantum entropy:   {quantum:.1} bits");
}
```

### Pattern 2: Configuration Mapping Layer

**What:** A dedicated function (or `impl From<&Cli> for Defaults`) that translates CLI-typed arguments into the library's `Defaults` struct. This keeps the `main()` function minimal and the mapping logic testable in isolation.

**When to use:** When the CLI argument struct has different field names, types, or defaults than the library's configuration struct. Clap flags often use `--no-X` for boolean options while the library uses `X_flag: bool`. The mapping layer handles this impedance mismatch.

**Trade-offs:**
- Pro: Mappable in isolation for unit testing.
- Pro: Clean separation — change CLI flags without touching library types.
- Pro: The mapping can apply defaults and overrides in one place.
- Con: Extra struct or function. Overkill if CLI and library structs are 1:1.

**Example:**

```rust
/// Translate CLI arguments into library configuration.
///
/// Mapping rules:
/// - CLI boolean flags are inverted from library (--no-caps → caps_alphabet_flag: false)
/// - CLI `--avoid` string is converted to &'static str via leak (safe: process lifetime)
/// - Missing CLI flags fall through to `password_generator::DEFAULTS`
fn defaults_from_cli(args: &Cli) -> Defaults {
    Defaults {
        length: args.length,
        caps_alphabet_flag: !args.no_caps,
        num_flag: !args.no_numbers,
        symbol_flag: !args.no_symbols,
        quantum_check: !args.no_quantum,
        chars_to_avoid: Box::leak(args.avoid.clone().into_boxed_str()),
        ..password_generator::DEFAULTS
    }
}
```

### Pattern 3: Library-First Error Handling

**What:** The CLI never defines its own error types for operational failures. All errors originate from the library (`PasswordError`). The CLI's only responsibility is to surface them to the user via `Result` propagation or `expect()` with a user-friendly message.

**When to use:** When the library provides a comprehensive error enum that covers all failure modes. Adding CLI-specific error types would duplicate categories and create two sources of truth.

**Trade-offs:**
- Pro: Single error taxonomy. Library errors are the canonical source.
- Pro: GUI frontend can reuse the same error handling (consistent messages).
- Con: Library errors may be too technical for CLI users. Mitigate with `thiserror` display formatting that targets end-user readability.

## Data Flow

### Password Generation Flow

```
User types: $ y-s-n-p --length 20 --no-symbols
                    │
                    ▼
            clap parses Cli {
                length: 20,
                no_caps: false,
                no_numbers: false,
                no_symbols: true,
                no_quantum: false,
                avoid: "",
            }
                    │
                    ▼
            defaults_from_cli() maps to Defaults {
                length: 20,
                alphabet_flag: true,
                caps_alphabet_flag: true,
                num_flag: true,
                symbol_flag: false,
                quantum_check: true,
                ...
            }
                    │
                    ▼
            generate_password(&config)
                    │
        ┌───────────┼───────────┐
        ▼           ▼           ▼
    enabled_groups  build_avoid_set  filtered_groups
        │               │               │
        └───────┬───────┘               │
                ▼                       │
        build_character_pool            │
                │                       │
                ▼                       ▼
            random_index() (OsRng rejection sampling)
                │
                ▼
            [picks: Vec<char>] → shuffle → String
                    │
                    ▼
            Ok("Kx9#mP2vQn...")
                    │
                    ▼
    println!("Password:   Kx9#mP2vQn...")
```

### Entropy Display Flow

```
User types: $ y-s-n-p --quantum (or default)
                    │
                    ▼
            defaults_from_cli() → Defaults { quantum_check: true, ... }
                    │
                    ▼
            calc_entropy(&config)
                    │
                    ▼
            pool_size = build_character_pool(...).len()
            entropy = log2(pool_size) * length
            if quantum_check { entropy /= 2.0 }
                    │
                    ▼
            Ok(77.4)  // example: 12 chars × 6.45 bits, halved
                    │
        ┌───────────┴───────────┐
        ▼                       ▼
    Classical entropy      Quantum entropy
    (quantum_check: false)  (quantum_check: true)
        │                       │
        ▼                       ▼
    println!("Classical entropy: 154.8 bits")
    println!("Quantum entropy:    77.4 bits")
```

### Error Propagation Flow

```
User request
    │
    ▼
generate_password(&config)
    │
    ├── Err(PasswordError::EmptyLength)
    │   → "Error: password length must be greater than zero"
    │
    ├── Err(PasswordError::NoCharsetGroups)
    │   → "Error: no charset groups enabled for generation"
    │
    ├── Err(PasswordError::LengthTooShort { required: 4, actual: 2 })
    │   → "Error: password length 2 is shorter than required minimum 4"
    │
    └── Err(PasswordError::ExhaustedGroup { group_index: 2 })
        → "Error: character group at index 2 has no usable characters after filtering"
```

### Key Data Flows

1. **CLI Args → Library Config (mapping):** The CLI defines arguments as user-facing flags (`--length 20`, `--no-caps`). A mapping function translates these into the library's `Defaults` struct. This is the fault line between presentation and logic.

2. **Config → Password (generation):** The `Defaults` struct flows into `generate_password()`, which builds a character pool, samples it with unbiased random indices from `OsRng`, and returns a `Zeroizing`-cleaned `String`. The password never persists on disk or in logs.

3. **Config → Entropy (calculation):** The same `Defaults` struct (and a copy with `quantum_check: false` for classical entropy) flows into `calc_entropy()`, which computes `log2(pool_size) * length`. The CLI displays both classical and quantum-adjusted values side by side.

4. **Library Error → stdout (presentation):** Library errors propagate up via `Result<T, PasswordError>`. The CLI prints the `Display` message to stderr (via `eprintln!` or `expect()`) and exits with a non-zero code.

## Build Order Implications

The CLI phase has a natural build order within itself. These are implementation steps, ordered by dependency:

### Step 1: CLI Argument Struct (clap derive)

**What:** Define `#[derive(Parser)] struct Cli` in `src/main.rs` with all required flags.

**Depends on:** `clap` crate (already in `Cargo.toml`)

**Produces:** Parsed CLI arguments as typed Rust values (validated by clap before reaching library code).

**Rationale:** This must come first — everything else depends on having parsed user input. Testable immediately: `cargo run -- --help` should produce clap-generated help output.

### Step 2: Configuration Mapping (CLI → Defaults)

**What:** Implement `defaults_from_cli()` to translate the `Cli` struct into the library's `Defaults` struct.

**Depends on:** Step 1 (Cli struct), `src/lib.rs` (Defaults type)

**Produces:** A `Defaults` instance ready for library consumption.

**Rationale:** This is the glue between CLI and library. Must happen before any library calls. Testable as a unit (construct a `Cli`, call mapping, assert on `Defaults` fields).

### Step 3: Core Wiring (library call + output)

**What:** Replace the placeholder `println!("Hello, world!")` with calls to `generate_password()` and `calc_entropy()`, then print formatted results.

**Depends on:** Step 2 (mapping), `src/lib.rs` (generate_password, calc_entropy)

**Produces:** Working CLI that generates and prints passwords with entropy metrics.

**Rationale:** This is the main deliverable of the phase. Once Step 2 is done, this is a straightforward function call.

### Step 4: Output Polish

**What:** Refine output formatting — column alignment, entropy precision, error message clarity, pipe-safe output (password-only mode via `--quiet` / `--password-only`).

**Depends on:** Step 3 (working output)

**Produces:** Production-quality CLI output.

**Rationale:** Separated from Step 3 because formatting iteration is lower risk and should not block the functional milestone. Getting a working pipeline first, then polishing, prevents perfectionism blocking progress.

### Step 5: CLI-Specific Tests

**What:** Integration tests for argument parsing edge cases, mapping correctness, and output format expectations.

**Depends on:** Steps 1–4 (complete implementation)

**Produces:** Test coverage for the CLI layer.

**Rationale:** Tests can be added incrementally alongside each step, but the final push ensures all edge cases are covered before declaring the phase complete.

### Dependency Graph

```
Step 1 (Cli struct)
    │
    ▼
Step 2 (defaults_from_cli)
    │
    ▼
Step 3 (core wiring: generate + print)
    │
    ▼
Step 4 (output polish)
    │
    ▼
Step 5 (CLI tests)
```

Steps 1–3 are sequential dependencies. Steps 4 and 5 can be parallelized (one person polishes output, another writes tests). Steps 1–3 constitute the **minimum viable CLI**.

## Scaling Considerations

This is a local CLI tool, not a server application. Scaling concerns are about code complexity growth, not user concurrency.

| Stage | Concern | Approach |
|-------|---------|----------|
| Current (Phase 4) | Single CLI binary, one lib module | `src/main.rs` + `src/lib.rs` — flat is fine |
| Near-term (Phase 5: GUI) | Two binaries sharing library | Refactor `main.rs` → `src/bin/cli.rs`, add `src/bin/gui.rs`. Both call same `lib.rs` API. Add `[[bin]]` sections to `Cargo.toml`. |
| Mid-term (Phase 6: Config) | Config loading adds module | Extract `src/config.rs` from lib or main. `Defaults::from_file()` loads TOML. |
| Long-term (Phase 7: Full) | Multiple modules, complex feature set | Split lib.rs into `src/generator.rs`, `src/entropy.rs`, `src/config.rs`, re-exported from lib.rs |

### What Breaks First

1. **lib.rs size:** When passphrase generation and policy presets are added, the single `password_generator` module will exceed maintainability (~500+ lines). Split into `src/generator.rs` and `src/entropy.rs` at that point, re-exporting from `lib.rs`.

2. **CLI argument count:** As more flags are added (config file path, output format options, preset selection), the flat `Cli` struct becomes unwieldy. Use `clap`'s `#[command(flatten)]` to group related arguments into sub-structs (e.g., `CharsetArgs`, `OutputArgs`, `EntropyArgs`).

3. **Output formatting complexity:** Plain `println!` is adequate for Phase 4. If structured output is needed (JSON, YAML), add a formatting layer rather than inline formatting in `main()`.

## Anti-Patterns

### Anti-Pattern 1: CLI Doing Library Work

**What people do:** Put password generation logic, entropy math, or RNG calls directly in `main.rs` instead of calling through `lib.rs`.

**Why it's wrong:**
- GUI frontend must duplicate the code (AGENTS.md: "No entropy math or RNG in UI code").
- Harder to test (can't test library in isolation).
- Violates the thin-frontend architecture constraint from PROJECT.md.

**Do this instead:** All generation and entropy logic lives in `src/lib.rs`. `main.rs` only parses args, maps to `Defaults`, calls public library functions, and prints results.

### Anti-Pattern 2: Library Coupled to CLI

**What people do:** Define `clap`-derived types in `lib.rs`, or make library functions accept `clap::Args` types directly.

**Why it's wrong:**
- Ties the library to a specific CLI framework. If the GUI frontend needs different configuration, it can't.
- Library becomes impossible to test without clap parsing infrastructure.
- Added compile time and binary size for both frontends.

**Do this instead:** Library types (`Defaults`, `PasswordError`) are defined in `lib.rs` with zero dependency on `clap`. The mapping from CLI types to library types is a one-way translation in `main.rs`.

### Anti-Pattern 3: Premature Module Splitting

**What people do:** Create `src/generator.rs`, `src/entropy.rs`, `src/config.rs` with one module per function, even at 200 lines of total code.

**Why it's wrong:**
- Adds complexity (module declarations, cross-module imports, visibility management) without benefit.
- AGENTS.md says: "Minimize new abstractions. Avoid speculative features."
- The codebase analysis explicitly flags this.

**Do this instead:** Keep all core logic in `src/lib.rs` until a module naturally outgrows ~500 lines or has clearly separable concerns. The current `password_generator` module is ~200 lines — splitting is premature.

### Anti-Pattern 4: Silent Error Swallowing

**What people do:** Use `unwrap()` or `eprintln!("{}", e)` without `process::exit(1)` in `main()`.

**Why it's wrong:**
- `unwrap()` produces an unhelpful panic message with a backtrace for library errors.
- Missing `exit(1)` means the CLI returns success (exit code 0) even on failure, confusing scripts and pipelines.

**Do this instead:** Use `match` on `Result` with explicit error handling, or propagate with `?` in a `main() -> Result<(), Box<dyn Error>>` return type. Always exit with non-zero on failure:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let config = defaults_from_cli(&args);
    let password = password_generator::generate_password(&config)?;
    println!("{password}");
    Ok(())
}
```

Or if using `expect()`:
```rust
fn main() {
    let password = password_generator::generate_password(&config)
        .expect("Password generation failed");
}
// expect() panics → exit code 101, which is non-zero
```

### Anti-Pattern 5: Password Logging

**What people do:** Add `println!("Debug: generated password = {password}")` or pass the password to a logging framework.

**Why it's wrong:** This violates AGENTS.md security rule: "Never log secrets." The generated password is a secret. Logging it defeats the purpose of the tool and creates persistent traces of secrets.

**Do this instead:** Never log, `dbg!()`, or `eprintln!` the password value. If debugging is needed, log the length and entropy, never the password itself.

## Integration Points

### CLI → Library

| Aspect | Detail |
|--------|--------|
| **Call pattern** | Synchronous, single-threaded function calls |
| **Data passed** | `&Defaults` (immutable reference to config struct) |
| **Data returned** | `Result<String, PasswordError>` for generation; `Result<f64, PasswordError>` for entropy |
| **Ownership** | CLI owns `Defaults`. Library borrows it. Returned `String` is moved to CLI scope. |
| **Memory cleanup** | `Zeroizing` in library cleans intermediate password buffer. Final `String` is up to CLI to handle (dropped naturally at end of `main()`). |

### CLI → Terminal

| Aspect | Detail |
|--------|--------|
| **stdout** | Password and entropy metrics (can be piped) |
| **stderr** | Error messages (not piped by default) |
| **Exit code** | 0 on success, non-zero on failure (via panic or `process::exit`) |
| **Color** | None initially. clap auto-colors `--help` output. Consider `termcolor` or `owo-colors` for future polish. |

## Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| `src/main.rs` ↔ `src/lib.rs` | Direct function calls via `use you_shall_not_pass::password_generator` | Simplest possible boundary — no channels, traits, or abstractions. A function call. |
| `Cli` struct → `Defaults` struct | One-way mapping via `defaults_from_cli()` | This is the only translation layer. Both structs are plain data. |
| Library → OS | `rand::rngs::OsRng` calls | Library's only side effect. No filesystem, network, or environment variable access. |

## Sources

- [Cargo Book: Cargo Targets](https://doc.rust-lang.org/cargo/reference/cargo-targets.html) — binary/library target auto-discovery (HIGH confidence)
- [clap Derive Tutorial](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html) — `#[derive(Parser)]`, `#[arg(short, long)]`, `#[command(flatten)]` (HIGH confidence)
- [clap Value Parsers](https://docs.rs/clap/latest/clap/) — custom validation with `value_parser`, range constraints (HIGH confidence)
- Codebase analysis at `.planning/codebase/ARCHITECTURE.md` — existing component map, data flow diagrams, anti-patterns (HIGH confidence)
- AGENTS.md — architecture constraints: thin frontends, no UI in library, no unsafe (HIGH confidence)
- PROJECT.md — active milestone requirements, out-of-scope items, key decisions (HIGH confidence)

---

*Architecture research for: you-shall-not-pass CLI frontend*
*Researched: 2026-04-26*
