# Codebase Concerns

**Analysis Date:** 2026-04-26

## Tech Debt

**Core library does not compile:**
- Issue: `src/lib.rs` uses `rand 0.9.2` APIs that were removed in `rand` 0.9 / `rand_core` 0.9. `OsRng` now only implements `TryRngCore`, not `RngCore` directly. Code calls `rng.next_u64()` and passes `&mut rng` to `SliceRandom::shuffle`, both of which fail to compile.
- Files: `src/lib.rs`
- Impact: **Total build failure**. `cargo build`, `cargo test`, and `cargo clippy` all fail. No artifact can be produced.
- Fix approach: Replace `let mut rng = OsRng;` with `let mut rng = OsRng.unwrap_mut();` or switch to `rand::rng()` (thread-local RNG) which implements `RngCore`. Alternatively, downgrade to `rand = "0.8"` to match existing code.

**Stub binary and missing crate structure:**
- Issue: `src/main.rs` is a "Hello, world!" placeholder. The expected `src/bin/cli.rs` and `src/bin/gui.rs` do not exist. `Cargo.toml` has no `[[bin]]` stanzas.
- Files: `src/main.rs`, `Cargo.toml`
- Impact: Project cannot be run as a CLI or GUI despite packaging metadata claiming both.
- Fix approach: Implement `src/bin/cli.rs` and `src/bin/gui.rs` as outlined in `AGENTS.md`, and add corresponding `[[bin]]` entries to `Cargo.toml`.

**Unused heavyweight dependencies:**
- Issue: `Cargo.toml` declares `gtk4`, `nix`, `serde`, `toml`, and `unicode-segmentation`, yet none are referenced in the source code. `gtk4` in particular pulls in a massive dependency tree, inflating compile times and binary size for no benefit.
- Files: `Cargo.toml`
- Impact: Bloated lockfile, longer builds, larger attack surface.
- Fix approach: Remove unused crates from `Cargo.toml`. Re-add them only when the features that need them are actually implemented.

**Formatting drift:**
- Issue: `cargo fmt -- --check` reports import ordering violations and an extraneous blank line in `src/lib.rs`.
- Files: `src/lib.rs`
- Impact: Inconsistent style; CI linting would fail if formatting checks were enabled.
- Fix approach: Run `cargo fmt` and enforce it in CI.

## Known Bugs

**Entropy calculation inconsistent with generation validation:**
- Issue: `generate_password` rejects `config.length < groups.len()`, but `calc_entropy` allows the same configuration and computes a misleading entropy value.
- Files: `src/lib.rs` (`generate_password` at line 66, `calc_entropy` at line 102)
- Trigger: Call `calc_entropy` with `length = 1` and all four character groups enabled.
- Workaround: None — the API permits invalid states.

**Hardcoded charset index mapping:**
- Issue: `enabled_groups` assumes a fixed order for `charset_groups` (0 = lowercase, 1 = uppercase, 2 = digits, 3 = symbols) using magic indices.
- Files: `src/lib.rs` (`enabled_groups`, line 128)
- Trigger: Reordering `DEFAULTS.charset_groups` or adding a new group silently changes which flags control which sets.
- Workaround: None — the mapping is implicit.

## Security Considerations

**Secrets returned in non-zeroized `String`:**
- Risk: `generate_password` returns a plain `String`. While intermediate state (`picks`) is wrapped in `Zeroizing<Vec<char>>`, the final `String` is not zeroized on drop, potentially leaving password material in freed heap memory.
- Files: `src/lib.rs` (`generate_password`, line 96)
- Current mitigation: `Zeroizing` used for internal buffer only.
- Recommendations: Return `zeroize::Zeroizing<String>` or a dedicated secret type. Document that callers must zeroize if they move the secret out.

**No clipboard cleanup implemented:**
- Risk: `AGENTS.md` and `PLAN.md` describe secure clipboard integration and optional cleanup commands (e.g., `cliphist`), but no clipboard code exists.
- Files: N/A (missing)
- Current mitigation: None.
- Recommendations: Implement clipboard access in a dedicated `src/clipboard/` module, ensure cleanup commands are opt-in, and zeroize buffers after copying.

**No tests for RNG failure paths:**
- Risk: `rand 0.9` makes `OsRng` fallible via `TryRngCore`. The code currently unwraps implicitly by using `RngCore` methods. If the OS RNG fails (e.g., early boot, restricted container), behavior is a panic.
- Files: `src/lib.rs` (`random_index`, line 188)
- Current mitigation: None.
- Recommendations: Adopt `TryRngCore` explicitly, propagate errors via `PasswordError`, and add tests for simulated RNG failure.

## Performance Bottlenecks

**Quadratic character pool construction:**
- Problem: `build_character_pool` uses `Vec::contains` inside a nested loop, yielding `O(n * m)` complexity where `n` is total characters and `m` is pool size.
- Files: `src/lib.rs` (`build_character_pool`, line 174)
- Cause: Manual deduplication instead of a `HashSet`.
- Improvement path: Build the pool with a `HashSet<char>` and convert to `Vec` once.

**Entropy calculation does not account for guaranteed-character constraint:**
- Problem: `calc_entropy` treats all positions as drawn from the full pool, ignoring the fact that `generate_password` guarantees at least one character from each enabled group. This slightly underestimates true entropy.
- Files: `src/lib.rs` (`calc_entropy`, line 120)
- Cause: Simplified formula.
- Improvement path: Adjust the entropy model to include the combinatorial benefit of the guaranteed-character constraint, or document the conservative approximation.

## Fragile Areas

**Core library (`src/lib.rs`) — monolithic module:**
- Files: `src/lib.rs`
- Why fragile: All logic (generation, entropy, filtering, RNG) lives in a single 200-line module with no tests. Any refactor risks breaking generation or entropy math without detection.
- Safe modification: Extract sub-modules (`charset`, `entropy`, `rng`) before adding features. Add property tests before refactoring math.
- Test coverage: **0%**. No unit, property, or integration tests exist.

**Dependency on `rand` 0.9 bleeding-edge API:**
- Files: `Cargo.toml`, `src/lib.rs`
- Why fragile: The crate adopted `rand 0.9` immediately after release without updating code for breaking changes (`TryRngCore`, `RngCore` delegation via `DerefMut`).
- Safe modification: Pin to a known-working minor version or migrate fully to the new API.
- Test coverage: None.

## Scaling Limits

**Configuration parsing — not implemented:**
- Current capacity: Hard-coded `DEFAULTS` constant only.
- Limit: No runtime customization of length, charset, or entropy target without recompiling.
- Scaling path: Implement XDG-compliant TOML config parsing in `src/config/` using the already-declared `serde` + `toml` dependencies.

**Test infrastructure — absent:**
- Current capacity: Zero automated tests.
- Limit: Cannot validate correctness, entropy math, or security properties as the codebase grows.
- Scaling path: Add a `tests/` directory with unit tests for `generate_password` and `calc_entropy`, plus property tests (e.g., `proptest` or `quickcheck`) for entropy bounds and charset coverage.

## Dependencies at Risk

**`rand` / `rand_core` 0.9:**
- Risk: The project compiles against the latest `rand` 0.9 series but uses APIs that no longer exist. Future `rand` patches will not restore the old behavior.
- Impact: Permanent build failure until code is updated.
- Migration plan: Update `src/lib.rs` to use `OsRng.unwrap_mut()` or `rand::rng()`, and replace `SliceRandom::shuffle` with the `rand::seq::SliceRandom` trait bound compatible with the new RNG handle.

## Missing Critical Features

**CLI frontend:**
- Problem: `src/bin/cli.rs` does not exist. `cargo run` executes a "Hello, world!" stub.
- Blocks: Users cannot generate passwords from the command line.

**GTK GUI frontend:**
- Problem: `src/bin/gui.rs` does not exist. Despite `gtk4` being a dependency, there is no UI code.
- Blocks: Interactive parameter exploration and clipboard integration via GTK APIs.

**Configuration system:**
- Problem: No config parsing, no XDG paths, no presets.
- Blocks: Persistent user settings and policy presets (NIST, corporate, archival).

**CI build/test pipeline:**
- Problem: `.github/workflows/opencode.yml` only triggers an external bot. There is no workflow that runs `cargo build`, `cargo test`, or `cargo clippy`.
- Blocks: Automated detection of compilation failures and regressions.

## Test Coverage Gaps

**All library functions:**
- What's not tested: `generate_password`, `calc_entropy`, `enabled_groups`, `filtered_groups`, `build_character_pool`, `random_index`.
- Files: `src/lib.rs`
- Risk: Entire core logic is unverified. Bugs in entropy math, filtering, or RNG usage will go unnoticed.
- Priority: **Critical**.

**Entropy math validation:**
- What's not tested: No validation that `calc_entropy` matches known reference values or property-based bounds (e.g., entropy increases with length and pool size).
- Files: `src/lib.rs`
- Risk: Incorrect quantum-adjusted or classical entropy figures mislead users about security.
- Priority: **High**.

**Config parsing and edge cases:**
- What's not tested: Empty charsets, exhausted groups after `chars_to_avoid` filtering, unicode vs ASCII filtering, zero-length passwords.
- Files: `src/lib.rs`
- Risk: Panics or incorrect results on edge-case inputs.
- Priority: **High**.

---

*Concerns audit: 2026-04-26*
