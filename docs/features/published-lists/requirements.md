> Status: approved
> Last updated: 2026-11-26
> Mode: new-project

# Requirements: published-lists

Feature for the advanced user with a slow connection: get the feature list of the installed build from `flagscope/feature-lists` instead of scanning; and for the contributor: prepare a scanned list for that repository.

## Functional

### RF-14.1: Order of sources

Refines: RF-14
Without `--rescan`, the system must use, in this order: the list saved locally for the installed build; the published list for that build; a scan.

### RF-14.2: Published list is kept locally

Refines: RF-14
A published list the system downloads must be saved as the local list of that build, so that later runs need no network.

### RF-14.3: State the source

Refines: RF-14
The system must state whether the list came from the local saved list, from `flagscope/feature-lists`, or from a scan.

### RF-14.4: Unreachable repository

Refines: RF-14
When `flagscope/feature-lists` cannot be reached and no local list exists, the system must say so and must not save a list.

### RF-15.1: Nearest build

Refines: RF-15
The nearest published build is the published build with the same architecture and the same major build number whose revision is closest to the installed revision. Published builds with a different major build number are never offered.

### RF-15.2: Choice between nearest build and scan

Refines: RF-15
When only a nearest build exists, the system must name it, state that it is not the installed build, and neither download it nor scan until the user chooses one of the two through a command-line option.

### RF-18.1: Contribution file

Refines: RF-18
The system must write a scanned list to `<architecture>/<major>/<major>.<revision>.txt` under a directory the user gives, with the `#` header lines and the sorted `Name: ID` lines defined in AD-16.

### RF-18.2: Only scanned lists

Refines: RF-18
The system must refuse to prepare a contribution from a list that did not come from a scan on this machine (a published list or a nearest-build list).

## Non-functional

### RNF-01.2: Runs on the supported platform

Refines: RNF-01
Every requirement in this feature must work on Windows 11 x64, build 22000 or later.
