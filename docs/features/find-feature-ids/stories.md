> Status: approved
> Last updated: 2026-11-26
> Mode: new-project

# User stories: find-feature-ids

## HU-01: Extract feature names and IDs from one symbol file

Covers: RF-06.2, RNF-08.1

**Story.** As an advanced user, I want the tool to read feature names and IDs from one of Microsoft's public symbol files, so that I know hidden features can be found by name on current builds.

This story is the proof of concept for the main project risk. If CA-01.2 fails, work stops and the approach is reconsidered with the user before any other story starts.

**Acceptance criteria.**

1. CA-01.1: Given one system binary of the installed build, the tool downloads its symbol file from Microsoft's public symbol server and prints at least one `Name: ID` entry.
2. CA-01.2: At least three printed entries have the same name and the same ID as the same feature in a mach2 list for build 26100.
3. CA-01.3: The run prints the size of the symbol file it downloaded.
4. CA-01.4: The formatter reports zero differences and the linter reports zero warnings on the code of this story.

**Status.** verified

## HU-02: Scan the installed build

Covers: RF-06.1, RF-06.3, RNF-01.1, RNF-04.1

**Story.** As an advanced user, I want to scan my installed Windows build, so that I get the list of every feature it contains.

**Acceptance criteria.**

1. CA-02.1: On Windows 11 x64, a scan produces a feature list and shows a build number equal to the one `winver` shows on the same machine.
2. CA-02.2: The list contains no duplicate feature name.
3. CA-02.3: The scan completes when started from a console that is not running as administrator.
4. CA-02.4: With the network disconnected and no saved list for the build, the tool prints a message naming the missing connection, exits with a non-zero exit code, and leaves no list file behind.

**Status.** in progress

## HU-03: Get the list as a file and on the console

Covers: RF-11.1, RF-11.2

**Story.** As an advanced user, I want the list written to a file and printed to the console, so that I can read it, search it, and share it before the graphical interface exists.

**Acceptance criteria.**

1. CA-03.1: The tool writes the list to the file path given on the command line, and every line of that file has the form `Name: ID`.
2. CA-03.2: The entries printed to the console are the same as the entries in the file, in the same order.
3. CA-03.3: When the given file already exists, the tool does not overwrite it unless the user passes the explicit overwrite option. (proposed by the assistant, accepted with the approval of this feature)

**Status.** in progress

## HU-04: See the estimate and the progress of a scan

Covers: RF-13.1, RF-13.2

**Story.** As an advanced user, I want to see how much will be downloaded and how long it may take, so that I can decide when to scan and know that it is still working.

**Acceptance criteria.**

1. CA-04.1: Before the first download starts, the tool prints the estimated total download size and the estimated duration.
2. CA-04.2: While the scan runs, the tool shows the progress as a percentage and an estimated time remaining, and both values change as the scan advances.
3. CA-04.3: When the scan ends, the tool prints the actual total download size and the actual duration, so that they can be compared with the estimate.

**Status.** in progress

## HU-05: Reuse saved lists and clean up symbol files

Covers: RF-07.1, RF-07.2, RF-12.1, RF-12.2, RNF-08.1

**Story.** As an advanced user, I want a build I already scanned to load instantly and the downloaded files to be removed, so that I do not repeat downloads or lose disk space.

**Acceptance criteria.**

1. CA-05.1: A second run on the same build returns the list, states that it came from the saved list, and downloads no symbol file.
2. CA-05.2: With the rescan option, the tool downloads the symbol files again and saves a new list.
3. CA-05.3: After a scan with default options, none of the symbol files downloaded by that scan remains on disk.
4. CA-05.4: After a scan with the keep option, every symbol file downloaded by that scan remains on disk.
5. CA-05.5: Every scan prints the peak additional disk space it used.

**Status.** in progress
