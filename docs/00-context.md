> Status: approved
> Last updated: 2026-11-26
> Mode: new-project

# Context

## Problem

Windows ships hidden features that are enabled or disabled by numeric ID (feature IDs). The reference tool, ViVeTool, is command-line only and does not know which name belongs to which ID. The graphical tools that existed took the names from per-build lists published by mach2. That repository was archived on 2024-12-14 and has no lists past build 26100.2314. The best-known GUI (PeterStrick/ViVeTool-GUI, VB.NET) was archived in December 2025 and declared unmaintained by its author. Today no maintained tool shows the features of a current Windows build by name and lets the user toggle them.

## Users

- Advanced Windows user / Windows Insider: wants to find hidden features by name, see their state, and toggle them without using the command line. [inferred]
- Contributor: wants to read, build, and improve the code through pull requests. (stated by the user)

## Constraints

- Language: Rust. (stated by the user)
- User interface: GPUI, the Zed framework, built last, once the logic works. (stated by the user)
- One repository (a Cargo workspace) that keeps the logic and the user interface in separate crates. (stated by the user)
- The code must be able to grow: clean, modular structure with reusable code, formatted code, and clean comments. ISO standards are followed where they apply. (stated by the user)
- Precise technical terminology throughout the repository. (stated by the user)
- Repository language: English for code, comments, documentation, commits, and pull requests. (stated by the user)
- Application language: English first, designed so that it can later be translated into other languages. (stated by the user)
- Public repository, open to anyone who wants to use it or contribute: https://github.com/flagscope/flagscope, owned by the `flagscope` GitHub organization. Product, repository, and executable name: `flagscope`. (stated by the user)
- Published feature lists live in a separate public data repository, https://github.com/flagscope/feature-lists. (stated by the user, added 2026-11-26)
- License: a community open-source tool that nobody can redistribute as a closed product for sale. GPL-3.0-only for now (SPDX `GPL-3.0-only`), chosen among the alternatives offered; `GPL-3.0-or-later` may be adopted later. The `LICENSE` file is added when the repository is created. (stated by the user)
- Every commit carries a DCO sign-off. (chosen by the user among the alternatives offered)
- Every commit, push, and pull request is the responsibility of a human author, never under an AI identity. (stated by the user)
- Every pull request is reviewed and approved by a maintainer before merging. Automated AI review may run first, but it never replaces human approval. (stated by the user)
- While there is a single maintainer, the maintainer's own pull requests are merged after the maintainer reviews the diff; GitHub merge commits are exempt from the DCO sign-off. (chosen by the user among the alternatives offered, 2026-11-27)
- Privilege model: the application runs without administrator rights and relaunches itself elevated only to apply changes. (chosen by the user among the alternatives offered; recorded as a decision in Phase 4)
- Every action that changes system state records the previous state first, so that it can be reverted. (stated by the user)
- Automated tests at the unit, integration, and end-to-end levels, wherever each level applies. (stated by the user, added after Phase 1 approval)
- No sensitive data is ever committed, and neither is content that adds no value to the code. (stated by the user)
- Minimum supported OS: Windows 11. (stated by the user)
- Architecture: x64 only for now; other architectures later. (stated by the user)
- Distribution: GitHub Releases only for now; winget later. (stated by the user)
- No deadline. (stated by the user)

## Team

One lead maintainer. Contributions from the community are accepted through pull requests, and collaborators with repository access may also approve pull requests, under their responsibility. (stated by the user)

## Methodology

Chosen: Kanban with a work-in-progress limit of 1, plus XP practices: continuous integration running `fmt`, `clippy`, and tests, and small commits that carry their story ID.
Reason: one lead developer, no deadline, and no external client, so Scrum ceremonies would have nobody to synchronise. Community pull requests arrive unpredictably, which fits a flow model. [inferred]
Consequences: no sprints. Progress is measured by verified stories. Each story must be small enough to close in a few days. The scanner proof of concept comes before any user interface.

## Glossary

General software-engineering terms follow ISO/IEC/IEEE 24765:2017. Rust and Cargo terms follow The Cargo Book and The Rust Reference. Project terms:

- **Feature ID**: 32-bit number that identifies a feature in the Windows feature configuration store. It is the ID of the task in Microsoft's internal tracker.
- **Feature state**: the configuration the store holds for an ID: default, enabled, or disabled.
- **Feature list**: `Name: ID` mapping for one Windows build.
- **Build**: Windows version identified as `major.revision`, for example `26100.2314`.
- **PDB**: debug-symbol file published by Microsoft on its symbol server. It contains the feature names and IDs.
- **Scanner**: process that produces the feature list of a build from the PDBs of its binaries.
- **Snapshot**: record of the feature states taken before a change, used to revert it.
- **FFI**: calls from Rust into functions of `ntdll.dll`.
