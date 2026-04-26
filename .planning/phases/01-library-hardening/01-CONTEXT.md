# Phase 1: Library Hardening - Context

**Gathered:** 2026-04-26
**Status:** Ready for planning

## Phase Boundary

Secure the core library API before any CLI wiring begins. The library (`src/lib.rs`) must return secrets in `Zeroizing`-protected containers and must never panic due to RNG failure. Downstream CLI work (Phase 2) depends on this hardened API surface.

## Implementation Decisions

### RNG Strategy
- **D-01:** Replace direct `OsRng` instantiation with `rand::rng()` — the thread-local CSPRNG that seeds from OS entropy via `getrandom`. This eliminates the fallible RNG construction path that `OsRng::TryRngCore` would require in rand 0.9.
- **D-02:** `generate_password()` and `calc_entropy()` accept `rng: impl RngCore` as a parameter. Callers control the RNG source — production code passes `rand::rng()`, tests can inject deterministic or failing mocks. No global RNG state.

### the agent's Discretion
- `Zeroizing<String>` display ergonomics — how `Deref<Target=String>` interacts with `println!`/`format!` for downstream CLI.
- Error variant design for `PasswordError` — whether to add `RngFailure` variant, and how to structure diagnostic info.
- Entropy validation alignment — how `calc_entropy()` shares validation logic with `generate_password()` (shared helper, `CheckedDefaults`, etc.).
- Exact test strategy for RNG parameterization — mock vs seeded vs passthrough `rand::rng()`.
- rand 0.9 API migration specifics — `ThreadRng` usage patterns, removal of `OsRng` import.

## Specific Ideas

No specific references given — open to standard approaches.

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope and requirements
- `.planning/ROADMAP.md` — Phase 1 goal, requirements SEC-01/SEC-02, success criteria
- `.planning/REQUIREMENTS.md` — SEC-01 (`Zeroizing<String>` return), SEC-02 (graceful RNG failure handling)

### Research findings (PITFALLS.md is critical — identifies rand migration blocker)
- `.planning/research/PITFALLS.md` — Critical Pitfall #1 (Zeroizing<String> return), Pitfall #3 (RNG fallibility / rand 0.9 migration)
- `.planning/research/STACK.md` — Zero new dependencies, `clap` derive for CLI (relevant for Phase 2)
- `.planning/research/ARCHITECTURE.md` — Two-layer architecture, `defaults_from_cli()` mapping pattern
- `.planning/research/SUMMARY.md` — Library Hardening must complete before any CLI wiring

### Codebase context
- `.planning/codebase/ARCHITECTURE.md` — Component diagram, data flow, anti-patterns (no tests, monolithic module)
- `.planning/codebase/STACK.md` — rand 0.9.2, zeroize 1.8.2, thiserror 2.0.17 versions
- `.planning/codebase/CONCERNS.md` — Missing RNG fallibility handling, missing tests, Zeroizing not on return type

### Source files
- `src/lib.rs` — Current `password_generator` module: `Defaults`, `PasswordError`, `generate_password()`, `calc_entropy()`, `random_index()`
- `Cargo.toml` — rand 0.9.2 dependency

## Existing Code Insights

### Reusable Assets
- `Defaults` struct: configuration hub — changes to `generate_password()` signature propagate through this
- `PasswordError` enum: already uses `thiserror` derive — adding variants follows established pattern
- `random_index()`: unbiased rejection sampling — may need signature change when RNG is parameterized
- `Zeroizing<Vec<char>>`: already used for intermediate buffer — return type hardening extends this pattern

### Established Patterns
- No unsafe code (`unsafe_code = "forbid"`)
- `thiserror` for error handling, `Error` and `Display` derives
- `Zeroizing` for secret-bearing allocations
- `OsRng` used directly (to be replaced per D-01)
- Functions take `&Defaults` config reference (no mutation, no side effects)

### Integration Points
- `generate_password()` return type change to `Result<Zeroizing<String>>` is a **breaking change** — no callers exist yet (main.rs is placeholder), so no compat concerns
- `calc_entropy()` signature change (adding `impl RngCore` param) — same impact scope
- `random_index()` currently takes `&mut OsRng` — must change to `&mut impl RngCore`

## Deferred Ideas

None — discussion stayed within phase scope.

---

*Phase: 01-library-hardening*
*Context gathered: 2026-04-26*
