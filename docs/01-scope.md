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
- Command-line output of the feature list: written to a file in the mach2 format and printed to the console. This is how the first version delivers its result. (added 2026-11-26, stated by the user)
- Delete downloaded symbol files once the feature list is saved, with an option to keep them. (added 2026-11-26, stated by the user)
- Estimated download size and duration before a scan, and progress with time remaining while it runs. (added 2026-11-26, stated by the user)
- Automated unit, integration, and end-to-end tests. (added 2026-11-26, stated by the user)
- GPUI user interface on top of everything above, built last, in English, with all user-facing strings ready for translation.
- Release binaries for Windows 11 x64 through GitHub Releases.

## Delivery order

The scope is delivered as five features, in this order. Only the first has full documents so far.

1. `find-feature-ids`: the installed build's feature list (first version). See `docs/features/find-feature-ids/`.
2. `contributor-quality-gates`: continuous integration for formatting, linting, tests by level, DCO sign-off, and secret scanning.
3. `inspect-features`: read feature states, import mach2 lists, and fuzzy search.
4. `toggle-features-safely`: change feature states with snapshots, elevation, and revert.
5. `desktop-app`: GPUI user interface.

## Out of scope

- Automated end-to-end tests that change feature states on a contributor's machine or on a CI runner: state-changing tests run only in a disposable environment (for example Windows Sandbox or a virtual machine). (added 2026-11-26)
- A fixed time limit for scanning: the duration depends on Microsoft's symbol server and the user's connection. (added 2026-11-26, stated by the user)
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
- Scanning a build for the first time needs an Internet connection, because the symbol files are downloaded from Microsoft's public symbol server. Reusing a saved list and importing mach2 lists work offline. [inferred]

## Open questions

- Whether GPUI builds and runs on Windows and has a published release. Its standalone repository only says "See you in a few weeks". To be checked before the user-interface story. (assistant)
- `README.md` is still a placeholder. Before the first release it must describe the project and the build prerequisites, including the Visual Studio C++ build tools (AD-11). (assistant)
- Code conventions: resolved in `docs/03-architecture.md` ("Conventions") on 2026-11-26.
