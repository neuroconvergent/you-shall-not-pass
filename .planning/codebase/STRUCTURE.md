# Codebase Structure

**Analysis Date:** 2026-04-26

## Directory Layout

```
[project-root]/
├── src/
│   ├── lib.rs          # Core library: password generation + entropy logic
│   └── main.rs         # CLI entry point (placeholder)
├── Cargo.toml          # Crate manifest + dependencies
├── Cargo.lock          # Dependency lockfile
├── README.md           # Project overview and build instructions
├── PLAN.md             # Roadmap, theory references, package notes
├── AGENTS.md           # Agent instructions and conventions
├── LICENSE.txt         # BSD-3-Clause-No-Military-License
└── .github/
    └── workflows/       # CI/CD definitions (if any)
```

## Directory Purposes

**`src/`:**
- Purpose: All Rust source code
- Contains: Library and binary entry points
- Key files: `src/lib.rs`, `src/main.rs`

**`.github/workflows/`:**
- Purpose: GitHub Actions CI configuration
- Contains: Workflow YAML files
- Key files: (none detected yet)

## Key File Locations

**Entry Points:**
- `src/main.rs`: CLI binary entry point (currently a placeholder)

**Configuration:**
- `Cargo.toml`: Crate manifest, dependencies, lint rules

**Core Logic:**
- `src/lib.rs`: `password_generator` module with `Defaults`, `PasswordError`, `generate_password`, `calc_entropy`

**Testing:**
- Not yet present. Add tests under `tests/` or inline `#[cfg(test)]` in `src/lib.rs`.

**Documentation:**
- `README.md`: User-facing overview
- `PLAN.md`: Detailed roadmap and theory references
- `AGENTS.md`: Developer/agent conventions and rules

## Naming Conventions

**Files:**
- Rust source files: `snake_case.rs` (e.g., `lib.rs`, `main.rs`)
- Planned modules: `config.rs`, `entropy.rs`, `clipboard.rs`

**Directories:**
- Source modules: `snake_case/` (e.g., `src/config/`, `src/entropy/`)
- Binary targets: `src/bin/` for additional executables

**Types / Structs / Enums:**
- `PascalCase` (e.g., `Defaults`, `PasswordError`)

**Functions / Variables:**
- `snake_case` (e.g., `generate_password`, `calc_entropy`)

**Constants:**
- `SCREAMING_SNAKE_CASE` (e.g., `DEFAULTS`)

## Where to Add New Code

**New Feature (e.g., passphrase generation):**
- Primary code: Add module to `src/` and declare in `src/lib.rs`
- Tests: `tests/passphrase_tests.rs` or inline `#[cfg(test)]` block

**New Component/Module:**
- Implementation: `src/{module_name}.rs`
- Re-export: Add `pub mod {module_name};` to `src/lib.rs`

**CLI Binary:**
- Replace `src/main.rs` with `clap`-based argument parsing
- Or create `src/bin/cli.rs` and update `Cargo.toml` `[[bin]]` section

**GUI Binary:**
- Implementation: `src/bin/gui.rs`
- Manifest entry: Add `[[bin]]` section to `Cargo.toml`

**Utilities:**
- Shared helpers: `src/utils.rs` or `src/common.rs`

**Configuration / Presets:**
- Parser: `src/config.rs`
- Default presets: embedded in `src/config.rs` or loaded from `~/.config/you-shall-not-pass/`

**Clipboard Integration:**
- GTK wrapper: `src/clipboard.rs`
- Cleanup commands: `src/clipboard.rs` (opt-in, documented)

## Special Directories

**`target/`:**
- Purpose: Cargo build artifacts
- Generated: Yes (by `cargo build`)
- Committed: No (in `.gitignore`)

**`.planning/`:**
- Purpose: GSD agent outputs (codebase maps, plans)
- Generated: Yes (by agent tools)
- Committed: Yes (intended for project documentation)

**`.opencode/`:**
- Purpose: Opencode agent framework files
- Generated: Yes (by opencode tooling)
- Committed: No (external tooling)

---

*Structure analysis: 2026-04-26*
