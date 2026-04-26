# Requirements: You Shall Not Pass

**Defined:** 2026-04-26
**Core Value:** Generate quantum-resistant passwords with transparent entropy modelling

## v1 Requirements

Requirements for CLI phase — minimal working password generator wrapping the core library.

### Generation

- [ ] **GEN-01**: User can generate a password via CLI and see it on stdout
- [ ] **GEN-02**: User can set password length with `-l/--length <N>` (default 12)
- [ ] **GEN-03**: User can toggle charset groups with `--no-alpha`, `--no-caps`, `--no-numbers`, `--no-symbols`
- [ ] **GEN-04**: CLI displays classical entropy alongside generated password
- [ ] **GEN-05**: User can enable quantum entropy display with `-q/--quantum` flag
- [ ] **GEN-06**: CLI provides `--help` with usage examples and `--version`
- [ ] **GEN-07**: CLI returns exit code 0 on success, 1 on bad args, 2 on internal error
- [ ] **GEN-08**: When piped to another program, password goes to stdout and entropy/metrics to stderr

### Security

- [ ] **SEC-01**: Library returns `Zeroizing<String>` from `generate_password()` instead of plain `String`
- [ ] **SEC-02**: CLI handles `OsRng` failure with graceful error message (no panic)
- [ ] **SEC-03**: CLI detects TTY vs pipe and routes password to stdout, entropy to stderr
- [ ] **SEC-04**: CLI sets `RLIMIT_CORE=0` at startup to prevent core dumps containing secrets
- [ ] **SEC-05**: CLI installs panic handler that sanitizes output (no secret leakage via Debug backtraces)
- [ ] **SEC-06**: CLI never accepts secrets or passwords as command-line arguments

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Generation

- **GEN-09**: `--avoid/-x <CHARS>` flag for character exclusion
- **GEN-10**: `--json` flag for structured machine-readable output
- **GEN-11**: `--verbose/-v` flag showing group breakdown and pool size

## Out of Scope

| Feature | Reason |
|---------|--------|
| Multiple passwords (`--count`) | Deferred — user can loop `ysnp` in shell |
| Clipboard integration | Deferred to dedicated clipboard phase |
| Config file / XDG dotfile | Deferred to config phase |
| Passphrase generation | Deferred to later phase per PLAN.md |
| Pronounceable/passphrase modes | Never — CSPRNG-only entropy-maximizing |
| Reproducible passwords (`--sha1`) | Never — violates CSPRNG principle |
| GTK GUI | Deferred to later phase |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| GEN-01 | Phase 2 | Pending |
| GEN-02 | Phase 2 | Pending |
| GEN-03 | Phase 2 | Pending |
| GEN-04 | Phase 2 | Pending |
| GEN-05 | Phase 2 | Pending |
| GEN-06 | Phase 2 | Pending |
| GEN-07 | Phase 2 | Pending |
| GEN-08 | Phase 3 | Pending |
| SEC-01 | Phase 1 | Pending |
| SEC-02 | Phase 1 | Pending |
| SEC-03 | Phase 3 | Pending |
| SEC-04 | Phase 3 | Pending |
| SEC-05 | Phase 3 | Pending |
| SEC-06 | Phase 2 | Pending |

**Coverage:**
- v1 requirements: 14 total
- Mapped to phases: 14 ✓
- Unmapped: 0


---
*Requirements defined: 2026-04-26*
*Last updated: 2026-04-26 after initial definition*
