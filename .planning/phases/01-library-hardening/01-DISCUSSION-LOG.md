# Phase 1: Library Hardening - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-26
**Phase:** 01-library-hardening
**Areas discussed:** rand 0.9 API path

---

## rand 0.9 API path

| Option | Description | Selected |
|--------|-------------|----------|
| `rand::rng()` | Thread-local CSPRNG — seeds from OsRng, infallible, works with rand 0.9 with no TryRngCore ceremony | ✓ |
| `OsRng` with unwrap | Keep OsRng but unwrap/expect on construction — panics on failure, defeats SEC-02 | |

**User's choice:** `rand::rng()` — thread-local CSPRNG

**Follow-up decision:**

| Option | Description | Selected |
|--------|-------------|----------|
| Refactor to accept RNG param | `generate_password()` and `calc_entropy()` accept `impl RngCore` — testable, composable, panic-free by construction | ✓ |
| Add RngFailure variant anyway | Keep `PasswordError::RngFailure` as future-proof variant | |

**User's choice:** Refactor to accept `impl RngCore` parameter — callers inject RNG, tests use mock, production uses `rand::rng()`

**Notes:** This approach makes SEC-02 a non-issue for the default code path since `rand::rng()` is infallible. The parameterization provides the testability and future-proofing originally intended by SEC-02.

---

## the agent's Discretion

Areas not discussed — agent handles during planning/implementation:
- `Zeroizing<String>` display ergonomics (Deref/Target=String interaction with println/format)
- `PasswordError` variant design (whether to add RngFailure)
- Entropy validation alignment between `generate_password()` and `calc_entropy()`
- Test strategy for RNG parameterization
- rand 0.9 API migration specifics (ThreadRng usage, OsRng removal)

## Deferred Ideas

None.
