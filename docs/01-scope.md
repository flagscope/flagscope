> Status: approved
> Last updated: 2026-11-27
> Mode: new-project

# Scope

## In scope

- Read the state of one feature ID, and of all IDs configured on the machine.
- Enable, disable, and reset to default one or more feature IDs.
- Take a snapshot of the affected feature states before every change, and revert to a snapshot.
- Load `Name: ID` lists in the mach2 format for earlier builds.
- Scan the PDBs of the installed build to produce its feature list, and cache it by build number.
- Search features by name or by ID.
- Command-line output of the feature list: written to a file in the mach2 format and printed to the console. This is how the first version delivers its result. (added 2026-11-26, stated by the user)
- Delete downloaded symbol files once the feature list is saved, with an option to keep them. (added 2026-11-26, stated by the user)
- Estimated download size and duration before a scan, and progress with time remaining while it runs. (added 2026-11-26, stated by the user)
- Automated unit, integration, and end-to-end tests. (added 2026-11-26, stated by the user)
- Published feature lists: by default, the tool downloads the list of the installed build from the public data repository `flagscope/feature-lists` instead of scanning. When no list exists for the exact build, it offers the list of the nearest build with a warning. (added 2026-11-26, stated by the user)
- Contributing lists: anyone who scans a build can submit its list to `flagscope/feature-lists` through a pull request; the tool writes the file in that repository's format, and the person submits it. (added 2026-11-26, stated by the user)
- Automated format check of contributed lists: every pull request to `flagscope/feature-lists` that adds or changes a list is checked automatically for the AD-16 format before a human reviews it. (added 2026-11-27, stated by the user)
- Three scan modes when a scan is needed (added 2026-11-26, stated by the user):
  - Light (default): downloads only the parts of each symbol file the scanner reads, each identical symbol file once, and after a cumulative update only the symbol files that changed.
  - Full: downloads complete symbol files; required to keep them for other tools.
  - Targeted: scans only the binaries the user names.
- GPUI user interface on top of everything above, built last, in English, with all user-facing strings ready for translation.
- Release binaries for Windows 11 x64 through GitHub Releases.

## Delivery order

The scope is delivered as eight features, in this order. Features 1 to 5 have full documents; all their stories are verified except HU-08 (`published-lists`), which waits for the next cumulative update (2026-11-27). Features 6 to 8 are only named. The next session starts `inspect-features` at Phase 3: propose its slicing, then write its requirements, stories, and plan.

1. `find-feature-ids`: the installed build's feature list (first version). See `docs/features/find-feature-ids/`.
2. `lighter-scans`: light and targeted scan modes, and reuse of unchanged symbol-file results across scans. (added 2026-11-26; moved before `published-lists` on 2026-11-26, because it changes the scan pipeline again and the full-scan verification of `find-feature-ids` is done once, after it)
3. `published-lists`: download the list of the installed build (or the nearest build) from `flagscope/feature-lists`, and prepare lists for contribution. (added 2026-11-26)
4. `contributor-quality-gates`: continuous integration for formatting, linting, tests by level, DCO sign-off, and secret scanning.
5. `list-format-check`: the automated format check of pull requests to `flagscope/feature-lists`. (added 2026-11-27)
6. `inspect-features`: read feature states, import mach2 lists, and fuzzy search.
7. `toggle-features-safely`: change feature states with snapshots, elevation, and revert.
8. `desktop-app`: GPUI user interface.

## Out of scope

- Automated end-to-end tests that change feature states on a contributor's machine or on a CI runner: state-changing tests run only in a disposable environment (for example Windows Sandbox or a virtual machine). (added 2026-11-26)
- A fixed time limit for scanning: the duration depends on Microsoft's symbol server and the user's connection. (added 2026-11-26, stated by the user)
- Generating lists in continuous integration: GitHub-hosted runners run Windows Server, not the Windows 11 builds users have, and obtaining each build's binaries would mean downloading full builds from unofficial sources. Lists come from people who scan their own builds. (added 2026-11-26)
- Submitting lists automatically from the tool: the person who scanned opens the pull request. (added 2026-11-26)
- Operating systems other than Windows: the feature APIs exist only in `ntdll.dll`.
- Windows 10: the user set Windows 11 as the minimum.
- ARM64 and x86: deferred; x64 only for now.
- winget and other package managers: deferred; GitHub Releases only for now.
- Translations other than English: deferred, but the code must make them possible without restructuring.
- A network service: the logic lives in library crates linked into the executable. The published lists are static files in a GitHub repository.
- ~~A public catalogue of per-build lists~~: moved into scope on 2026-11-26 as "Published feature lists", after the first full scan downloaded 7.95 GB.

## First version

The first version is done when it **finds the IDs**: it produces the `Name: ID` list of the installed build. (stated by the user)

## Assumptions

- Microsoft's public PDBs contain each feature's name in a public symbol, and its ID in the binary's feature descriptor. Observed on build 26300.10075 (HU-01, HU-02; AD-12).
- The mach2 lists can be reused as data. mach2 and ViVe are both GPL-3.0 (checked through the GitHub API on 2026-11-26). This code does not copy theirs; it only reads their list files. Not reviewed by anyone with legal knowledge.
- Publishing feature lists (names and IDs derived from Microsoft's public symbols) is acceptable, as mach2 did for years. Not reviewed by anyone with legal knowledge.
- GPUI is licensed under Apache-2.0, which is compatible with GPL-3.0. Not yet checked against the current GPUI repository. [inferred]
- `RtlQueryAllFeatureConfigurations` and `RtlSetFeatureConfigurations` are still available in `ntdll.dll` on current builds.
- Reading feature states does not require administrator rights; changing them does. Not yet observed. [inferred]
- Scanning a build for the first time needs an Internet connection, because the symbol files are downloaded from Microsoft's public symbol server. Reusing a saved list and importing mach2 lists work offline. [inferred]
- Microsoft's symbol server answers HTTP range requests, and the scanner gets identical results from symbol files that hold only the parts it reads. Observed on 2026-11-26 for `shell32`, `twinui`, `Windows.UI.Xaml`, and `explorer`; a light scan of build 26300.10075 is measured at about 3.1 GB instead of 7.7 GB.

## Open questions

- Licence of the data in `flagscope/feature-lists`: resolved on 2026-11-26, CC0-1.0 (AD-16).
- Format and folder layout of the published lists: resolved on 2026-11-26 (AD-16). How a contributed list is checked before merging is settled in the `published-lists` feature. (assistant, with the user's approval)
- Whether GPUI builds and runs on Windows and has a published release. Its standalone repository only says "See you in a few weeks". To be checked before the user-interface story. (assistant)
- `README.md` is still a placeholder. Before the first release it must describe the project and the build prerequisites, including the Visual Studio C++ build tools (AD-11). (assistant)
- Review rule with a single maintainer: resolved on 2026-11-27, self-review until there are two maintainers (`CONTRIBUTING.md`, "Review and merging").
- DCO and merge commits: resolved on 2026-11-27, merge commits created by GitHub are exempt from the sign-off (`CONTRIBUTING.md`, "License and sign-off").
- Automated format check for `flagscope/feature-lists` pull requests: moved into scope on 2026-11-27 (RF-19, feature `list-format-check`). Whether the check must pass before a list can be merged is still open. (assistant)
- Code conventions: resolved in `docs/03-architecture.md` ("Conventions") on 2026-11-26.
