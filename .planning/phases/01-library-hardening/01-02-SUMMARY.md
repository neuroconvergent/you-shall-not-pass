---
phase: 01-library-hardening
plan: 02
subsystem: testing
tags: [rust, testing, password-generator, entropy, zeroize]

# Dependency graph
requires:
  - phase: 01-01
    provides: hardened library API with RNG parameterization, Zeroizing return, aligned calc_entropy validation
provides:
  - Comprehensive integration test suite for password_generator module
  - Happy-path coverage for password generation and entropy calculation
  - Error-path coverage for all 3 primary PasswordError variants
  - Entropy validation alignment proof between generate_password and calc_entropy
  - Verification that Zeroizing<String> return type works correctly via Deref
affects: [01-03, Phase 2 CLI, all future development requiring test safety net]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Integration tests in tests/lib_tests.rs using rand::rng() as RNG source
    - Structural password validation (length, charset membership) rather than exact output
    - Separate test sections for happy-path vs error-path tests
    - Explicit PasswordError variant matching with matches! macro

key-files:
  created:
    - tests/lib_tests.rs
  modified: []

key-decisions:
  - "14 integration tests in a single flat file (tests/lib_tests.rs) — no sub-modules needed yet"
  - "rand::rng() used as RNG source for all tests — consistent with D-02 and infallible ThreadRng path"
  - "Struct update syntax ..DEFAULTS used for config construction — cleaner than DEFAULTS.clone()"
  - "Entropy validation tests use quantum_check: false to avoid halving factor in comparisons"
  - "Error-path tests are config-driven (no RNG involvement), making them deterministic"

patterns-established:
  - "Tests use rand::rng() instead of seeded/mock RNG — structural assertions only"
  - "Password verification checks length and charset membership, never exact string value"
  - "Error-path tests use matches! macro for PasswordError variant assertion"
  - "Defaults constructed via struct update syntax for readability"

requirements-completed: [SEC-01, SEC-02]
duration: 5min
completed: 2026-04-26
---

# Phase 1 Plan 02: Test Suite for Hardened Library

**14 integration tests covering happy paths, all 3 primary error variants, and entropy validation across generate_password and calc_entropy**

## Performance

- **Duration:** 5 min
- **Started:** 2026-04-26T12:33:00Z
- **Completed:** 2026-04-26T12:38:00Z
- **Tasks:** 2
- **Files modified:** 1 (created tests/lib_tests.rs, 209 lines)

## Accomplishments

- Created `tests/lib_tests.rs` with 14 integration tests for the password_generator module
- Happy-path tests cover: default length, custom length, no-symbols filter, classical entropy, quantum entropy, Zeroizing return type
- Error-path tests cover: EmptyLength, NoCharsetGroups, LengthTooShort — all tested for both `generate_password` and `calc_entropy`
- Entropy validation alignment proven: both `generate_password` and `calc_entropy` now reject `length < groups.len()` with identical `LengthTooShort` errors
- Entropy monotonicity verified: entropy increases with length and decreases with fewer charset groups
- `Zeroizing<String>` return type verified via `Deref<Target = String>` with compile-time type assertion

## Task Commits

Each task was committed atomically:

1. **Task 1: Create test scaffold and happy-path tests** - `7d33520` (test)
2. **Task 2: Add error-path and entropy validation alignment tests** - `6802f6a` (test)

**Plan metadata:** (committed in final step below)

## Files Created/Modified

- `tests/lib_tests.rs` - 14 integration tests for the password_generator module (209 lines)

## Decisions Made

- **Single flat test file**: Kept all 14 tests in `tests/lib_tests.rs` without sub-modules. The tests are compact enough that sub-module structure would add overhead without benefit. If the suite grows significantly in a later phase, it can be split.
- **rand::rng() for all tests**: Consistent with D-02 (RNG parameterization). ThreadRng is infallible in practice, and error-path tests are config-driven (not RNG-driven), so deterministic assertions work naturally.
- **Struct update syntax**: `..DEFAULTS` is used for config construction instead of manual struct literals. This is cleaner and avoids copying all 10 fields while making only the changed fields explicit.
- **quantum_check: false in entropy comparison tests**: Avoids the halving factor that would otherwise apply to both sides of the comparison, making the monotonicity assertions simpler to reason about.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## Threat Compliance

- T-01-06 (Information Disclosure via test output): Tests assert on length and charset membership only. No `println!` of generated passwords.
- T-01-07 (Hardcoded passwords): All tests use `rand::rng()` for password generation. No hardcoded secret material.

## Threat Flags

None — no new security-relevant surface introduced (test file only).

## Next Phase Readiness

- Test suite provides safety net for all future modifications to password_generator module
- Phase 2 (CLI Core) can proceed with confidence that library behavior is verified
- `ExhaustedGroup` and `EmptyCharacterPool` error variants remain untested — deferred (per plan's explicit exclusion)

## Self-Check: PASSED

- [x] `tests/lib_tests.rs` exists (209 lines)
- [x] `01-02-SUMMARY.md` exists
- [x] Commit `7d33520` (Task 1) verified in git log
- [x] Commit `6802f6a` (Task 2) verified in git log
- [x] `cargo test` — 14/14 pass, 0 failures
- [x] `cargo clippy --all-targets --all-features` — exits 0 (2 pre-existing acceptable warnings)
- [x] `cargo build` — exits 0
- [x] `grep -cF '#[test]' tests/lib_tests.rs` — 14 (exceeds minimum of 10)
- [x] `grep -c 'PasswordError::' tests/lib_tests.rs` — 6 (covers EmptyLength, NoCharsetGroups, LengthTooShort)
- [x] `rand::rng()` present in test file — verified in all 14 tests

---

*Phase: 01-library-hardening*
*Completed: 2026-04-26*
