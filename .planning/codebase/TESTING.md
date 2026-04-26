# Testing Patterns

**Analysis Date:** 2026-04-26

## Test Framework

**Runner:**
- Built-in Rust test harness (`cargo test`)
- No additional test runner configured (no `nextest`, `tarpaulin`, or `grcov` in dependencies)

**Assertion Library:**
- Standard `assert!`, `assert_eq!`, `assert_ne!` macros

**Run Commands:**
```bash
cargo test              # Run all tests
cargo test <name>       # Run a single test by name
cargo test -- --nocapture  # Show println! output during tests
```

## Test File Organization

**Location:**
- Unit tests: inline inside source files under `#[cfg(test)]` modules
- Integration / property tests: expected in `tests/` directory at crate root
- Currently **no tests exist** in the codebase

**Naming:**
- Test functions use `snake_case` prefixed with the behavior under test
- Example expected pattern: `fn generate_password_returns_error_when_length_is_zero()`

**Structure:**
```
you-shall-not-pass/
├── src/
│   └── lib.rs          # inline #[cfg(test)] modules
├── tests/
│   └── *.rs            # integration and property tests (expected, not yet present)
└── Cargo.toml
```

## Test Structure

**Suite Organization (expected pattern):**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_password_with_defaults_succeeds() {
        let config = DEFAULTS.clone();
        let password = generate_password(&config).unwrap();
        assert_eq!(password.len(), config.length);
    }
}
```

**Patterns:**
- Import `super::*` inside `#[cfg(test)]` modules for access to private items
- Use `unwrap()` and `expect()` freely in tests (project rule allows panics in tests only)
- Group related tests in nested `mod` blocks if the test file grows large

## Mocking

**Framework:** Not currently used; no mocking crates in `Cargo.toml`

**What to Mock (when needed):**
- OS-level clipboard APIs in GUI tests
- File-system operations for config loading tests
- Time-based clipboard cleanup triggers

**What NOT to Mock:**
- Do not mock `OsRng` or CSPRNG calls; tests should use real OS randomness
- Do not mock entropy math; verify exact calculations against known values

## Fixtures and Factories

**Test Data (expected pattern):**
```rust
fn test_config() -> Defaults {
    Defaults {
        length: 16,
        charset_groups: &[
            "abcdefghijklmnopqrstuvwxyz",
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            "0123456789",
            "!@#$%^&*",
        ],
        target_entropy: 80.0,
        chars_to_avoid: "",
        utf8_flag: false,
        alphabet_flag: true,
        caps_alphabet_flag: true,
        num_flag: true,
        symbol_flag: true,
        quantum_check: true,
    }
}
```

**Location:**
- Inline in `#[cfg(test)]` modules for unit tests
- Shared fixtures in `tests/common/mod.rs` for integration tests

## Coverage

**Requirements:** None enforced currently

**View Coverage (if added later):**
```bash
# With tarpaulin (not yet installed)
cargo tarpaulin --out Html

# With llvm-cov (not yet installed)
cargo llvm-cov --html
```

## Test Types

**Unit Tests:**
- Scope: Individual functions in `src/lib.rs`
- Approach: Test each public function with valid and edge-case inputs
- Priority functions to test:
  - `generate_password` — validate length, charset inclusion, error cases
  - `calc_entropy` — validate against hand-calculated values
  - `filtered_groups` — validate `chars_to_avoid` and `utf8_flag` behavior
  - `random_index` — validate unbiased distribution (statistical smoke test)

**Integration Tests:**
- Scope: End-to-end password generation workflows
- Approach: Test the public API surface from a consumer perspective
- Expected location: `tests/integration.rs`

**Property Tests:**
- Framework expected: `proptest` or `quickcheck` (not yet added)
- Target properties:
  - Generated password length always equals `config.length`
  - Entropy calculation is monotonic with respect to length and pool size
  - `quantum_check = true` always halves entropy vs `quantum_check = false`

**Fuzz Tests:**
- Framework expected: `cargo-fuzz` or `libfuzzer-sys` (not yet added)
- Targets:
  - Config parsing (TOML deserialization)
  - Input boundary values for `length`, `target_entropy`

**E2E Tests:**
- Not used; GUI testing would require GTK test harness infrastructure

## Common Patterns

**Async Testing:**
- Not applicable; codebase is currently fully synchronous

**Error Testing:**
```rust
#[test]
fn generate_password_fails_when_length_is_zero() {
    let mut config = DEFAULTS.clone();
    config.length = 0;
    let result = generate_password(&config);
    assert!(matches!(result, Err(PasswordError::EmptyLength)));
}
```

**Deterministic Testing:**
- Where possible, write deterministic tests using fixed inputs
- For randomness-dependent tests, verify structural properties (length, charset membership) rather than exact outputs

## Current State & Gaps

**Compilation Status:**
- `cargo test` currently fails because `src/lib.rs` does not compile
- Root cause: `rand` 0.9 API change — `OsRng` no longer implements `RngCore` directly via `&mut OsRng`
- Fix required before any tests can run

**Missing Tests:**
- No `#[cfg(test)]` module in `src/lib.rs`
- No `tests/` directory exists
- No test dependencies in `Cargo.toml`

**Recommended Test Dependencies:**
```toml
[dev-dependencies]
proptest = "1.6"  # for property testing
```

---

*Testing analysis: 2026-04-26*
