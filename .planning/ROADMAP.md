# Roadmap: You Shall Not Pass

## Overview

A Rust password/passphrase generator with quantum brute-force modelling. v1 delivers a minimal CLI wrapping the existing core library — users generate passwords from the command line with classical and quantum-adjusted entropy display, pipe-safe output, and production security hardening. The CLI is a thin presentation layer; all generation logic stays in `src/lib.rs`.

## Phases

- [ ] **Phase 1: Library Hardening** — Secure the core library API before CLI wiring begins
- [ ] **Phase 2: CLI Core** — Working password generator CLI with entropy display
- [ ] **Phase 3: Output & Security Hardening** — Pipe-safe output, core dump prevention, panic sanitization

## Phase Details

### Phase 1: Library Hardening
**Goal**: Library API is secure by construction — no secrets leak through return types, no panics on system failures. All downstream CLI work depends on this foundation.
**Depends on**: Nothing (first phase)
**Requirements**: SEC-01, SEC-02
**Success Criteria** (what must be TRUE):
  1. `generate_password()` returns `Result<Zeroizing<String>>` — the password is wrapped in a zeroize-on-drop container and callers cannot obtain a plain `String` without explicit dereference.
  2. When `OsRng` is unavailable (early boot, restricted container), `generate_password()` returns `Err(PasswordError::RngFailure)` instead of panicking.
  3. `calc_entropy()` validates `length >= groups.len()` before computing entropy — matching the validation already present in `generate_password()`.
  4. All existing library tests pass with the updated signatures; new tests cover the `OsRng` failure path.
**Plans**: 2 plans

Plans:
- [x] 01-01-PLAN.md — Fix rand 0.9 compilation, parameterize RNG, wrap password in Zeroizing, align calc_entropy validation
- [ ] 01-02-PLAN.md — Create comprehensive test suite covering happy paths, error cases, entropy validation, and Zeroizing return

### Phase 2: CLI Core
**Goal**: User can generate passwords from the command line with full control over length and character sets, see classical and quantum entropy, and get help. The CLI is a working tool.
**Depends on**: Phase 1
**Requirements**: GEN-01, GEN-02, GEN-03, GEN-04, GEN-05, GEN-06, GEN-07, SEC-06
**Success Criteria** (what must be TRUE):
  1. Running `ysnp` with no arguments generates a 12-character password from the default charset and prints it to stdout.
  2. Running `ysnp -l 24 --no-symbols` generates a 24-character password without symbol characters.
  3. Running `ysnp -q` prints both classical and quantum-adjusted entropy alongside the generated password.
  4. Running `ysnp --help` displays usage information with examples; `ysnp --version` prints the crate version.
  5. Invalid arguments (e.g., `ysnp -l 0` or disabling all charset groups) produce a clear error message on stderr and exit code 1.
  6. The CLI has no flags that accept secret material — no `--seed`, `--key`, or `--password` arguments exist.
**Plans**: TBD

### Phase 3: Output & Security Hardening
**Goal**: CLI is safe for production use — pipe mode works correctly, core dumps are disabled, panic messages never leak secrets.
**Depends on**: Phase 2
**Requirements**: GEN-08, SEC-03, SEC-04, SEC-05
**Success Criteria** (what must be TRUE):
  1. When piped to another program (`ysnp | cat`), only the raw password appears on stdout; all diagnostics (entropy, warnings) go to stderr.
  2. In an interactive terminal (TTY), the password is displayed on its own line with formatted entropy values below it.
  3. If the process crashes (SIGSEGV), no core dump is produced — `RLIMIT_CORE=0` is enforced at startup.
  4. If a panic occurs, the panic message does not expose any secret material — password buffers are sanitized before display.
**Plans**: TBD

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Library Hardening | 1/2 | In progress | 01-01: 2026-04-26 |
| 2. CLI Core | 0/TBD | Not started | - |
| 3. Output & Security Hardening | 0/TBD | Not started | - |
