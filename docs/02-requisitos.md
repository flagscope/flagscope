> Status: approved
> Last updated: 2026-11-26
> Mode: new-project

# Requirements

## Functional

### RF-01: Read the state of one feature

The system must show the current state (default, enabled, or disabled) of a feature ID the user provides.
Source: scope, "Read the state of one feature ID".

### RF-02: List the configured features

The system must list every feature ID configured on the machine, with its state.
Source: scope, "and of all IDs configured on the machine".

### RF-03: Change feature states

The system must enable, disable, or reset to default one or more feature IDs in a single operation.
Source: scope, "Enable, disable, and reset to default one or more feature IDs".

### RF-04: Snapshot before every change

Before any operation that changes feature states, the system must record the previous state of every affected feature ID. The operation must not run if the snapshot could not be recorded.
Source: user statement, "every action that changes system state records the previous state first".

### RF-05: Revert to a snapshot

The system must restore the feature states recorded in a snapshot the user selects.
Source: user statement, "so that it can be reverted".

### RF-06: Find the feature IDs of the installed build

The system must produce the feature list (`Name: ID`) of the installed Windows build from Microsoft's public debug symbols.
Source: user statement, first version "finds the IDs".

### RF-07: Reuse a list already produced

The system must keep the feature list of each build it has produced and reuse it for that build without producing it again, unless the user asks for a new scan.
Source: scope, "cache it by build number".

### RF-08: Import mach2 feature lists

The system must load feature lists in the mach2 format (`Name: ID`, one entry per line) for builds the system has not scanned.
Source: scope, "Load `Name: ID` lists in the mach2 format for earlier builds".

### RF-09: Search features

The system must find features by name or by exact ID within the loaded feature list. Name search is fuzzy: it ignores letter case, accepts the characters of the query in order even when other characters lie between them, and ranks the results from best to worst match.
Source: scope, "Search features by name or by ID"; fuzzy search confirmed by the user.

### RF-11: Output the feature list without the graphical user interface

The system must write the feature list produced by RF-06 to a file in the mach2 format (`Name: ID`, one entry per line) and print it to the console, from the command line.
Source: user statement, accepted proposal for the first version before the graphical user interface exists.

### RF-12: Symbol file retention

After a feature list has been saved, the system must delete the symbol files it downloaded to produce it, unless the user has set the option to keep them. Deleting is the default.
Source: user statement, "configurable, but by default delete them after saving the list".

### RF-13: Scan estimate

Before a scan starts, the system must show the estimated total download size and the estimated duration. While the scan runs, it must show its progress and the estimated time remaining, recalculated from the download speed measured so far.
Source: user statement, the duration depends on the server and the user's connection, so there is no generic limit, but there can be an estimate.

### RF-10: Graphical user interface

The system must offer every capability in RF-01 to RF-09, RF-12, and RF-13 through a graphical user interface.
Source: user statement, GPUI user interface built last.

## Non-functional

### RNF-01: Supported platform

The system must run on Windows 11 x64, measured as: every functional requirement works on Windows 11 build 22000 or later on x64.
Source: user statement, minimum OS Windows 11, x64 only for now.

### RNF-02: Translatable user interface

The user interface must be ready for translation, measured as: 100% of user-facing strings come from translation resources, and adding a language requires no change outside those resources.
Source: user statement, English first and easy to translate later.

### RNF-03: Reversibility

Every change to system state must be reversible, measured as: 100% of state-changing operations have a snapshot recorded before they run (RF-04).
Source: user statement, "casi una regla para cada acción".

### RNF-04: Least privilege

Only the operation that changes feature states may run with administrator rights, measured as: reading, scanning, downloading, and parsing run in a process without administrator rights.
Source: privilege model chosen by the user among the alternatives offered.

### RNF-05: Maintainability

The code must be modular, reusable, formatted, and documented, measured as:
- The library crates build and pass their tests without the user-interface crate.
- The formatter reports zero differences, and the linter reports zero warnings, with the configuration recorded in the architecture document (Phase 4).
- 100% of public items in the library crates have documentation comments.

Source: user statement, clean, modular, reusable structure, formatted code, and clean comments.

### RNF-06: No sensitive data in the repository

The repository must not contain sensitive data, measured as: secret scanning reports zero findings on every push and pull request.
Source: user statement, no sensitive data is ever committed.

### RNF-07: Test coverage by level

The system must have automated tests, measured as: every pull request runs the unit and integration tests of the library crates and the end-to-end tests of the command-line interface, and none of them fails.
Source: user statement, unit, integration, and end-to-end tests.

### RNF-08: Disk usage

No stated target. The size of the symbol files is measured in the scanner proof of concept, and the user sets the threshold from that measurement. The estimate in RF-13 shows the expected size before every scan.
Source: user statement, the user rejected a 1 GB limit because the average size of the files is not known yet.
