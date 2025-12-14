# AGENTS.md

## Purpose
This repository implements **You Shall Not Pass**, a Rust-based password and
passphrase generator with quantum brute-force modelling, delivered as a CLI
and GTK application.

Agents should prioritize:
- correctness
- security hygiene
- minimal dependencies
- clear separation between core logic, CLI, and GUI

---

## Build / Run / Test

- Build: `cargo build`
- Run: `cargo run`
- Test (all): `cargo test`
- Test (single): `cargo test <name>`
- Lint: `cargo clippy --all-targets --all-features`
- Format: `cargo fmt`

Assume Linux + Wayland unless stated otherwise.

---

## Project Structure (expected)

- `src/lib.rs` — core generation + entropy logic (no UI, no IO side effects)
- `src/bin/cli.rs` — CLI frontend
- `src/bin/gui.rs` — GTK frontend
- `src/config/` — config parsing and defaults
- `src/entropy/` — entropy + quantum modelling
- `src/clipboard/` — clipboard + cleanup commands
- `tests/` — unit and property tests

Do not mix UI code with core logic.

---

## Code Style & Conventions

- Rust edition: **2024**
- Formatting: `rustfmt`
- Linting: `clippy` (warnings treated seriously)

### Naming
- Functions / variables: `snake_case`
- Types / structs / enums: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`

### Imports
Order strictly as:
1. `std`
2. external crates
3. local modules

---

## Error Handling

- Use `Result<T, E>` and the `?` operator
- Prefer `thiserror` / `anyhow` for user-facing errors
- No `unwrap()` or `expect()` in production paths
- Panics allowed only in tests

---

## Security Rules (Critical)

- Never log secrets
- Never persist generated passwords unless explicitly requested
- Zeroize secret buffers after use (`zeroize`)
- Clipboard cleanup commands must be **opt-in**
- Shell execution must be explicit and documented
- Treat config files as untrusted input

---

## Cryptography Scope

- Use OS-backed CSPRNG only
- Do NOT implement cryptographic primitives
- Do NOT use post-quantum crypto libraries
- “Quantum-safe” refers ONLY to entropy modelling (Grover-adjusted brute force)

---

## GTK / UI Rules

- GTK is a thin wrapper over core logic
- No entropy math or RNG in UI code
- UI must remain responsive (avoid blocking calls)
- Clipboard access via GTK APIs only

---

## Testing Expectations

- Deterministic tests where possible
- Property tests for entropy calculations
- Fuzz config parsing and input handling
- Validate entropy math against known values

---

## Documentation Rules

- Prefer code clarity over comments
- Comment only non-obvious logic
- Keep README and docs technically precise
- Avoid marketing language

---

## Agent Guidance

When modifying code:
1. Reuse existing patterns
2. Minimize new abstractions
3. Avoid speculative features
4. Prefer explicitness over cleverness
5. Keep token usage low and diffs small

If requirements are unclear, infer conservatively and document assumptions.
