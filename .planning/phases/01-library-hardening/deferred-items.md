# Deferred Items — Phase 01 Library Hardening

## Pre-existing Clippy Warnings (not introduced by this plan)

Found during 01-01-PLAN.md execution. These are out-of-scope per executor scope boundary rules.

1. **unused variable: `index`** — `src/lib.rs:83`
   - Variable `index` is destructured in `for (index, chars) in ...` but never used.
   - Fix: prefix with `_`: `for (_index, chars)` or `for (_, chars)`.

2. **needless lifetimes on `enabled_groups`** — `src/lib.rs:128`
   - Function: `fn enabled_groups<'a>(config: &'a Defaults) -> Vec<&'a str>`
   - Lifetimes can be elided: `fn enabled_groups(config: &Defaults) -> Vec<&str>`
   - Flagged by `clippy::needless_lifetimes`.

Both should be resolved before or during Phase 2 to achieve clean `cargo clippy` output.
