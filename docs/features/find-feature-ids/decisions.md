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
Status: superseded by AD-11

## AD-05: Read PDBs with `ms-pdb`

Date: 2026-11-26
Context: the scanner reads global symbols and constant values from Microsoft PDBs.
Alternatives considered:
- `pdb` 0.8.0 (MIT OR Apache-2.0): mature and read-only, but its last release was in 2022.
- `pdb2` (fork of `pdb`): more recent, but a smaller community and unclear long-term maintenance.
- Writing an MSF/CodeView parser: full control, but a large amount of format work that existing crates already do.
Decision: `ms-pdb` (0.1.22, MIT OR Apache-2.0), published by Microsoft and released regularly in 2025 and 2026. `pdb` (0.8.0, pure Rust) is the backup reader, confirmed by the user on 2026-11-26: on 2026-11-26 both crates read the same 261 feature symbols from the `shell32.pdb` of build 26300.10075. Only one reader is compiled in. All PDB access goes through one module of `flagscope-scanner`, so switching to `pdb` means replacing that module, recorded as a new decision, if `ms-pdb` fails to read a PDB or stops being maintained.
Consequences: `ms-pdb` returns `anyhow::Error`, so `anyhow` is a direct dependency of `flagscope-scanner`, used only inside the PDB module and converted to the crate's own error there. Version 0.1.x may change its API between releases; it is pinned in `[workspace.dependencies]` and upgraded deliberately. HU-01 confirms that it reads the symbols the scanner needs.
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
Status: superseded by AD-12

## AD-09: Store lists and temporary symbol files under the user's local application data

Date: 2026-11-26
Context: saved lists (RF-07) and downloaded symbol files (RF-12) need a location that works without administrator rights and does not roam.
Alternatives considered:
- Next to the executable: breaks when the executable is in a read-only or protected folder.
- `%APPDATA%` (roaming): lists are machine-specific and symbol files are large; roaming them is wasteful.
- The system temporary folder for symbols: may be cleaned at any time and is shared with other applications.
Decision: `%LOCALAPPDATA%\flagscope\lists\<build>.txt` for saved lists, in the mach2 format, and `%LOCALAPPDATA%\flagscope\symbols\` for downloaded symbol files. The location is read from the environment with the standard library.
Consequences: no extra dependency. Deleting the folder resets the tool. The user can read and share saved lists directly. Each scan stores its symbol files in its own `symbols\scan-<process ID>` subfolder, so two scans running at the same time never share files (found during HU-05 verification on 2026-11-26, when two concurrent scans failed with a sharing violation).
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

## AD-11: Accept GPL-3.0-compatible dependencies in Rust, C, or C++ that build with the standard Windows Rust toolchain

Date: 2026-11-26
Context: while implementing HU-01, `ms-pdb` (AD-05) brought in `zstd-sys`, which compiles bundled C code, breaking AD-04. The user stated that adding a C compiler is acceptable: the project is not limited to one language, although Rust alone would be simpler.
Alternatives considered:
- Keep AD-04 and switch to the `pdb` crate (pure Rust): it reads the current PDB format, but has had no release since 2022 and does not support Microsoft's newer compressed PDB format.
- Keep AD-04 and write an own PDB reader: avoids the dependency, but repeats format work `ms-pdb` already does.
Decision: dependencies must have a licence compatible with GPL-3.0-only (MIT, Apache-2.0, BSD, MPL-2.0, Zlib, and similar) and be maintained. They may be written in Rust, C, or C++, provided they build with the toolchain the Rust MSVC target already requires (Visual Studio Build Tools with the C++ workload), with no other installation. Stated by the user.
Consequences: contributors need the Visual Studio C++ build tools, which the MSVC Rust target needs anyway; `README.md` must say so. Builds take longer when C code is compiled. `ms-pdb` stays as the PDB reader (AD-05). Every new dependency is still checked against the licence and maintenance rule.
Status: active

## AD-12: Take feature names from PDB public symbols and IDs from the feature descriptor in the binary

Date: 2026-11-26
Context: AD-08 assumed that each feature's ID is a constant (`::id`) in the PDB, as mach2 read it through DIA. During HU-01, the public `shell32.pdb` of build 26300.10075 had no constant records and no type records: Microsoft strips them from public PDBs. It did contain one public symbol `Feature_<Name>__descriptor` per feature, pointing into the `.rdata` section of `shell32.dll`, and the 32-bit little-endian value at offset 8 of that structure is the feature ID. Evidence: `Feature_57437827__descriptor` holds 57437827 and `Feature_63092242__descriptor` holds 63092242; of the 261 features extracted, the 40 whose names also appear in mach2's `26100.2314_WNC` list have identical IDs, with no differences.
Alternatives considered:
- PDB constants (AD-08): not available in public PDBs.
- Disassembling the getter functions in the binary, mach2's fallback: works without descriptors, but needs a disassembler and depends on the compiler's instruction choices.
Decision: the feature name is the text between `Feature_` and `__descriptor` in a public symbol of the PDB. The ID is the `u32` (little-endian) at offset 8 of the descriptor, read from the binary at the symbol's section and offset. Approved by the user.
Consequences: the scanner needs both the PDB and the binary, which it already has. The scan depends on the descriptor layout Microsoft's feature-staging library uses; HU-02 checks it across many binaries, and features with a name but no readable descriptor are counted and reported, not dropped silently. Disassembly is not implemented.
Status: active

## AD-13: Scan only x64 system binaries whose bytes reference the feature-staging API

Date: 2026-11-26
Context: HU-02 scans the installed build. On build 26300.10075, `%SystemRoot%\System32` and `%SystemRoot%\SystemApps` hold 6,358 `.dll` and `.exe` files, 5,870 of them x64 with a PDB reference. Downloading every PDB would take several gigabytes and most binaries have no features.
Alternatives considered:
- Every x64 binary with a PDB reference: complete by construction, but more than twice the downloads.
- Only binaries whose import table names the feature-staging API: cheap, but it missed `shell32.dll` (261 features), because those calls are not all in the regular import table.
- A fixed list of known binaries: small, but goes stale with every build.
Decision: The scan covers x64 PE files with a CodeView record under `%SystemRoot%\System32` and `%SystemRoot%\SystemApps` (recursively, without following links) and the executables directly in `%SystemRoot%` (for example `explorer.exe`). Of those, it downloads PDBs only for binaries whose bytes contain `featurestaging` (ignoring case) or `RtlNotifyFeatureUsage`. Evidence on 2026-11-26: 2,483 of 5,870 binaries match, including `shell32.dll`; in a sample of 25 matching binaries, 20 had features; in a sample of 25 non-matching binaries, the 20 that had a public PDB had no features, and 5 third-party binaries had no public PDB.
Consequences: The filter rests on a sample, not on a proof: a binary that uses features without either string would be missed. Binaries whose PDB is not on the server (HTTP 404) are skipped and counted. The sample predicted about 3.2 GB for a full scan; the first full scan (HU-02, 2026-11-26) downloaded 7,952 MB from 2,413 PDBs in 54 minutes, so the sample underestimated the average size.
Status: active

## AD-14: Merge features found in several binaries and report conflicting IDs

Date: 2026-11-26
Context: the same feature is often compiled into several binaries, so a build-wide scan finds the same name many times. CA-02.2 requires a list without duplicate names.
Alternatives considered:
- Keep one entry per binary: duplicates names, breaking CA-02.2 and the mach2 format.
- Keep the first ID found: hides conflicts and depends on scan order.
- Keep all IDs under renamed entries, as mach2 did (`|mach2.warning.duplicate<N>`): keeps data, but invents names.
Decision: Entries with the same name and the same ID become one entry. A name found with different IDs is left out of the list and reported as a conflict, with every ID and the binaries it came from.
Consequences: The list never contains a guessed ID. The user sees how many conflicts there were and can inspect them. If conflicts turn out to be frequent, this decision is revisited.
Status: active

## AD-15: Parse command-line arguments with `clap`

Date: 2026-11-26
Context: `flagscope-cli` needs a `scan` command now, and HU-03 and HU-05 add options (output file, overwrite, rescan, keep symbol files).
Alternatives considered:
- Parsing `std::env::args` by hand: no dependency, but help text, errors, and option validation all become own code.
- `argh` or `lexopt`: smaller, but less common among contributors and with fewer features for help and validation.
Decision: `clap` 4 (MIT OR Apache-2.0) with its derive API, only in `flagscope-cli`.
Consequences: Help text and argument errors come from `clap` in English; when the user-interface translation layer arrives, they are reviewed against RNF-02. Library crates never depend on `clap`.
Status: active

## AD-17: `flagscope scan` uses the saved list by default; `--keep-symbol-files` implies a new scan

Date: 2026-11-26
Context: HU-05 adds saved lists (RF-07) and the option to keep symbol files (RF-12.2). With a saved list, `flagscope scan` normally downloads nothing, so an option to keep downloaded files would have no effect.
Alternatives considered:
- A separate command for loading a saved list (`flagscope list`) and `scan` always scanning: clearer names, but the common case ("give me the list") would need the user to know whether a list exists.
- Reject `--keep-symbol-files` without `--rescan`: explicit, but makes the user repeat an intention the first option already states.
Decision: `flagscope scan` returns the saved list of the installed build when it exists and scans otherwise; `--rescan` forces a scan; `--keep-symbol-files` keeps the downloaded files and implies `--rescan`. A saved list is written to a temporary file and renamed, so no partial list is ever left (RF-06.3).
Consequences: the command name says "scan" even when it only loads a list; the messages say which happened. When published lists arrive (RF-14), the same command gains a third source, and this decision is revisited.
Status: active
