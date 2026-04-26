<!-- refreshed: 2026-04-26 -->
# Architecture

**Analysis Date:** 2026-04-26

## System Overview

```text
┌─────────────────────────────────────────────────────────────┐
│                      Application Frontends                   │
│  CLI (unimplemented)        GUI (unimplemented)              │
│  `src/main.rs`              `src/bin/gui.rs` (planned)       │
└──────────────────┬──────────────────┬───────────────────────┘
                   │                  │
                   ▼                  ▼
┌─────────────────────────────────────────────────────────────┐
│                    Core Library (`src/lib.rs`)               │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  `password_generator` module                          │  │
│  │  - `Defaults` configuration struct                    │  │
│  │  - `PasswordError` error enum                         │  │
│  │  - `generate_password()`                              │  │
│  │  - `calc_entropy()`                                   │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────────┐
│  External Dependencies                                       │
│  - `rand::rngs::OsRng` (CSPRNG)                             │
│  - `zeroize` (secret memory clearing)                       │
└─────────────────────────────────────────────────────────────┘
```

## Component Responsibilities

| Component | Responsibility | File |
|-----------|----------------|------|
| Core Generator | Password generation, entropy calculation, charset filtering | `src/lib.rs` |
| CLI Entry | Command-line interface (placeholder only) | `src/main.rs` |
| GTK Entry | GUI frontend (not yet implemented) | `src/bin/gui.rs` (planned) |

## Pattern Overview

**Overall:** Single-crate library with multiple binary targets (planned)

**Key Characteristics:**
- Core logic lives in `lib.rs` as a public module; frontends are thin wrappers
- Configuration is struct-based (`Defaults`) with const defaults
- Error handling uses a custom `thiserror`-derived enum
- Security-critical state (password buffers) wrapped in `Zeroizing`
- OS-backed CSPRNG (`OsRng`) used directly; no custom RNG

## Layers

**Core Library Layer:**
- Purpose: Password generation, entropy math, charset management
- Location: `src/lib.rs`
- Contains: `password_generator` module with `Defaults`, `PasswordError`, `generate_password`, `calc_entropy`
- Depends on: `rand`, `thiserror`, `zeroize`
- Used by: `src/main.rs` (planned), `src/bin/gui.rs` (planned)

**Frontend Layer:**
- Purpose: User interaction (CLI args, GTK UI)
- Location: `src/main.rs` (CLI), `src/bin/gui.rs` (planned GUI)
- Contains: Entry points, argument parsing, UI widgets
- Depends on: Core library, `clap` (CLI), `gtk4` (GUI)
- Used by: End users

**Configuration Layer:**
- Purpose: XDG-compliant config loading and preset management
- Location: `src/config/` (planned)
- Contains: Config parsers, policy presets
- Depends on: `serde`, `toml`
- Used by: CLI and GUI frontends

**Clipboard Layer:**
- Purpose: Secure clipboard access and cleanup
- Location: `src/clipboard/` (planned)
- Contains: GTK clipboard wrappers, cleanup command runners
- Depends on: `gtk4`, `nix`
- Used by: GUI frontend

## Data Flow

### Primary Request Path (Password Generation)

1. Frontend constructs `Defaults` config (`src/main.rs` or `src/bin/gui.rs`)
2. Calls `generate_password(&config)` in `src/lib.rs:56`
3. `enabled_groups()` filters charset groups by boolean flags (`src/lib.rs:128`)
4. `build_avoid_set()` collects excluded characters (`src/lib.rs:147`)
5. `filtered_groups()` removes avoided/ non-ASCII chars per group (`src/lib.rs:151`)
6. `build_character_pool()` deduplicates into flat pool (`src/lib.rs:174`)
7. `random_index()` draws unbiased indices from `OsRng` (`src/lib.rs:188`)
8. One char drawn from each enabled group, remainder filled from pool
9. Result shuffled in-place and returned as `String`

### Entropy Calculation Path

1. Frontend calls `calc_entropy(&config)` in `src/lib.rs:102`
2. Same filtering pipeline as generation (steps 3–6 above)
3. Entropy computed as `log2(pool_size) * length` (`src/lib.rs:120`)
4. If `quantum_check` is true, result halved (`src/lib.rs:122`)
5. Returns `f64` entropy value

**State Management:**
- No mutable global state; all configuration passed as `&Defaults`
- Password buffers use `Zeroizing<Vec<char>>` for automatic memory clearing
- `OsRng` instantiated locally per call (no shared RNG state)

## Key Abstractions

**`Defaults`:**
- Purpose: Central configuration for generation parameters
- Examples: `src/lib.rs:9`
- Pattern: Plain struct with public fields and a `const DEFAULTS` instance

**`PasswordError`:**
- Purpose: Structured, user-displayable error cases
- Examples: `src/lib.rs:40`
- Pattern: `thiserror`-derived enum with formatted messages

**Unbiased Random Index:**
- Purpose: Rejection-sampling to avoid modulo bias when selecting characters
- Examples: `src/lib.rs:188`
- Pattern: Loop-draw `u64` from `OsRng`, reject values above cutoff

## Entry Points

**CLI Binary:**
- Location: `src/main.rs`
- Triggers: `cargo run`
- Responsibilities: Currently placeholder (`println!("Hello, world!")`)

**Library API:**
- Location: `src/lib.rs`
- Triggers: External crates importing `you_shall_not_pass`
- Responsibilities: `generate_password`, `calc_entropy`, `Defaults`, `PasswordError`

## Architectural Constraints

- **Threading:** Single-threaded; no async or threaded code yet. Future GTK UI will run on the GTK main loop.
- **Global state:** None. No `lazy_static`, `once_cell`, or module-level mutable state.
- **Circular imports:** None possible in current flat structure.
- **Unsafe code:** Explicitly forbidden at the crate level (`[lints.rust] unsafe_code = "forbid"` in `Cargo.toml:37`).
- **Entropy source:** Hard-coded to `rand::rngs::OsRng`; no pluggable RNG abstraction.
- **Memory hygiene:** Secret buffers wrapped in `Zeroizing`; not yet applied to final `String` return value.

## Anti-Patterns

### Placeholder Entry Point

**What happens:** `src/main.rs` is a "Hello, world!" stub that does not call the library.
**Why it's wrong:** It prevents end-to-end testing and gives no feedback to users.
**Do this instead:** Implement CLI argument parsing with `clap` and wire it to `generate_password` / `calc_entropy`.

### Monolithic Library Module

**What happens:** All core logic is in a single `password_generator` module inside `lib.rs`.
**Why it's wrong:** As features grow (config, clipboard, passphrase generation), the file will become unmaintainable.
**Do this instead:** Split into `src/config.rs`, `src/entropy.rs`, `src/generator.rs`, etc., re-exporting from `lib.rs`.

### No Tests

**What happens:** There are zero test files or inline `#[cfg(test)]` blocks.
**Why it's wrong:** Entropy math and unbiased sampling are security-critical and need property-based verification.
**Do this instead:** Add `tests/` directory and `#[cfg(test)]` modules with deterministic and property tests.

## Error Handling

**Strategy:** Custom error enum with `thiserror` for display formatting.

**Patterns:**
- Validation errors returned as `Result<T, PasswordError>` (`src/lib.rs:54`)
- Early returns with `?` operator not yet used (functions are shallow)
- No `unwrap()` or `expect()` in library code

## Cross-Cutting Concerns

**Logging:** Not implemented. Use `log` + `env_logger` for CLI; GTK has its own logging.
**Validation:** Input validation co-located with generation logic (`generate_password` validates `length`, `groups`, `pool`).
**Authentication:** Not applicable (no user accounts or sessions).
**Memory hygiene:** `Zeroizing` used for intermediate password buffer; not yet applied to returned `String`.

---

*Architecture analysis: 2026-04-26*
