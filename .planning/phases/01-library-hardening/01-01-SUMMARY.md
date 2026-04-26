---
phase: 01-library-hardening
plan: 01
subsystem: security
tags: [rand, zeroize, rng, entropy, password-generator, rust]

# Dependency graph
requires:
  - phase: 00-initialization
    provides: project structure, Cargo.toml with rand 0.9, zeroize, thiserror
provides:
  - RNG-parameterized generate_password() and calc_entropy() (impl Rng parameter)
  - Zeroizing<String> return type for generate_password()
  - Consistent LengthTooShort validation across both public functions
  - No direct OsRng usage (compliant with rand 0.9 TryRngCore API)
affects: [02-cli-core, 03-output-security]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "RNG injected as impl Rng parameter (D-02) — no global/static RNG state"
    - "Zeroizing used on return type (SEC-01) — secret zeroized on drop"
    - "rand::rng() as production RNG source — infallible thread-local CSPRNG"

key-files:
  created:
    - .planning/phases/01-library-hardening/deferred-items.md
  modified:
    - src/lib.rs

key-decisions:
  - "D-02 affirmed: both public functions take rng: impl Rng for testability and API symmetry"
  - "Used impl Rng (not impl RngCore) for generate_password/calc_entropy because SliceRandom::shuffle in rand 0.9 requires R: Rng"
  - "rand::rng() thread-local CSPRNG replaces direct OsRng construction — eliminates fallible RNG path"

patterns-established:
  - "RNG is a function parameter, never a global or static"
  - "Secret-bearing returns use Zeroizing<T> wrapper"
  - "calc_entropy shares validation logic pattern with generate_password (same error variants in same order)"

requirements-completed: [SEC-01, SEC-02]

# Metrics
duration: 5min
completed: 2026-04-26
---

# Phase 01 Plan 01: Library Hardening — RNG Parameterization & Zeroizing Return

**Fixed rand 0.9 compilation (OsRng → impl Rng), parameterized both public functions with generic RNG, wrapped generate_password return in Zeroizing<String>, and aligned calc_entropy validation with generate_password's LengthTooShort check**

## Performance

- **Duration:** 5 min
- **Started:** 2026-04-26T13:18:00Z (approx)
- **Completed:** 2026-04-26T13:23:08Z
- **Tasks:** 2
- **Files modified:** 1 (src/lib.rs)

## Accomplishments
- Fixed total build failure caused by rand 0.9's `OsRng` implementing `TryRngCore` instead of `RngCore` — replaced with generic `impl Rng` parameter
- Both `generate_password()` and `calc_entropy()` now accept `rng: impl Rng` per D-02 (RNG parameterization)
- `random_index()` uses `&mut impl RngCore` — only needs `next_u64()`, minimal trait bound
- `generate_password()` returns `Result<Zeroizing<String>>` per SEC-01 — password zeroized on drop
- `calc_entropy()` now validates `config.length >= groups.len()` before computing entropy, matching `generate_password()`'s existing validation
- Pre-existing clippy warnings (unused `index`, needless lifetimes on `enabled_groups`) logged as deferred items

## Task Commits

Each task was committed atomically:

1. **Task 1: Fix rand 0.9 compilation and parameterize RNG** - `af08ea8` (feat)
2. **Task 2: Wrap password return in Zeroizing and align calc_entropy validation** - `e817d41` (feat)

**Plan metadata:** (to be committed after this summary)

## Files Created/Modified

- `src/lib.rs` - Core library hardened: RNG parameterization, Zeroizing return, aligned entropy validation
- `.planning/phases/01-library-hardening/deferred-items.md` - Pre-existing clippy warnings documented for future resolution

## Decisions Made

- **Used `impl Rng` not `impl RngCore` for generator functions** (diverges from plan's stated target signatures). Necessitated because `SliceRandom::shuffle()` in rand 0.9 requires `R: Rng`, which is a supertrait of `RngCore`. `random_index()` correctly uses `&mut impl RngCore` since it only calls `next_u64()`.
- **No `RngFailure` error variant added** per D-02's guidance and plan instructions. Using `rand::rng()` (thread-local CSPRNG) as default makes RNG construction infallible.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Generator functions need `impl Rng` not `impl RngCore`**
- **Found during:** Task 1 (RNG parameterization)
- **Issue:** Plan specified `impl RngCore` for all three function signatures, but `SliceRandom::shuffle()` in rand 0.9 requires `R: Rng + ?Sized`. `RngCore` alone does not satisfy this bound — `Rng` is a supertrait that extends `RngCore`.
- **Fix:** Used `impl Rng` for `generate_password()` and `calc_entropy()`, kept `impl RngCore` for `random_index()` (only calls `next_u64()`)
- **Files modified:** `src/lib.rs`
- **Verification:** `cargo build` exits 0, `random_index` uses `&mut impl RngCore`, generator functions use `impl Rng`
- **Committed in:** `af08ea8` (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Necessary fix — plan's signatures wouldn't compile with rand 0.9's `shuffle` trait bound. No scope creep.

## Issues Encountered

- No issues during planned work — both tasks executed cleanly with the Rule 1 deviation handled automatically.

## Pre-existing Issues (Deferred)

Two clippy warnings exist in the original codebase (not introduced by this plan):
- Unused variable `index` in `for (index, chars)` loop (src/lib.rs:83)
- `needless_lifetimes` on `enabled_groups` function (src/lib.rs:134)

These are logged in `deferred-items.md` for resolution in a later plan.

## Threat Model Compliance

The plan's threat register (T-01-01 through T-01-05) is addressed:
- T-01-01 (Information Disclosure / return): **mitigated** — `Zeroizing<String>` return type
- T-01-02 (Information Disclosure / errors): **mitigated** — error messages contain no secret material
- T-01-03 (DoS / rejection sampling): **accepted** — loop bounded by cryptographic probability
- T-01-04 (Tampering / config): **accepted** — `&Defaults` is immutable borrow
- T-01-05 (Information Disclosure / intermediate buffer): **mitigated** — `Zeroizing<Vec<char>>` extended to final `String`

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Library API hardened and ready for Phase 2 (CLI Core) wiring
- `generate_password()` returns `Result<Zeroizing<String>>` — CLI needs to handle `Deref<Target=String>` for `println!`/`format!`
- `rand::rng()` as default RNG source ready to be passed from CLI entry point
- No tests yet — Phase 1 Plan 02 (01-02-PLAN.md) covers comprehensive test suite

## Threat Flags

None — all modified files are within the scope of the plan's existing threat model. No new endpoints, auth paths, file access patterns, or trust boundary crossings introduced.

---

*Phase: 01-library-hardening*
*Completed: 2026-04-26*
