# Technology Stack

**Analysis Date:** 2026-04-26

## Languages

**Primary:**
- Rust (Edition 2024) — sole implementation language across the entire codebase

**Secondary:**
- Shell (POSIX sh) — `addlicense.sh` license header automation script only

## Runtime

**Environment:**
- Native Rust binary (no VM or managed runtime)
- Minimum supported toolchain: Rust 1.91.1+ (2024 edition)

**Package Manager:**
- Cargo (bundled with Rust toolchain)
- Lockfile: `Cargo.lock` present (version 4 format)

## Frameworks

**Core:**
- `gtk4` 0.10.3 — GTK 4 bindings via gtk-rs for the planned GUI frontend
- `clap` 4.5.53 — Command-line argument parsing for the CLI frontend
- `rand` 0.9.2 — Cryptographically secure random number generation (OS-backed CSPRNG)

**Serialization & Configuration:**
- `serde` 1.0.228 — Data serialization framework
- `toml` 0.9.8 — TOML configuration file parsing

**Error Handling:**
- `thiserror` 2.0.17 — Structured, idiomatic error types with `#[derive(Error)]`

**Security & Memory Hygiene:**
- `zeroize` 1.8.2 — Securely clears sensitive buffers from memory after use

**System Integration:**
- `nix` 0.30.1 — Unix-specific APIs (secure file handling, keyfile generation)

**Unicode:**
- `unicode-segmentation` 1.12.0 — Grapheme-aware text processing for password generation

**Build/Dev:**
- Cargo — build, test, and package management
- `rustfmt` — code formatting (standard Rust toolchain component)
- `clippy` — linting (warnings treated seriously per project conventions)

## Key Dependencies

**Critical (direct dependencies from `Cargo.toml`):**

| Package | Version | Purpose |
|---------|---------|---------|
| `clap` | 4.5.53 | CLI argument parsing and help generation |
| `gtk4` | 0.10.3 | GTK 4 GUI bindings (planned interactive frontend) |
| `nix` | 0.30.1 | Unix system calls and secure file operations |
| `rand` | 0.9.2 | OS-backed CSPRNG (`OsRng`) for password generation |
| `serde` | 1.0.228 | Serialization for XDG-compliant configuration files |
| `thiserror` | 2.0.17 | Derive-macro error types with `#[error("...")]` messages |
| `toml` | 0.9.8 | Parse dotfile-based TOML configuration |
| `unicode-segmentation` | 1.12.0 | Grapheme cluster awareness for character handling |
| `zeroize` | 1.8.2 | Secure memory zeroization of password buffers (`Zeroizing<T>`) |

**Notable transitive dependencies:**
- `getrandom` 0.3.4 — Platform entropy backend for `rand::OsRng`
- `libc` 0.2.178 — Low-level C library bindings (GTK/sys deps)
- `futures-*` 0.3.31 — Async runtime support (pulled in by gtk-rs)
- `cairo-rs`, `gdk4`, `gsk4`, `gio`, `glib`, `pango` 0.21.5 — GTK ecosystem sys crates

## Configuration

**Build & Lint:**
- `Cargo.toml` lint configuration: `unsafe_code = "forbid"` (global deny of `unsafe` blocks)
- No custom `rustfmt.toml` or `clippy.toml` detected; using defaults

**Runtime Configuration (planned, not yet implemented):**
- XDG-compliant dotfile configuration (`~/.config/you-shall-not-pass/config.toml`)
- Policy presets for NIST-style entropy targets, corporate rules, and archival secrets

**Environment:**
- No `.env` file or environment variable parsing detected in current source
- `OPENCODE_API_KEY` used only in CI (GitHub Actions secrets)

## Platform Requirements

**Development:**
- Rust toolchain supporting Edition 2024 (verified: `rustc 1.91.1` / `cargo 1.91.1`)
- GTK 4 development libraries and headers (for GUI builds)
- Linux / Unix-like environment (Wayland or X11)

**Production:**
- Target: Linux desktop (CLI and GTK GUI binaries)
- No server deployment or containerization currently configured
- `nix` crate limits portability to Unix-like systems; Windows builds would require conditional compilation

---

*Stack analysis: 2026-04-26*
