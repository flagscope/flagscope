> Status: approved
> Last updated: 2026-11-26
> Mode: new-project

# Requirements: find-feature-ids

Feature for the advanced user: obtain the `Name: ID` list of the installed Windows build. This is the first version of the product.

## Functional

### RF-06.1: List for the running build

Refines: RF-06
The system must produce the feature list for the build reported by the running system, and show that build number with the result.

### RF-06.2: Names match the feature store

Refines: RF-06
Each entry must pair a feature name with the same ID that the Windows feature configuration store uses for that feature.

### RF-06.3: Failure without a connection

Refines: RF-06
When symbol files cannot be downloaded and no saved list exists for the build, the system must report the cause and must not save a partial list.

### RF-07.1: Reuse without downloading

Refines: RF-07
When a list for the running build has already been saved, the system must return it without downloading any symbol file.

### RF-07.2: Forced rescan

Refines: RF-07
The system must produce the list again, ignoring the saved one, when the user asks for a new scan.

### RF-11.1: File output

Refines: RF-11
The system must write the list to a file path the user gives, one `Name: ID` entry per line.

### RF-11.2: Console output

Refines: RF-11
The system must print the same entries to the console.

### RF-12.1: Delete symbol files by default

Refines: RF-12
After the list is saved, the system must delete the symbol files it downloaded for it.

### RF-12.2: Keep symbol files on request

Refines: RF-12
When the user sets the option to keep them, the system must leave the downloaded symbol files on disk.

### RF-13.1: Estimate before scanning

Refines: RF-13
Before downloading, the system must show the estimated total download size and the estimated duration.

### RF-13.2: Progress while scanning

Refines: RF-13
While the scan runs, the system must show its progress and the estimated time remaining, recalculated from the measured download speed.

## Non-functional

### RNF-01.1: Runs on the supported platform

Refines: RNF-01
Every requirement in this feature must work on Windows 11 x64, build 22000 or later.

### RNF-04.1: Scanning without administrator rights

Refines: RNF-04
Scanning, downloading, and parsing must run in a process without administrator rights.

### RNF-08.1: Disk usage is measured

Refines: RNF-08
Each scan must report the total size of the symbol files it downloaded and the peak additional disk space it used, so that the user can set the RNF-08 threshold.
