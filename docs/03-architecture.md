> Status: approved
> Last updated: 2026-11-26
> Mode: new-project

# Architecture

## Overview

flagscope is a Cargo workspace of small crates. Library crates hold all the logic, and binary crates only parse input, call the libraries, and render the output. For the first feature, three library crates form a pipeline: `flagscope-symbols` finds and downloads the symbol file of each system binary, `flagscope-scanner` extracts feature names from those symbol files and the matching IDs from the binaries (AD-12) and orchestrates a full scan, and `flagscope-core` holds the domain types and the mach2 list format they share. The `flagscope-cli` binary crate exposes the scan on the command line. Later features add a crate for the Windows feature store and a crate for the GPUI user interface, without changing the existing crates' boundaries.

## Boundaries

| Crate | Kind | Owns | Must not know about |
| ----- | ---- | ---- | ------------------- |
| `flagscope-core` | library | Domain types (`FeatureId`, `FeatureName`, `Build`, `FeatureList`) and the mach2 text format (parse and serialise). No I/O beyond `std::io` readers and writers. | The network, the file system layout, PDBs, Windows APIs, any user interface. |
| `flagscope-symbols` | library | Reading the CodeView debug information (PDB name, GUID, age) from PE files; the Microsoft symbol server client (URL, size query, download); the temporary symbol-file directory. | Feature names or IDs, list caching, any user interface. |
| `flagscope-scanner` | library | Extracting `Name: ID` pairs from a PDB and its binary; the scan pipeline (choosing binaries, estimate, download, extraction, de-duplication, saving the list per build, deleting symbol files); scan progress events. | How progress or results are displayed; the Windows feature store. |
| `flagscope-cli` | binary (`flagscope`) | Command-line arguments, console output, exit codes, the output file. | Symbol-server details, PDB parsing. It calls `flagscope-scanner`, and uses `flagscope-core` only for the domain types and the list format. |
| `flagscope-store` (later, `toggle-features-safely`) | library | FFI to the feature configuration functions of `ntdll.dll`, snapshots, revert. | The scanner, any user interface. |
| `flagscope-gui` (later, `desktop-app`) | binary | The GPUI user interface. | Anything below the public APIs of the library crates. |

Dependency direction: `flagscope-cli` → `flagscope-scanner` → `flagscope-symbols` and `flagscope-core`; `flagscope-cli` also uses `flagscope-core`. No library crate depends on a binary crate. Library crates do not depend on each other in a cycle.

## Conventions

**Workspace.** Virtual manifest at the root with `members = ["crates/*"]`. Shared `[workspace.package]` (edition, `rust-version`, license, repository), `[workspace.dependencies]` (one version per dependency), and `[workspace.lints]`; every crate inherits them. `Cargo.lock` is committed.

**Toolchain.** Edition 2024. `rust-version = "1.100"`, the stable version current when the workspace was created (2026-11). It is raised deliberately, with a decision entry, never by accident.

**Formatting.** `rustfmt` with default settings; `cargo fmt --all --check` reports zero differences.

**Lints.** `cargo clippy --workspace --all-targets -- -D warnings` reports zero warnings. Workspace lints:
- `unsafe_code = "deny"`; only the FFI module of `flagscope-store` may allow it. Other Windows access goes through safe wrappers (for example `windows-registry` to read the build number in `flagscope-scanner`).
- `missing_docs = "warn"` (an error through `-D warnings`).
- Clippy: `undocumented_unsafe_blocks`, `missing_safety_doc`, `missing_errors_doc`, and `missing_panics_doc`.

**Documentation comments.** Every public item has a rustdoc comment in the style of the Rust API Guidelines. Functions that return `Result` have an `# Errors` section, functions that can panic have a `# Panics` section, and `unsafe` functions have a `# Safety` section. Every `unsafe` block has a `// SAFETY:` comment that states why the call is sound. Comments explain why, not what.

**Errors.** Each library crate defines its own error enum with `thiserror`; library code never panics on input it does not control and never calls `unwrap` outside tests. The binary crate maps errors to a message and a non-zero exit code.

**Naming.** Crates are `flagscope-<area>`. Types and modules follow the Rust API Guidelines naming conventions. Terms follow `CONTRIBUTING.md` ("Terminology").

**Tests.** Unit tests next to the code (`#[cfg(test)]`). Integration tests in each library crate's tests directory, using fixtures the project creates itself, never Microsoft symbol files. End-to-end tests run the `flagscope` executable. Tests that need the network or the real feature store are marked `#[ignore]` with a reason and run separately.

**User-facing text.** Every string a user reads is defined in one messages module per binary crate, never inline, so that the translation layer of the `desktop-app` feature can replace that module without touching the logic.

**Commits.** Conventional Commits with the story ID, for example `feat(scanner): extract feature IDs from PDB constants (HU-01)`, plus the DCO sign-off.

## Decisions

| ID | Decision | Where |
| -- | -------- | ----- |
| AD-01 | Virtual Cargo workspace with a flat `crates/*` layout | `features/find-feature-ids/decisions.md` |
| AD-02 | Split the logic into `core`, `symbols`, and `scanner` library crates, plus a thin CLI | `features/find-feature-ids/decisions.md` |
| AD-03 | Run unelevated; relaunch elevated only to apply feature changes | `features/find-feature-ids/decisions.md` |
| AD-04 | Accept only GPL-3.0-compatible, pure-Rust dependencies (superseded by AD-11) | `features/find-feature-ids/decisions.md` |
| AD-05 | Read PDBs with `ms-pdb` | `features/find-feature-ids/decisions.md` |
| AD-06 | Read PE debug information with `object` | `features/find-feature-ids/decisions.md` |
| AD-07 | Download symbols with the blocking `ureq` client | `features/find-feature-ids/decisions.md` |
| AD-08 | Take feature IDs from PDB constants; defer binary disassembly (superseded by AD-12) | `features/find-feature-ids/decisions.md` |
| AD-09 | Store lists and temporary symbol files under the user's local application data | `features/find-feature-ids/decisions.md` |
| AD-10 | Report scan progress through an observer trait | `features/find-feature-ids/decisions.md` |
| AD-11 | Accept GPL-3.0-compatible dependencies in Rust, C, or C++ that build with the standard Windows Rust toolchain | `features/find-feature-ids/decisions.md` |
| AD-12 | Take feature names from PDB public symbols and IDs from the feature descriptor in the binary | `features/find-feature-ids/decisions.md` |
| AD-13 | Scan only x64 system binaries whose bytes reference the feature-staging API | `features/find-feature-ids/decisions.md` |
| AD-14 | Merge features found in several binaries and report conflicting IDs | `features/find-feature-ids/decisions.md` |
| AD-15 | Parse command-line arguments with `clap` | `features/find-feature-ids/decisions.md` |
| AD-16 | Store published lists as one text file per build, grouped by architecture and major build | `features/published-lists/decisions.md` |
| AD-17 | `flagscope scan` uses the saved list by default; `--keep-symbol-files` implies a new scan | `features/find-feature-ids/decisions.md` |
| AD-18 | Estimate a scan by asking the server for every size in parallel and timing one partial download | `features/find-feature-ids/decisions.md` |
| AD-19 | Read light-mode symbol files from memory, and download distinct symbol files in parallel | `features/lighter-scans/decisions.md` |
