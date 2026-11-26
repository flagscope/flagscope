> Status: approved
> Last updated: 2026-11-26
> Mode: new-project

# Decisions: find-feature-ids

## AD-01: Use a virtual Cargo workspace with a flat `crates/*` layout

Date: 2026-11-26
Context: the user asked for one repository that keeps the logic and the user interface in separate crates and can grow cleanly.
Alternatives considered:
- One package with a library target and a binary target: simplest, but every dependency is shared by everything, and the GPUI dependency would reach the logic.
- A root package plus member crates: mixes the product with the workspace definition, and the root crate becomes the default for every `cargo` command.
- Nested directories by area (`crates/scanner/symbols/...`): useful for very large workspaces, but harder to scan at a glance with fewer than about ten crates.
Decision: a virtual manifest at the root, `members = ["crates/*"]`, with shared `[workspace.package]`, `[workspace.dependencies]`, and `[workspace.lints]`.
Consequences: every crate is a directory under `crates/`; adding a crate needs no root change beyond its dependencies. Commands need `--workspace` or `-p` to choose crates. Dependency versions are changed in one place.
Status: active

## AD-02: Split the logic into `core`, `symbols`, and `scanner` library crates, plus a thin CLI

Date: 2026-11-26
Context: the scan pipeline mixes pure parsing, network and file I/O, and orchestration. The same pipeline will later be driven by the GPUI application, so it cannot live in the command-line crate.
Alternatives considered:
- One `flagscope-lib` crate with modules: fewer manifests, but no compile-time boundary; network code could leak into parsing and the GPUI crate would pull in everything.
- One crate per step (PE reader, symbol server, PDB extractor, list cache, pipeline): the boundaries would be clean, but five crates for one feature means more manifests and more public API surface than the code needs.
Decision: `flagscope-core` (domain types and mach2 format, no I/O), `flagscope-symbols` (PE debug information and symbol server), `flagscope-scanner` (extraction and pipeline), and `flagscope-cli` (binary `flagscope`), with the boundaries in `03-architecture.md`.
Consequences: parsing is tested offline with project fixtures, and network code is isolated in one crate. The pipeline's public API must be designed for two front ends (CLI now, GPUI later). A future split of `flagscope-scanner` is possible if it grows, without touching the other crates.
Status: active

## AD-03: Run unelevated; relaunch elevated only to apply feature changes

Date: 2026-11-26
Context: changing feature states needs administrator rights. Scanning downloads files from the Internet and parses them, which should never run with those rights.
Alternatives considered:
- The whole application requires administrator rights through its manifest: simplest, but every download and every parsed file runs elevated, and a parsing bug becomes a privileged one.
- A Windows service or a permanent elevated helper with inter-process communication: secure, but adds a protocol, an installer, and a resident process to a portable tool.
Decision: the application always starts unelevated. To apply changes, it starts its own executable elevated (UAC prompt) with a command and an input file; that process takes the snapshot, applies the changes, and exits. Chosen by the user among the alternatives offered.
Consequences: the scanner never needs elevation (RNF-04.1). The elevated command line is a trust boundary and must validate its input. The design of that command belongs to the `toggle-features-safely` feature.
Status: active

## AD-04: Accept only GPL-3.0-compatible, pure-Rust dependencies

Date: 2026-11-26
Context: the project is GPL-3.0-only and open to contributors, who should be able to build it with the standard Rust toolchain alone.
Alternatives considered:
- Reuse the tools mach2 used (DIA SDK `msdia140.dll`, Capstone): proven, but they are C or C++ components that need separate installation or a C toolchain, and the DIA SDK cannot be redistributed freely.
- Accept any open-source licence: some licences (for example GPL-2.0-only or SSPL) are incompatible with GPL-3.0-only.
Decision: dependencies must have a licence compatible with GPL-3.0-only (MIT, Apache-2.0, BSD, MPL-2.0, Zlib, and similar), be written in Rust or wrap only Windows system libraries, and be maintained. The user may revise this rule if it blocks a necessary capability.
Consequences: every new dependency is checked against this rule before it is added. Some capabilities may need more own code than with C libraries. Revisiting the rule needs a new decision entry.
Status: active

## AD-05: Read PDBs with `ms-pdb`

Date: 2026-11-26
Context: the scanner reads global symbols and constant values from Microsoft PDBs.
Alternatives considered:
- `pdb` 0.8.0 (MIT OR Apache-2.0): mature and read-only, but its last release was in 2022.
- `pdb2` (fork of `pdb`): more recent, but a smaller community and unclear long-term maintenance.
- Writing an MSF/CodeView parser: full control, but a large amount of format work that existing crates already do.
Decision: `ms-pdb` (0.1.22, MIT OR Apache-2.0), published by Microsoft and released regularly in 2025 and 2026. If HU-01 shows that it cannot read the constant values the scanner needs, `pdb` is the fallback, recorded as a new decision.
Consequences: version 0.1.x may change its API between releases; it is pinned in `[workspace.dependencies]` and upgraded deliberately. HU-01 confirms that it reads the symbols the scanner needs.
Status: active

## AD-06: Read PE debug information with `object`

Date: 2026-11-26
Context: to download a binary's PDB, the scanner needs the PDB name, GUID, and age from the binary's CodeView debug directory.
Alternatives considered:
- `goblin`: also parses PE, but `object` is more widely used across the Rust toolchain ecosystem and exposes the CodeView record directly.
- Own PE parser: the CodeView record is small, but PE parsing has many edge cases that are already solved.
Decision: `object` (0.41.0, Apache-2.0 OR MIT), read-only features.
Consequences: one more dependency in `flagscope-symbols`, maintained alongside the Rust toolchain ecosystem. Binaries without a CodeView record are skipped and counted.
Status: active

## AD-07: Download symbols with the blocking `ureq` client

Date: 2026-11-26
Context: the scanner downloads many files and needs their sizes for the estimate (RF-13.1).
Alternatives considered:
- `reqwest` with an async runtime (`tokio`): powerful, but brings an async runtime and a much larger dependency tree into a tool that does not otherwise need async code.
- Calling WinHTTP through FFI: no extra dependency, but adds `unsafe` code outside the store crate.
Decision: `ureq` (3.5.2, MIT OR Apache-2.0), blocking, with parallelism (if HU-04 measurements call for it) through standard threads. The TLS back end is chosen in HU-01: the Windows system TLS through `native-tls` is preferred, so that no C compiler is needed beyond the standard MSVC toolchain.
Consequences: no async code in the workspace. Parallel downloads, if added, use `std::thread::scope`. The GPUI application must run scans off its UI thread.
Status: active

## AD-08: Take feature IDs from PDB constants; defer binary disassembly

Date: 2026-11-26
Context: mach2 read each feature's ID from the constant value of its `::id` symbol in the PDB, and as a fallback disassembled the binary's getter function to read the ID from a `mov` instruction.
Alternatives considered:
- Implement both paths now: covers more features, but doubles the work before knowing whether the fallback is needed on current builds.
- Disassembly only: does not need PDB constants, but still needs PDBs to locate the functions, and is fragile across compiler changes.
Decision: extract IDs from PDB constants only. The disassembly fallback is added, with `iced-x86` (MIT), only if HU-01 or HU-02 show that a significant share of features have no ID in the PDB; that case is reported to the user first.
Consequences: features whose ID is only in the binary are reported as "name without ID" and counted, not dropped silently. The name patterns follow mach2's list of symbol name forms.
Status: active

## AD-09: Store lists and temporary symbol files under the user's local application data

Date: 2026-11-26
Context: saved lists (RF-07) and downloaded symbol files (RF-12) need a location that works without administrator rights and does not roam.
Alternatives considered:
- Next to the executable: breaks when the executable is in a read-only or protected folder.
- `%APPDATA%` (roaming): lists are machine-specific and symbol files are large; roaming them is wasteful.
- The system temporary folder for symbols: may be cleaned at any time and is shared with other applications.
Decision: `%LOCALAPPDATA%\flagscope\lists\<build>.txt` for saved lists, in the mach2 format, and `%LOCALAPPDATA%\flagscope\symbols\` for downloaded symbol files. The location is read from the environment with the standard library.
Consequences: no extra dependency. Deleting the folder resets the tool. The user can read and share saved lists directly.
Status: active

## AD-10: Report scan progress through an observer trait

Date: 2026-11-26
Context: the estimate and the progress (RF-13) are computed in `flagscope-scanner` but displayed by the CLI now and by GPUI later.
Alternatives considered:
- The scanner prints progress itself: simplest, but ties the library to the console and breaks the boundary in `03-architecture.md`.
- A channel of progress events: decouples threads well, but forces every caller to drain a channel even when it does not need progress.
Decision: the scan function takes a reference to a `ScanObserver` trait whose methods receive the estimate, progress updates, and the final totals. The CLI implements it for the console; GPUI will implement it by forwarding to its UI thread.
Consequences: the scanner has no user-facing text. A no-op observer serves tests. The trait is part of the public API and changes to it affect every front end.
Status: active
