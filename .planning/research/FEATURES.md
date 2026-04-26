# Feature Research

**Domain:** Password Generator CLI
**Researched:** 2026-04-26
**Confidence:** HIGH

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| `-l/--length <N>` | User must control password length; every competitor has it | LOW | Already in core `Defaults.length`. Wire to clap arg with default from `DEFAULTS.length = 12`. |
| `--no-alpha/--no-caps/--no-numbers/--no-symbols` | Toggle charset groups on/off; policy compliance (e.g., legacy systems with no symbol support) | LOW | Already in core via `alphabet_flag`, `caps_alphabet_flag`, `num_flag`, `symbol_flag`. Default all on, let user disable. |
| `--help` with usage examples | CLI discoverability; clap generates auto-help, but users expect examples of common operations | LOW | clap `#[command(after_help = "...")]` with 2-3 examples (`ysnp`, `ysnp -l 24`, `ysnp --no-symbols`). |
| `--version` | Standard CLI behavior | LOW | clap `#[command(version)]` reads from `Cargo.toml`. |
| Entropy display alongside password | Users need to verify the password's strength; core differentiator but also table stakes for this project | LOW | Already in core via `calc_entropy()`. Wire to CLI output after generation. |
| Exit codes for errors | Essential for scripting; non-zero on failure, zero on success | LOW | `PasswordError` already exists. Map to exit code 1 for user errors (bad args), 2 for internal errors. |
| Single password on stdout (pipe mode) | For `ysnp | xclip` or `ysnp > /tmp/pass` scripting | LOW | Default behavior: print only password to stdout, write entropy/metrics to stderr. |

**Note:** Many of these are already implemented in the core library (`src/lib.rs`). The CLI phase is primarily wiring existing capabilities to `clap` arguments.

### Differentiators (Competitive Advantage)

Features that set the product apart. Not required, but valuable.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| `-q/--quantum` toggle with displayed quantum entropy | Only password generator showing quantum-adjusted (Grover) brute-force resistance; informs users about post-quantum threat model | LOW | Already in core (`quantum_check` flag, `/2` factor). Show both classical and quantum entropy: `Entropy: 80 bits (classical), 40 bits (Grover-adjusted)`. |
| `--avoid <CHARS>` / `-x <CHARS>` | Explicit character exclusion for sites with banned characters; combines with charset group toggles | LOW | Already in core (`chars_to_avoid` field, `build_avoid_set()`). Expose as comma-separated or raw string arg. |
| Entropy as structured output | Enables scripting: `ysnp --json` returns `{"password":"...","entropy_classical":80,"entropy_quantum":40}` for machine consumption | MEDIUM | Not in core yet. Requires `serde` serialization. But `serde` is already a declared dependency. |
| Verbose mode (`-v/--verbose`) showing group breakdown | Shows how many chars from each group were used, pool size, effective entropy — builds user trust and education | LOW-MEDIUM | Not in core yet. Requires minor instrumentation of `generate_password()` to track per-group character counts. |

**Prioritization:** The quantum entropy toggle is the primary differentiator and must ship in this phase. The `--avoid` flag is already working and costs nothing to expose. The JSON output and verbose mode are "nice to have differentiators" — useful but deferrable if complexity grows.

### Anti-Features (Deliberately Excluded from CLI Phase)

Features that seem good but create problems, or are out of scope for this milestone.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Multiple passwords (`--count N`) or column output | pwgen convention — "generate 20, pick one" | Increases scope: TTY detection (columns vs lines), screen clearing logic for shoulder-surfing prevention, output formatting complexity. Core lib generates one password per call. | Defer to later phase. User can run in a loop: `for i in $(seq 5); do ysnp; done`. |
| Clipboard integration (`--clip` like `pass -c`) | Convenience — one less step | Requires platform-specific clipboard handling (`xclip`, `wl-clipboard`, `cliphist`), cleanup command opt-in, secrets on clipboard are a security concern, Wayland-specific behavior | Defer to clipboard phase (planned via `src/clipboard/`). Document `ysnp \| wl-copy` in `--help` examples. |
| Pronounceable/phoneme-based passwords (`pwgen` style) | Easier to memorize | Entirely different generation algorithm (phoneme engine), fundamentally incompatible with `generate_password()` which selects random chars from pool. Massive scope increase. | Defer indefinitely unless validated user demand. The project's value prop is entropy-guaranteed passwords, not memorability. |
| Passphrase generation (XKCD-style words) | Trendy, user-friendly | Requires wordlist files, dictionary management, multi-language support, acrostic constraints, delimiter logic. Entirely separate generation path from `generate_password()`. Explicitly deferred per PLAN.md. | Deferred to later phase per plan. CLI is password-only for now. |
| Config file (`--config` flag or XDG auto-load) | Persistent preferences | Requires `serde` + `toml` parsing, XDG path resolution, merge logic (config file defaults + CLI overrides), error handling for malformed config. Explicitly deferred per PLAN.md. | Deferred to config phase. User can use shell aliases: `alias ysnpa="ysnp -l 24 -q"`. |
| `--target-entropy N` with auto-length | "Give me a password with 128 bits of entropy" | Requires loop in CLI (try lengths until entropy >= target), which is non-trivial and belongs in core library. Current core `Defaults.target_entropy` is a constant, not an auto-adjustment algorithm. | Defer until core library supports auto-sizing. Document manual calculation: `ysnp -l 20` gets you ~80 bits with standard charset. |
| Interactive mode (`-i` for password selection from list) | "Let me see options and pick one" | Requires TTY input handling, screen manipulation, conflicts with pipe mode. Adds complexity with no security benefit for this project's audience. | Defer indefinitely. Interactive UI belongs in GTK GUI, not CLI. |
| `--sha1 /path/to/file#seed` for reproducible passwords | pwgen legacy feature | Deterministic password generation from file contents is a terrible security pattern (file contents may be guessable, accidentally committed, or predictable). Violates the project's CSPRNG-only principle. | Never implement. Use a password manager instead. |
| `--alpha-only-characters` (for legacy systems) | Some ancient systems reject anything beyond a-z | Already covered by `--no-caps --no-numbers --no-symbols`. A separate "alpha-only" flag is redundant. | Use existing flags in combination. |
| `--random-delimiters` (xkcdpass style) | Variety in passphrase delimiters | Passphrase-only feature. CLI phase is password-only. | Deferred to passphrase phase. |

### Edge Cases the CLI Must Handle

These aren't features per se, but behaviors the CLI must get right.

| Edge Case | Expected Behavior | Implementation Note |
|-----------|-------------------|---------------------|
| Zero-length password (`-l 0`) | Exit with clear error message (`password length must be greater than zero`), exit code 1 | Already handled by `PasswordError::EmptyLength` in core |
| All charset groups disabled | Exit with clear error message (`no charset groups enabled`), exit code 1 | Already handled by `PasswordError::NoCharsetGroups` |
| `--avoid` removes **all** chars from a group | Exit with error indicating which group was exhausted | Already handled by `PasswordError::ExhaustedGroup` |
| Length < number of enabled groups | Exit with error (`length X is shorter than required minimum Y`), exit code 1 | Already handled by `PasswordError::LengthTooShort` |
| Pipe to another program | Password only on stdout, entropy/metrics on stderr | CLI must detect `is_terminal()` and adjust output. Single password, no TTY formatting. |
| TTY interactive use | Password + entropy display on stdout with formatting | Default: print password prominently, entropy line below it. No screen clearing. |
| Empty `--avoid` string (default) | No characters excluded | Core handles this (empty `HashSet`). No special CLI logic needed. |
| Unicode characters via `--utf8` | Include non-ASCII characters in output | Already in core (`utf8_flag`). Default off per `DEFAULTS`. Document that this may produce unprintable output in some terminals. |

## Feature Dependencies

```
[--json output]
    └──requires──> [serde::Serialize on output struct]

[--verbose output]
    └──requires──> [per-group char counting in generate_password()]

[All charset flags]
    └──requires──> [Defaults struct from core lib]

[Entropy display]
    └──requires──> [calc_entropy() from core lib]
    └──enhances──> [--quantum toggle] (shows both values)

[Password generation (all features)]
    └──requires──> [Defaults struct wired from clap args]

[Pipe mode detection]
    └──requires──> [is_terminal() call]
    └──conflicts──> (none, orthogonal to generation)

[--avoid flag]
    └──requires──> [chars_to_avoid field on Defaults]
```

### Dependency Notes

- **`--json` output requires `serde::Serialize`:** The output struct wrapping password + entropy values needs `#[derive(Serialize)]`. `serde` is already in `Cargo.toml` but unused. This is the only new code dependency for the JSON differentiator.
- **`--verbose` requires core library changes:** `generate_password()` currently doesn't track per-group character counts. Adding this is a small instrumentation change (a `Vec<usize>` counter pushed alongside picks).
- **Pipe mode detection is local to CLI:** `atty::is(atty::Stream::Stdout)` or Rust std's `std::io::IsTerminal` (stabilized in Rust 1.70) — no new dependency needed.
- **Charset flags map directly:** `--no-alpha` → `alphabet_flag = false`, etc. Zero translation logic, just `clap` derive defaults to `true`.

## MVP Definition

### Launch With (v1 — CLI Phase)

Minimum viable product — what's needed to validate the concept.

- [ ] **Password output on stdout** — The core deliverable. `generate_password()` wired to clap, password printed.
- [ ] **`-l/--length <N>` flag** — Length control (default 12).
- [ ] **`--no-alpha/--no-caps/--no-numbers/--no-symbols` flags** — Charset group toggles.
- [ ] **Entropy display** — Classical entropy alongside password (minimum). Quantum entropy when `-q/--quantum` is set.
- [ ] **`-q/--quantum` flag** — The primary differentiator. Must show both classical and Grover-adjusted entropy.
- [ ] **`--help` with examples** — Discoverability. At least 3 usage examples.
- [ ] **Proper exit codes** — 0 on success, 1 on bad args, 2 on internal error.
- [ ] **Pipe mode** — Password-only on stdout when piping, entropy on stderr.

### Add After Validation (v1.1+)

Features to add once core is working and feedback comes in.

- [ ] **`--avoid/-x <CHARS>` flag** — Character exclusion. Already in core, just wire it.
- [ ] **`--json` output flag** — Structured output for scripting consumers.
- [ ] **`--verbose/-v` flag** — Show group breakdown, pool size, per-group character counts.

### Future Consideration (v2+)

Features to defer until product-market fit is established.

- [ ] **Multiple passwords (`-c/--count`)** — Deferred to post-CLI validation.
- [ ] **Passphrase generation** — Separate subsystem per PLAN.md.
- [ ] **Config file support** — XDG dotfile parsing and CLI override merging.
- [ ] **Clipboard integration** — Platform-specific clipboard with cleanup commands.
- [ ] **Policy presets (NIST, corporate)** — Deferred to config phase.

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Password output | HIGH | LOW | P1 |
| `-l/--length` | HIGH | LOW | P1 |
| Charset group flags | HIGH | LOW | P1 |
| `-q/--quantum` toggle | HIGH (differentiator) | LOW | P1 |
| Classical entropy display | HIGH | LOW | P1 |
| Quantum entropy display | HIGH (differentiator) | LOW | P1 |
| `--help` with examples | MEDIUM | LOW | P1 |
| Exit codes | MEDIUM | LOW | P1 |
| Pipe mode (password on stdout, entropy on stderr) | MEDIUM | LOW | P1 |
| `--avoid/-x <CHARS>` | MEDIUM | LOW | P2 |
| `--json` output | MEDIUM | MEDIUM | P2 |
| `--verbose/-v` | LOW | LOW-MEDIUM | P2 |
| `--count/-c N` | LOW | MEDIUM | P3 |
| Passphrase generation | HIGH | HIGH | P3 (deferred) |
| Config file | MEDIUM | MEDIUM | P3 (deferred) |
| Clipboard integration | MEDIUM | HIGH | P3 (deferred) |

**Priority key:**
- P1: Must have for launch
- P2: Should have, add when possible
- P3: Nice to have, future consideration

## Competitor Feature Analysis

| Feature | pwgen | pass (password-store) | xkcdpass | Our Approach |
|---------|-------|----------------------|----------|--------------|
| Password length | `pwgen [length]` positional | `pass generate [name] [length]` | N/A (word count instead) | `-l/--length <N>` flag |
| Charset groups | `-0` (no numbers), `-A` (no caps), `-y` (symbols), `-n` (numerals) | `--no-symbols / -n` | N/A (words) | `--no-alpha/--no-caps/--no-numbers/--no-symbols` |
| Entropy display | None | None | `--verbose` (wordlist entropy only) | **Classical + quantum entropy always shown** — unique in market |
| Quantum modelling | None | None | None | **Grover-adjusted entropy (`/2`)** — unique |
| Multiple passwords | `pwgen [len] [num]` positional; columns/rows auto-detect | One per call | `-c/--count` | Deferred (P3) |
| Character exclusion | `-r <chars>` / `--remove-chars` | None | `-v/--valid-chars` with regex | `--avoid/-x <CHARS>` |
| Secure RNG | `-s/--secure` (uses /dev/urandom) | `/dev/urandom` internally | `cryptographically strong` by default | **Always OsRng CSPRNG** — no insecure RNG path exists |
| Pronounceable mode | Default mode, phoneme-based | None | None | **Not offered** — project focuses on entropy, not memorability |
| Passphrase/words | None | None | Default (word-based) | Deferred to later phase |
| Clipboard | None | `-c/--clip` | None | Deferred to later phase |
| Reproducible passwords | `-H/--sha1 file#seed` | None | None | **Never implement** — violates CSPRNG principle |
| Structured output | None | None | None (Python import API) | `--json` flag (P2 differentiator) |

### Key Competitive Insights

1. **No password generator displays entropy values to the user.** `pass` and `pwgen` give you a password and nothing else. `xkcdpass --verbose` shows wordlist entropy but not password-specific entropy. Our entropy display is a genuine differentiator.

2. **No password generator models quantum threats.** The Grover-adjusted entropy (`/2` factor) is unique in the market. This directly addresses the "steal-now-decrypt-later" threat model that motivates the project.

3. **Most generators offer both a "secure" and "pronounceable" mode.** By committing to CSPRNG-only entropy-maximizing passwords, we simplify the mental model for users who want security over memorability. This is a positioning choice, not a gap.

4. **pwgen is the most feature-complete password generator CLI** but hasn't been meaningfully updated since 2017 and uses non-cryptographic RNG by default (`rand()` unless `-s` flag). Our CSPRNG-by-default is a security improvement.

## Sources

- **pwgen man page** (v2.08, August 2017): Raw source from `tytso/pwgen` repository on GitHub — features inventoried from `pwgen.1` man page. Confidence: HIGH.
- **password-store** (`pass`): `zx2c4/password-store` repository and `passwordstore.org` documentation. Confidence: HIGH.
- **XKCD-password-generator** (`xkcdpass` v1.30.0): `redacted/XKCD-password-generator` repository, README, and CLI help output. Confidence: HIGH.
- **Core library analysis**: `src/lib.rs` (project source code), `.planning/codebase/CONCERNS.md`, `.planning/codebase/ARCHITECTURE.md`. Confidence: HIGH.
- **Project PLAN.md** and **PROJECT.md**: Out-of-scope items explicitly documented by project leadership. Confidence: HIGH.
- **crates.io**: Searched for existing Rust password generator CLIs (`password-generator`, `rpg-util`). None have quantum entropy modeling or significant adoption. Confidence: MEDIUM (limited search depth).

---

*Feature research for: Password Generator CLI*
*Researched: 2026-04-26*
