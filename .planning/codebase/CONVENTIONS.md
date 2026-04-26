# Coding Conventions

**Analysis Date:** 2026-04-26

## Naming Patterns

**Files:**
- Rust source files use `snake_case.rs`
- Binary entry points live in `src/bin/` or `src/main.rs`
- Library code lives in `src/lib.rs` or `src/<module>.rs`

**Functions / Variables:**
- Use `snake_case` for functions, variables, and modules
  - Example: `generate_password`, `enabled_groups`, `build_avoid_set`

**Types / Structs / Enums:**
- Use `PascalCase` for structs, enums, and type aliases
  - Example: `Defaults`, `PasswordError`, `Result<T>`

**Constants:**
- Use `SCREAMING_SNAKE_CASE` for compile-time constants
  - Example: `DEFAULTS` in `src/lib.rs`

**Error Variants:**
- Use descriptive `PascalCase` enum variants with `thiserror` derive macros
  - Example: `EmptyLength`, `NoCharsetGroups`, `LengthTooShort`

## Code Style

**Formatting:**
- Tool: `rustfmt` (default configuration; no `rustfmt.toml` present)
- Run: `cargo fmt`
- Check: `cargo fmt -- --check`
- Current state: minor ordering drift detected in `src/lib.rs` (`rand` imports are not in canonical order)

**Linting:**
- Tool: `clippy` (default configuration; no `clippy.toml` present)
- Run: `cargo clippy --all-targets --all-features`
- Key rule: `unsafe_code = "forbid"` is set in `Cargo.toml` under `[lints.rust]`

**License Headers:**
- Source files should include SPDX license headers
- Example from `src/main.rs`:
  ```
  // Copyright 2025 Sundar Gurumurthy
  // SPDX-License-Identifier: BSD-3-Clause-No-Military-License
  ```
- Helper script: `addlicense.sh` applies headers automatically

## Import Organization

**Order (strict):**
1. `std` imports
2. External crate imports
3. Local module imports

**Example from `src/lib.rs`:**
```rust
use std::collections::HashSet;

use rand::{rngs::OsRng, seq::SliceRandom, RngCore};
use thiserror::Error;
use zeroize::Zeroizing;
```

**Path Aliases:**
- No custom path aliases configured; use standard module paths

## Error Handling

**Patterns:**
- Use `Result<T, E>` and the `?` operator throughout
- Define a library-specific `Result<T>` type alias for brevity:
  ```rust
  type Result<T> = std::result::Result<T, PasswordError>;
  ```
- Prefer `thiserror` for library errors (derive `Error` with `#[error(...)]` messages)
- Prefer `anyhow` for user-facing / CLI errors when context chaining is needed
- No `unwrap()` or `expect()` in production paths
- Panics are allowed **only** in tests

**Example from `src/lib.rs`:**
```rust
#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("password length must be greater than zero")]
    EmptyLength,
    #[error("password length {actual} is shorter than required minimum {required}")]
    LengthTooShort { required: usize, actual: usize },
}
```

## Logging

**Framework:** Not yet introduced (no `log`, `tracing`, or `env_logger` dependency)

**Guidelines (from project rules):**
- Never log secrets or generated passwords
- If logging is added later, log only operational metadata (length, flags, errors)
- Keep logs at `debug` or `trace` level for sensitive workflows

## Comments

**When to Comment:**
- Comment only non-obvious logic
- Prefer code clarity over comments
- Keep documentation technically precise; avoid marketing language

**Documentation Comments:**
- Use `///` for public API documentation
- Use `//` for inline implementation notes

## Function Design

**Size:**
- Keep functions focused and small
- Extract helpers for repeated logic (e.g., `enabled_groups`, `build_avoid_set`)

**Parameters:**
- Pass configuration by reference (`&Defaults`) to avoid copying
- Use `&str` over `String` for read-only string parameters

**Return Values:**
- Return `Result<T, PasswordError>` for fallible operations
- Use `Zeroizing<T>` from the `zeroize` crate for secret buffers

## Module Design

**Exports:**
- Use `pub mod password_generator` in `src/lib.rs` to expose the core module
- Public items inside the module are marked `pub`

**Barrel Files:**
- Not used; `src/lib.rs` directly declares modules with `pub mod`

## Security-Specific Conventions

**Secret Handling:**
- Wrap secret buffers in `zeroize::Zeroizing` to ensure memory is cleared after use
- Example: `let mut picks = Zeroizing::new(Vec::with_capacity(config.length));`

**Randomness:**
- Use `rand::rngs::OsRng` only (OS-backed CSPRNG)
- Do not implement custom RNGs or cryptographic primitives

**Clipboard / Shell:**
- Clipboard cleanup commands must be **opt-in**
- Shell execution must be explicit and documented
- Treat config files as untrusted input

---

*Convention analysis: 2026-04-26*
