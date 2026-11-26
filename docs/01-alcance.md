> Status: approved
> Last updated: 2026-11-26
> Mode: new-project

# Scope

## In scope

- Read the state of one feature ID, and of all IDs configured on the machine.
- Enable, disable, and reset to default one or more feature IDs.
- Take a snapshot of the affected feature states before every change, and revert to a snapshot.
- Load `Name: ID` lists in the mach2 format for earlier builds.
- Scan the PDBs of the installed build to produce its feature list, and cache it by build number.
- Search features by name or by ID.
- GPUI user interface on top of everything above, built last, in English, with all user-facing strings ready for translation.
- Release binaries for Windows 11 x64 through GitHub Releases.

## Out of scope

- Operating systems other than Windows: the feature APIs exist only in `ntdll.dll`.
- Windows 10: the user set Windows 11 as the minimum.
- ARM64 and x86: deferred; x64 only for now.
- winget and other package managers: deferred; GitHub Releases only for now.
- Translations other than English: deferred, but the code must make them possible without restructuring.
- A network service: the logic lives in library crates linked into the executable.
- A public catalogue of per-build lists: the tool produces lists locally. Publishing them can be considered later.

## First version

The first version is done when it **finds the IDs**: it produces the `Name: ID` list of the installed build. (stated by the user)

## Assumptions

- Microsoft's public PDBs still contain enough information to extract each feature's name and ID. Not yet observed on a real PDB; this is the main project risk.
- The mach2 lists can be reused as data. mach2 and ViVe are both GPL-3.0 (checked through the GitHub API on 2026-11-26). This code does not copy theirs; it only reads their list files. Not reviewed by anyone with legal knowledge.
- GPUI is licensed under Apache-2.0, which is compatible with GPL-3.0. Not yet checked against the current GPUI repository. [inferred]
- `RtlQueryAllFeatureConfigurations` and `RtlSetFeatureConfigurations` are still available in `ntdll.dll` on current builds.
- Reading feature states does not require administrator rights; changing them does. Not yet observed. [inferred]

## Open questions

- Whether GPUI builds and runs on Windows and has a published release. Its standalone repository only says "See you in a few weeks". To be checked before the user-interface story. (assistant)
- Code conventions, to be fixed in Phase 4. Candidates found during research:
  - Virtual workspace with a flat `crates/*` layout; `[workspace.dependencies]`, `[workspace.package]`, and `[workspace.lints]` inherited by every crate.
  - `rustfmt` with default settings; `clippy` with `-D warnings`.
  - Documentation lints: `missing_docs`, `clippy::missing_safety_doc`, `missing_errors_doc`, `missing_panics_doc`, and `undocumented_unsafe_blocks`. Every `unsafe` block calling `ntdll` carries a `// SAFETY:` comment.
  - Rustdoc in the style of the Rust API Guidelines, with `# Errors`, `# Safety`, and `# Panics` sections.
  - ISO references: ISO/IEC 25010:2023 for quality attributes (maintainability: modularity, reusability, analysability, modifiability, testability) and ISO/IEC/IEEE 29148 for writing verifiable requirements.
  - Conventional Commits carrying the story ID.
  - Branch protection on the default branch: at least one approving review from a maintainer, with AI review as an advisory check only.
