# You Shall Not Pass

## What This Is

A Rust-based password and passphrase generator with quantum brute-force modelling.
Core library generates high-entropy secrets using OS-backed CSPRNG, with entropy
metrics displayed under both classical and quantum-adjusted (Grover) threat models.
Delivered as a CLI and GTK GUI.

## Core Value

Generate quantum-resistant passwords and passphrases for long-lived secrets
that could be targeted by steal-now-decrypt-later attacks — and show the user
exactly why they're secure.

## Requirements

### Validated

- ✓ Password generation with OS-backed CSPRNG (`OsRng`) — existing
- ✓ Classical entropy calculation (`log2(pool_size) * length`) — existing
- ✓ Quantum-adjusted entropy (Grover `/2` factor) — existing
- ✓ Character set filtering (enabled groups, avoid chars, Unicode gate) — existing
- ✓ Unbiased random index via rejection sampling — existing
- ✓ Memory hygiene with `Zeroizing` for password buffers — existing
- ✓ Structured error handling via `thiserror` — existing

### Active

- [ ] CLI with `clap` flags for length, charset groups, and quantum mode
- [ ] CLI prints generated password and its classical + quantum entropy
- [ ] Configuration: `Defaults` struct configurable via CLI flags
- [ ] CLI wired to `generate_password()` and `calc_entropy()` in core library

### Out of Scope

- GUI / GTK frontend — deferred to later phase
- Passphrase generation — deferred to later phase
- Dotfile-based configuration (XDG) — deferred to later phase
- Clipboard integration and cleanup — deferred to later phase
- Policy presets (NIST, corporate, archival) — deferred to later phase
- Packaging / deployment — deferred to later phase

## Context

- Existing codebase with working core library (`src/lib.rs`) containing
  `password_generator` module with `Defaults`, `PasswordError`, `generate_password`,
  `calc_entropy`
- `Cargo.toml` dependencies configured for the full plan (`clap`, `gtk4`, `rand`,
  `serde`, `toml`, `thiserror`, `zeroize`, `nix`, `unicode-segmentation`)
- `main.rs` currently a placeholder (`Hello, world!`)
- Project follows AGENTS.md conventions: Rust edition 2024, no unsafe code,
  `thiserror` for errors, `Zeroizing` for secrets, no mixing UI and core logic
- PLAN.md from December 2025 defines the full 7-phase vision; current focus is
  Phase 4 (CLI Development) as a minimal increment

## Constraints

- **Tech stack**: Rust (edition 2024), `clap` for CLI, `rand::OsRng` for CSPRNG
- **Platform**: Linux / Wayland (primary target)
- **Security**: No unsafe code (`unsafe_code = "forbid"`), secrets zeroized after use,
  no logging of secrets, no persistent secret storage
- **Dependencies**: Keep minimal — no new crates unless needed by plan
- **Architecture**: Core logic stays in `lib.rs` (or split modules under `src/`);
  CLI in `main.rs` is a thin wrapper over library calls

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Minimal CLI first | User wants a working tool quickly; GUI, config, presets add complexity without immediate value | — Pending |
| CLI uses `clap` for args | Already in Cargo.toml; standard Rust CLI ecosystem choice | — Pending |
| Core lib stays in `lib.rs` for now | Only one module exists; splitting into submodules adds overhead without benefit at current scale | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-26 after initialization*
