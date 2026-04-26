# Pitfalls Research

**Domain:** Rust CLI password/passphrase generator with quantum entropy modelling
**Researched:** 2026-04-26
**Confidence:** HIGH (verified against official docs, Context7, and real-world post-mortems)

## Critical Pitfalls

### Pitfall 1: Final password `String` is not zeroized on drop

**What goes wrong:**
`generate_password()` constructs the password inside a `Zeroizing<Vec<char>>` buffer, but then collects it into a plain `String` and returns it to the caller (currently `lib.rs:96`). The `Zeroizing` wrapper protects only the intermediate `Vec<char>`. The returned `String` lives on the heap and will not be zeroized when it goes out of scope, leaving password material in freed heap pages indefinitely.

**Why it happens:**
`Zeroizing<String>` does implement `Zeroize` (clearing to empty on drop) and `ZeroizeOnDrop`, but the function signature returns `String`. Developers focus on the internal construction hygiene and overlook the return path. The `String` API (`collect()`, `format!()`, `to_string()`) naturally copies the secret into a new allocation outside the `Zeroizing` boundary.

**How to avoid:**
Change `generate_password()` signature to return `Zeroizing<String>` instead of `String`. The `Zeroizing` wrapper will automatically zeroize on drop. All CLI code that reads the password must do so through the `Zeroizing` wrapper (via `Deref` to `&str` for display) and drop it as soon as possible.

```rust
// Current (dangerous):
pub fn generate_password(config: &Defaults) -> Result<String>

// Recommended:
pub fn generate_password(config: &Defaults) -> Result<Zeroizing<String>>
```

**Warning signs:**
- `generate_password()` return type is `String` (not `Zeroizing<String>`)
- The password is printed, formatted, or passed by value after being unwrapped from `Zeroizing`
- No audit of all paths the password `String` takes after leaving `generate_password()`

**Phase to address:**
Phase 4 (CLI Development) — this change in the library signature directly affects the CLI wiring.

---

### Pitfall 2: TTY detection — printing secrets when stdout is a pipe

**What goes wrong:**
The CLI prints the generated password to stdout by default. When stdout is a pipe (e.g., `you-shall-not-pass | xclip`), the password is written to another program or file. This is sometimes intentional (scripting), but is catastrophic when the user expects the TTY-only screen-clearing behavior of tools like `pwgen`. A password written to a pipe can end up in shell history, log files, or on disk.

**Why it happens:**
CLI tools that generate secrets often default to "print to stdout" without checking whether stdout is a TTY. The `std::io::IsTerminal` trait (stable since Rust 1.70) exists for this purpose but is easily overlooked. The `pwgen` man page explicitly describes this distinction — interactive mode prints a screenful then clears, pipe mode prints one password. However, `pwgen` also warns that its `-H` flag can leak secrets via shell history files (`~/.bash_history`).

**How to avoid:**
Use `std::io::stdout().is_terminal()` to detect whether output is going to a human or a machine. When stdout is a TTY:
- Print the password with a prominent label and entropy information
- Consider offering to clear the screen after a timeout or on Ctrl+C
- Emit entropy information to stderr (not stdout)

When stdout is NOT a TTY (pipe/file):
- Print ONLY the raw password to stdout (for `$()` or pipe consumption)
- Emit ALL diagnostic/entropy information to stderr
- Warn on stderr that the password is being piped
- Consider requiring an explicit `--force-pipe` flag for safety

**Warning signs:**
- `println!()` used for both passwords and metadata without `IsTerminal` checks
- Entropy stats and password printed to the same stream
- No distinction between interactive and scripted output modes

**Phase to address:**
Phase 4 (CLI Development) — this is a pure CLI concern.

---

### Pitfall 3: RNG panics on failure — no graceful error handling

**What goes wrong:**
With `rand` 0.9+, `OsRng` implements `TryRngCore` (not `RngCore` directly as in 0.8). If the OS CSPRNG fails (e.g., early boot before `/dev/urandom` is seeded, restricted container with no `getrandom` syscall access, or `/proc/sys/kernel/random/entropy_avail` exhaustion on older kernels), the RNG will not produce values. If the code uses `unwrap()` or implicitly panics via methods that require `RngCore`, the CLI crashes with an unhelpful panic message instead of returning a structured error.

**Why it happens:**
The current code calls `rng.next_u64()` and passes `&mut rng` to `SliceRandom::shuffle`, both of which require `RngCore`. In `rand` 0.9, `OsRng` only provides `RngCore` via a `DerefMut` to `UnwrapMut<OsRng>`, which panics on failure. The `CONCERNS.md` already flags this — the code doesn't compile against `rand` 0.9. After fixing compilation, the runtime failure path remains unhandled.

**How to avoid:**
1. Fix compilation first: use `OsRng.unwrap_mut()` or `rand::rng()` (thread-local RNG which implements `RngCore`), or downgrade to `rand = "0.8"` if that simplifies the API surface.
2. After compilation: make `OsRng` fallible. Return a `PasswordError::RngFailure` variant from `generate_password()` and `calc_entropy()`.
3. The CLI should handle `PasswordError::RngFailure` by printing a clear error to stderr and exiting with a distinct non-zero exit code (e.g., `exitcode::OSERR` or `exitcode::UNAVAILABLE`).

**Warning signs:**
- `unwrap()` or `expect()` anywhere in the RNG call chain
- No `PasswordError` variant for RNG failure
- `OsRng` instantiated and used inline without error propagation path
- `rand` 0.9 API migration done incompletely (code does not compile)

**Phase to address:**
Phase 4 (CLI Development) — must fix compilation before CLI can be wired.

---

### Pitfall 4: Shell history leakage via command-line arguments

**What goes wrong:**
If the CLI accepts secrets (passphrase for entropy seeding, key material, custom charset) as command-line arguments, those values are stored in the shell history file (`~/.bash_history`, `~/.zsh_history`, `~/.local/share/fish/fish_history`, etc.). Any user or process with filesystem access can recover those secrets. The `pwgen` man page explicitly warns about this for its `-H` (sha1 seeding) option.

**Why it happens:**
Command-line arguments are not private — they appear in `/proc/<pid>/cmdline`, `ps aux` output, and shell history. Developers accustomed to "args are inputs" forget that args have different threat models than stdin or config files.

**How to avoid:**
- **Never accept secrets via positional arguments or `--long` flags.** Secrets must come from stdin, environment variables, or files with restricted permissions.
- For the current phase, this means: the CLI takes only *non-secret* parameters (length, charset flags, quantum mode) as clap arguments. No passphrase input, no key material, no custom charset strings.
- If passphrase/seed input is added later, use `rpassword` or `terminal` crate for password prompts (disable echo), or read from a pipe.
- Document in `--help` that no flag accepts secret material.

**Warning signs:**
- Any `--password`, `--seed`, `--key`, or `--charset` clap argument
- Arguments that accept arbitrary strings with no length/structure validation
- `ps aux` showing password material during generation

**Phase to address:**
Phase 4 (CLI Development) — enshrined at CLI design time. Later phases (passphrase generation, seed-based generation) must inherit this constraint.

---

### Pitfall 5: Panic messages and backtraces exposing secrets

**What goes wrong:**
Rust's default panic handler prints the panic message and optionally a full stack backtrace to stderr. If a panic occurs while a password or seed material is in scope — particularly if the secret is part of a `Debug` or `Display` implementation — those secrets leak into stderr, which may be logged, captured, or redirected to files.

**Why it happens:**
The `Debug` derive on types holding secrets (even indirectly) can format the secret into the panic message. The `Zeroizing` wrapper's `Debug` implementation (from docs.rs) formats the inner value — meaning `Zeroizing<String>` in a `Debug`-derived struct **will print the password** in a panic backtrace. Even if the CLI exits normally, panics in library code (e.g., from `unwrap()`) will dump state.

**How to avoid:**
1. **Install a custom panic handler** (e.g., `human-panic` crate) that suppresses detailed backtraces from end users. The `human-panic` crate generates a sanitized report file instead of dumping raw state.
2. **Do NOT derive `Debug` on any struct that transitively contains a `Zeroizing<T>` or password material.** Instead, implement `Debug` manually to print `[REDACTED]` or `<secret>`.
3. **Never `unwrap()` or `expect()` in production paths** — use `Result` and `?` to propagate errors without panicking. The AGENTS.md already mandates this.
4. Audit all `#[derive(Debug)]` on structs in the CLI argument parser and configuration types — ensure none of them transitively contain secret types.

**Warning signs:**
- `#[derive(Debug)]` on any type containing `Zeroizing<T>`, `String` (when used for passwords), or `Vec<u8>` (when used for keys)
- Any `unwrap()`, `expect()`, or `panic!()` call outside of `#[cfg(test)]`
- No custom panic handler set up in `main()`

**Phase to address:**
Phase 4 (CLI Development) — panic handler setup and Debug audit happen during CLI wiring.

---

### Pitfall 6: Entropy calculation inconsistency with generation

**What goes wrong:**
`calc_entropy()` uses `log2(pool_size) * length` to compute entropy, treating each password position as an independent draw from the full character pool. However, `generate_password()` guarantees at least one character from each enabled group (one-per-group constraint). This means:
1. The entropy formula is slightly optimistic (overestimates entropy for short passwords where the one-per-group constraint significantly reduces combinatorial space).
2. `calc_entropy()` allows configurations that `generate_password()` rejects — specifically, `length < groups.len()`. Calling entropy calc with `length=1` and all four groups enabled returns a misleadingly high entropy value for a configuration that cannot actually generate a password.

**Why it happens:**
The entropy function was implemented independently from the generation function, without cross-validating constraints. The generation function validates `config.length >= groups.len()` (line 66), but `calc_entropy` does not (line 102). This is already flagged in `CONCERNS.md`.

**How to avoid:**
1. Add the same length-vs-groups validation to `calc_entropy()` that exists in `generate_password()`.
2. Document that the entropy formula is a conservative lower bound that does NOT account for the one-per-group guarantee. The actual entropy is slightly higher due to guaranteed charset diversity.
3. In a later phase, adjust the entropy model to include the combinatorial benefit of the guaranteed-character constraint: `entropy = log2(group1) + log2(group2) + ... + log2(pool) * (length - num_groups)`, accounting for both guaranteed characters and remaining free draws.

**Warning signs:**
- `calc_entropy()` and `generate_password()` have different input validation
- Entropy output for valid configs doesn't match hand-calculated reference values
- The CLI prints entropy values that don't correspond to the actual password generation space

**Phase to address:**
Phase 4 (CLI Development) — fix the validation mismatch immediately. The entropy model refinement can be deferred to a later phase (Phase 5 or beyond).

---

### Pitfall 7: Core dump leaks secrets to disk

**What goes wrong:**
On Linux, when a process crashes with SIGSEGV, SIGABRT, or certain other signals, the kernel may write a core dump to disk (typically `/var/lib/systemd/coredump/` on systemd systems, or `core` in the working directory). This core dump contains the full process memory, including any password buffers that had not been zeroized at the time of the crash. Even with `Zeroizing` on drop, a crash before drop means the secret is captured in the core dump.

**Why it happens:**
Core dumps are a kernel-level feature. The `zeroize` crate cannot prevent the kernel from snapshotting process memory. It only protects against the compiler optimizing away zeroing, not against the process dying before zeroing occurs.

**How to avoid:**
1. **Disable core dumps for the process at startup** using `nix::sys::resource::{setrlimit, Resource}` to set `RLIMIT_CORE` to 0. This prevents the kernel from writing a core dump even on crash.
2. If `nix` is not available (it's already in `Cargo.toml`), use the `libc` syscall directly: `libc::setrlimit(libc::RLIMIT_CORE, &libc::rlimit { rlim_cur: 0, rlim_max: 0 })`.
3. Document in the README that users should configure `ulimit -c 0` if running the tool in environments where core dumps are enabled.
4. The CLI should call this in `main()` before any password generation occurs.

**Warning signs:**
- No `setrlimit` call in `main()`
- Process crashes can be triggered by malformed input or resource exhaustion
- System configured with core dumps enabled (`ulimit -c unlimited`)

**Phase to address:**
Phase 4 (CLI Development) — core dump prevention is a startup concern in the CLI entry point.

---

### Pitfall 8: stdout-buffered password sits in kernel buffers

**What goes wrong:**
When the CLI prints the password to stdout using `println!()`, the password sits in the stdout buffer (8KiB by default with `BufWriter`, or unbuffered line-by-line with `println!`). If the program crashes or is killed between the `println!` and the actual flush to the terminal, the password may never reach the user, OR the password persists in kernel pipe buffers if stdout is piped. More critically, on some terminal emulators, the scrollback buffer preserves the password indefinitely — shoulder surfing or terminal history capture becomes possible.

**Why it happens:**
`println!` flushes to the terminal immediately, but terminal emulators buffer output for scrollback. There's no standard cross-terminal way to clear scrollback, unlike the `pwgen` approach of clearing the *visible* screen.

**How to avoid:**
1. **Flush stdout explicitly** after printing the password: `std::io::stdout().flush()?` to ensure the password is written to the terminal before the program exits.
2. **Print a prominent warning** when stdout is a TTY, advising the user to close the terminal after copying the password.
3. **Do NOT attempt to clear scrollback buffers** — it's terminal-specific and unreliable. Instead, educate the user.
4. Consider printing the password on its own line with no other output on the same line, so it can be easily selected with triple-click in most terminals.

**Warning signs:**
- No explicit `stdout.flush()` after printing the password
- Password printed alongside other text (makes selection harder, increases scrollback exposure)

**Phase to address:**
Phase 4 (CLI Development) — CLI output formatting concern.

---

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems in a security crypto tool.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Returning `String` instead of `Zeroizing<String>` | Simpler API, no `Zeroizing` ergonomics in callers | Password in heap forever; violates AGENTS.md security rules | **Never** for a password generator |
| Using `unwrap()` / `expect()` on `OsRng` | Less error handling code | RNG failure panics instead of graceful error; violates AGENTS.md | **Never** in production paths |
| `#[derive(Debug)]` on config/args that may hold secrets downstream | Free debug printing | Debug formatting may leak secrets through panic backtraces or logging | **Never** if the struct transitively touches secrets |
| Skipping `IsTerminal` check on stdout | No branching on output mode | Passwords leak to files/scripts unpredictably; users confused | Only acceptable if the CLI *always* pipes (no interactive mode) |
| Hardcoding charset group mapping by index (already present) | Works for the current 4 groups | Adding a 5th group silently breaks the flag-to-group mapping | Only acceptable if refactored to explicit keyed mapping first |
| Not setting `RLIMIT_CORE=0` | "Won't crash" assumption | Core dump on unexpected crash leaks all memory including passwords | **Never** for any program handling secrets |

## CLI Integration Gotchas

Specific to wiring `clap` + core library + secrets in Rust.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| `clap` `value_parser!` on numeric args | Accepting negative values for `length`, silently wrapping or panicking | Use `value_parser!(usize).range(1..)` or a custom `PasswordLength` newtype with validation |
| `clap` `default_value_t` | Hardcoding defaults in derive attributes that differ from `DEFAULTS` in `lib.rs` | Derive `Default` from `lib.rs::DEFAULTS` or import `DEFAULTS` constant as derive defaults |
| `std::process::exit()` in library code | Library calls `exit()` making it untestable and non-reusable | Library returns `Result`; only `main()` decides whether to exit |
| Printing to both `stdout` and `stderr` | Mixing password (data) and metadata (diagnostic) in same stream | Password → stdout. Everything else (entropy, warnings, errors) → stderr |
| `clap` `env` attribute on secret args | `#[arg(env = "SEED")]` reads secrets from environment, which leaks via `/proc/<pid>/environ` | Never use `env` for secrets. Use stdin or file descriptor passing. |
| Signal handling (Ctrl+C) cleanup | No handler → process killed mid-generation, password possibly in terminal buffer unscrollback | Install `ctrlc` handler that zeroizes buffers then exits gracefully. Set `RLIMIT_CORE=0` before any allocation. |
| `--version` / `--help` exits with code 0 | Clap default behavior — fine for help, but version banner may confuse scripted consumers | `--help` → stdout (exit 0). `--version` → stdout (exit 0). Both are standard clap behavior and correct. |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| `build_character_pool` O(n*m) deduplication | Slow generation with large Unicode charsets | Replace `Vec::contains` nested loop with `HashSet<char>` then collect to `Vec` | With `utf8_flag=true` and Unicode supplementary planes (tens of thousands of chars) |
| Re-instantiation of `OsRng` per generation call | Modest syscall overhead per password | Acceptable for CLI (single password). Only optimize if generating thousands. | Not a concern at CLI scale |
| `println!` in hot loop for batch generation | Terminal output is surprisingly slow (documented in rust-cli book) | Use `BufWriter` + batch output for multi-password generation | When generating 100+ passwords in one invocation |

## Security Mistakes

Domain-specific security issues beyond general CLI hygiene.

| Mistake | Risk | Prevention |
|---------|------|------------|
| `printf`-style formatting of password (`println!("Password: {}", pw)`) | Password visible in scrollback, terminal multiplexer history, screen recording | Print password without label on its own line; metadata (entropy stats) separately to stderr |
| Copying password to clipboard without cleanup | Password persists in clipboard history (wl-clipboard, cliphist, Klipper) indefinitely | Deferred to clipboard phase. For now, do NOT copy to clipboard. Document that users should pipe to `wl-copy` or `xclip` manually. |
| Returning password as `String` without zeroize | Password in freed heap; recoverable via memory forensics | Return `Zeroizing<String>` — see Pitfall 1 |
| Not calling `stdout.flush()` after printing password | Password stuck in buffer, user doesn't see it, terminal state undefined on crash | Call `io::stdout().flush()?` after last `println!` containing the password |
| Using `dbg!()` or `eprintln!` for password debugging | Password leaked to stderr even in debug builds | NEVER print the password value, even in debug. Use `assert!()` and property tests instead. |
| `OsRng` fallibility unhandled | Process panics; no password generated; confusing error for user | Return `PasswordError::RngFailure`. CLI prints "System random number generator unavailable" to stderr and exits non-zero. |
| Config file parsing without validation | TOML config with `length = 0` or nonsensical values silently picked up | Validate all config values at parse time. Reject invalid configs before any generation. |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Printing entropy as raw `f64` with 15 decimal places | `"Entropy: 92.383938473920183 bits"` — misleading precision, hard to parse | Round to 1 decimal place: `"Entropy: 92.4 bits (classical), 46.2 bits (quantum)"` |
| Quantum-adjusted entropy presented as "real entropy" | Users think their password is half as strong as it is; misunderstanding of Grover's algorithm | Label clearly: "Classical entropy: 92.4 bits" and "Quantum-adjusted (Grover): 46.2 bits". Explain the adjustment is a theoretical bound. |
| Error messages that are too technical | `"ExhaustedGroup { group_index: 2 }"` means nothing to a user | Map `PasswordError` variants to user-friendly messages in the CLI layer: `"No usable characters remain in the digit group after filtering."` |
| No `--help` examples | Users don't know how to compose flags | Include a realistic example in `#[command(after_help = "...")]`: `"Example: you-shall-not-pass --length 20 --no-symbols"` |
| Silent failure on invalid config | CLI exits with error but no suggestion for fix | Use clap's built-in `error.format(&mut stderr)` or suggest valid values: `"Length must be at least 4 when all character groups are enabled."` |
| No `--json` output for scripting | Scripts forced to parse human-readable output with `grep`/`awk` | Add `--json` flag that outputs `{"password":"...","entropy_classical":92.4,"entropy_quantum":46.2}` as a single JSON line. Password field still goes to stdout; structured data to... hmm, actually: when `--json` is set, output ONLY the JSON object to stdout (including password). When not set, TTY-aware behavior. |

## "Looks Done But Isn't" Checklist

Things that appear complete in a CLI password generator but are missing critical pieces.

- [ ] **Password generation:** Function returns a password — BUT is it `Zeroizing<String>` or plain `String`? Verify return type.
- [ ] **Entropy display:** Numbers print to screen — BUT are they rounded appropriately? Are classical and quantum clearly labeled? Verify output formatting.
- [ ] **Error handling:** `PasswordError` enum exists — BUT does the CLI map each variant to a user-friendly message? Does it exit with the correct code? Verify `main()` error mapping.
- [ ] **CLI arg parsing:** `clap` derive struct compiles — BUT are defaults consistent with `DEFAULTS` in `lib.rs`? Are validation ranges correct? Verify by comparing defaults.
- [ ] **TTY-aware output:** Password prints — BUT does it detect pipe vs TTY? Is entropy on stderr when piping? Verify with `| cat` and interactive terminal.
- [ ] **RNG error path:** `generate_password()` calls `OsRng` — BUT does it handle `TryRngCore` fallibility? Does the error propagate to the CLI? Trigger by running in a restricted environment.
- [ ] **Memory hygiene:** `Zeroizing` used in generation — BUT is the returned password also `Zeroizing`? Are core dumps disabled? Verify with `grep RLIMIT_CORE src/`.
- [ ] **No secret args:** `clap` flags accept length, bools, etc. — BUT is there any `--seed`, `--password`, `--key`, or `--charset` flag? Verify with `grep -E 'long\s*=\s*"(seed|key|pass|charset)"' src/main.rs`.
- [ ] **Panic handler:** Process can crash — BUT is `human-panic` installed? Does a panic dump secrets via `Debug`? Trigger a test panic and inspect output.
- [ ] **Test coverage:** Code compiles — BUT are there tests for generation, entropy math, edge cases? Verify `cargo test` output is non-empty.

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| `String` return type (Pitfall 1) | MEDIUM | Change signature to `Zeroizing<String>`. Update all callers (currently just placeholder `main.rs`). Auditing the codebase is trivial since only `main.rs` calls `generate_password`. |
| RNG panic (Pitfall 3) | LOW | Add `PasswordError::RngFailure` variant. Wrap `OsRng` in a `try_*` method. Update `main.rs` to handle the new error. No API consumers to break. |
| TTY detection missing (Pitfall 2) | LOW | Add `IsTerminal` check in `main.rs`. Two branches (TTY vs pipe) with different output formats. Refactor after initial implementation if needed. |
| Panic secrets leak (Pitfall 5) | LOW | Add `human-panic` dependency. Call `setup_panic!()` in `main()`. Review `#[derive(Debug)]` uses. |
| Core dump leak (Pitfall 7) | LOW | Add one `setrlimit` call at the top of `main()`. The `nix` crate is already in `Cargo.toml`. |
| Entropy inconsistency (Pitfall 6) | LOW | Add missing validation to `calc_entropy()`. Add a test that entropy calc and generation use the same constraint set. |
| Shell history (Pitfall 4) | HIGH (retroactive) | If a secret-accepting arg was ever released, all users who ran it have the secret in their shell history. Prevention is critical — never ship secret-bearing CLI args in the first place. |

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Pitfall 1: Non-zeroized return `String` | Phase 4 (CLI) | `grep '-> Result<String>' src/lib.rs` returns zero matches; `grep 'Zeroizing<String>' src/lib.rs` confirms return type |
| Pitfall 2: Missing TTY detection | Phase 4 (CLI) | `cargo test` with pipe capture validates TTY vs pipe output differ; manual `| cat` test |
| Pitfall 3: RNG failure unhandled | Phase 4 (CLI) | Unit test with mock RNG failure path; `PasswordError::RngFailure` exists and is mapped in CLI |
| Pitfall 4: Shell history via args | Phase 4 (CLI) | Audit all `#[arg(...)]` attributes in CLI derive struct — no `secret`/`seed`/`key` args exist |
| Pitfall 5: Panic secrets leak | Phase 4 (CLI) | `human-panic` installed; `#[derive(Debug)]` audit completed; no `Debug` on secret-adjacent types |
| Pitfall 6: Entropy inconsistency | Phase 4 (CLI) | `calc_entropy()` and `generate_password()` share validation; test validates same constraints |
| Pitfall 7: Core dump leak | Phase 4 (CLI) | `RLIMIT_CORE = 0` call in `main()`; verify with `prlimit --core` on running process |
| Pitfall 8: Scrollback persistence | Phase 4 (CLI) | `stdout.flush()` after password; warning printed when TTY detected |
| Magic index charset mapping | Phase 5 (Config/Refactor) | Replace magic indices with named keys or enum variants; config-driven charset groups |
| Entropy model refinement | Phase 5 or 6 (Passphrase) | Adjust formula for guaranteed-character constraint; property test validates entropy bounds |

## Sources

- [clap 4.6.1 official docs](https://docs.rs/clap/4.6.1/clap/) — HIGH confidence (official crate docs)
- [zeroize 1.8.2 official docs](https://docs.rs/zeroize/1.8.2/zeroize/) — HIGH confidence (official crate docs)
- [Rust CLI Book — Output for humans and machines](https://rust-cli.github.io/book/tutorial/output.html) — HIGH confidence (official Rust WG-CLI resource)
- [Rust CLI Book — Signal handling](https://rust-cli.github.io/book/in-depth/signals.html) — HIGH confidence (official Rust WG-CLI resource)
- [Rust CLI Book — Exit codes](https://rust-cli.github.io/book/in-depth/exit-code.html) — HIGH confidence (official Rust WG-CLI resource)
- [Rust CLI Book — Human communication](https://rust-cli.github.io/book/in-depth/human-communication.html) — HIGH confidence
- [Rust CLI Book — Machine communication](https://rust-cli.github.io/book/in-depth/machine-communication.html) — HIGH confidence
- [pwgen(1) man page — Arch Linux](https://man.archlinux.org/man/pwgen.1.en) — HIGH confidence (official upstream documentation with explicit security warnings about shell history and `-H` flag)
- `.planning/codebase/CONCERNS.md` — HIGH confidence (codebase-specific audit, dated 2026-04-26, authored by this project's research)
- `.planning/codebase/ARCHITECTURE.md` — HIGH confidence (codebase-specific architecture audit)
- `src/lib.rs` — author's own code analysis — HIGH confidence (direct source inspection)
- AGENTS.md — HIGH confidence (project's own security rules and conventions)
- Project constraints: Rust edition 2024, `unsafe_code = "forbid"`, `thiserror`, `Zeroizing`, `clap` for CLI

---

*Pitfalls research for: Rust CLI password/passphrase generator with quantum entropy modelling*
*Researched: 2026-04-26*
