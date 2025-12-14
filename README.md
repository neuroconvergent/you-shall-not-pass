# You Shall Not Pass

You Shall Not Pass is a Rust-based password and passphrase generator with
quantum brute-force modelling. It aims to provide both a CLI and a GTK
application for generating high-entropy secrets and visualising their
resistance to classical and Grover-style quantum brute-force attacks.

The project is focused on long-lived secrets (backups, key material, password
stores) that may be exposed to "steal now, decrypt later" threats.

> [!NOTE]
> Status: early in development. Core library, CLI, and GTK UI are being
> designed; the current code is just a scaffold.

## Features (planned)

- Core library for secure password and passphrase generation
- Entropy calculations with quantum-adjusted brute-force estimates
- CLI frontend with entropy metrics and presets
- GTK frontend for interactive exploration of parameters
- XDG-compliant configuration and policy presets
- Secure clipboard integration with optional cleaning helpers

See [`PLAN.md`](PLAN.md) for a detailed roadmap, background notes, and theory references.

## Getting started

### Prerequisites

- A recent Rust toolchain with support for the 2024 edition
- GTK 4 development libraries (for the future GUI)

### Build, run, test

```bash
# Build
cargo build

# Run CLI (currently a simple scaffold)
cargo run

# Run tests (when added)
cargo test
```

## Security scope and non-goals

- Uses OS-backed CSPRNGs; does not implement cryptographic primitives
- Models brute-force cost; does not provide post-quantum encryption schemes
- Does not replace full password managers or disk encryption tools

## License

This project is licensed under the
[BSD-3-Clause-No-Military-License](LICENSE.txt) terms.
