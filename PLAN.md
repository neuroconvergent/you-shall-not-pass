---
title: You Shall Not Pass
author: Sundar Gurumurthy
date: 13 December, 2025
---

# You Shall Not Pass

**You Shall Not Pass** is a quantum brute-force modelling password and passphrase
generator written in Rust. It provides both a CLI and a `GTK-rs` GUI for generating
high-entropy secrets and visualizing their effective security under classical and
quantum brute-force threat models.

The application displays metrics such as classical entropy and quantum-adjusted
entropy (accounting for Grover-style quadratic speedups). When the
_quantum-secure_ mode is enabled, the tool automatically recommends generation
parameters that satisfy user-defined constraints (maximum length, allowed
character sets, policy requirements) while maintaining a target security level.

The primary aim of this project is to generate long-lived passwords, passphrases,
and key material for remotely stored or archived secrets that could be targeted
by _steal now, decrypt later_ attacks (e.g., backups, encrypted containers,
password databases, SSH/PGP private keys).

---

## Features

1. **GTK-based UI** for interactive password and passphrase generation.
2. **Quantum brute-force modelling mode**, displaying both classical and
   quantum-adjusted entropy estimates.
3. **Automatic parameter recommendation** to meet a target security level under
   user-specified constraints.
4. **Dotfile-based configuration** following XDG conventions.
5. **Secure GTK clipboard integration**, including:
   - time-based clipboard clearing
   - optional, user-configurable commands to remove secrets from clipboard
     history tools (e.g. `cliphist`)
6. **Presets for common policies**, such as:
   - NIST-style entropy targets
   - corporate password rules
   - long-term archival secrets (LUKS, backups, private keys)

---

## Project Plan

1. **Threat Model & Entropy Research**
   - Classical brute-force cost models
   - Quantum brute-force impact (Grover’s algorithm)
   - NIST entropy guidance and password policy constraints

2. **Project Setup**
   - Initialize Rust project with Cargo
   - Define crate structure (core library + CLI + GUI)
   - Configure dotfile-based settings (XDG-compliant)

3. **Core Library Development**
   - Cryptographically secure random generation
   - Password and passphrase generation strategies
   - Entropy and quantum-adjusted entropy calculations

4. **CLI Development**
   - Fully functional CLI for password generation
   - Display of entropy metrics and recommendations
   - Clipboard output support

5. **GUI Development**
   - GTK UI as a wrapper around the core library
   - Interactive parameter selection and visualization
   - Clipboard integration using GTK APIs

6. **Testing & Security**
   - Unit and property-based tests
   - Fuzzing for configuration and input handling
   - Memory hygiene and secret zeroization
   - Review of threat assumptions and documentation

7. **Deployment & Documentation**
   - Packaging for Linux distributions (e.g., Flatpak/AppImage)
   - User documentation and examples
   - Clear explanation of security model and limitations

---

## Rust Packages

- **`gtk-rs`**  
  GTK bindings for building the graphical user interface.

- **`rand`**  
  Cryptographically secure random number generation via OS entropy sources.

- **`getrandom`**  
  Explicit access to platform-provided entropy.

- **`serde`** + **`toml`**  
  Parsing and managing dotfile-based configuration.

- **`clap`**  
  Command-line argument parsing for the CLI interface.

- **`unicode-segmentation`**  
  Optional support for grapheme-aware password generation.

- **`nix`**  
  Unix-specific functionality (e.g., secure file handling, optional keyfile
  generation).

- **`zeroize`**  
  Securely clearing sensitive data from memory.

- **`thiserror`** or **`anyhow`**  
  Structured and idiomatic error handling.

---

## Theory References

- **NIST Post-Quantum Cryptography Project**  
  https://csrc.nist.gov/projects/post-quantum-cryptography  
  (context for long-term cryptographic threats)

- **NIST SP 800-63B — Digital Identity Guidelines**  
  (entropy and password policy guidance)

- **OWASP Password Storage Cheat Sheet**  
  https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html  
  (constraints and best practices for password usage)

- _Quantum Computing and Cryptography_ — Michel Boyer  
  (overview of quantum threats to classical security assumptions)

- _Cryptography Engineering_ — Ferguson, Schneier, Kohno  
  (practical cryptographic threat modeling and design principles)

---

## Non-Goals

- Implementing cryptographic primitives
- Implementing post-quantum encryption or key exchange algorithms
- Replacing full password managers or encryption software

This project focuses on **high-entropy secret generation and transparent security
modelling**, not cryptographic protocol design.

