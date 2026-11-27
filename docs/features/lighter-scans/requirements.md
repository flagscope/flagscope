> Status: approved
> Last updated: 2026-11-26
> Mode: new-project

# Requirements: lighter-scans

Feature for the advanced user who has to scan, usually because no published list exists for the build yet: download less, reuse earlier work, and scan only what is needed.

## Functional

### RF-16.1: Light mode by default

Refines: RF-16
Without a mode option, a scan must download only the parts of each symbol file the scanner reads, and must produce the same list as a full scan of the same build.

### RF-16.2: Each identical symbol file once

Refines: RF-16
A scan must download each distinct symbol file (same name, GUID, and age) at most once, even when several binaries reference it.

### RF-16.3: Full mode

Refines: RF-16, RF-12
With the full-mode option, a scan must download complete symbol files. Keeping symbol files is only possible in full mode: asking to keep them selects full mode.

### RF-16.4: Targeted mode

Refines: RF-16
With the targeted-mode option, a scan must read only the binaries the user names, state that the result covers only those binaries, and must not save it as the list of the build.

### RF-17.1: Reuse results of identical symbol files

Refines: RF-17
The system must keep the features read from each distinct symbol file, and a later scan must use them instead of downloading that symbol file again.

## Non-functional

### RNF-08.2: Disk use reported correctly

Refines: RNF-08
The peak disk space a scan reports must count each file on disk once.
