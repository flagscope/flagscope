> Status: approved
> Last updated: 2026-11-26
> Mode: new-project

# User stories: lighter-scans

## HU-09: Scan with less download by default

Covers: RF-16.1, RF-16.2, RNF-08.2

**Story.** As an advanced user with a slow connection, I want a scan to download only what it needs, so that scanning a build costs a fraction of the full symbol files.

**Acceptance criteria.**

1. CA-09.1: A scan without a mode option reports a downloaded size smaller than the size a full-mode scan of the same build reports.
2. CA-09.2: The list it produces is identical, line for line, to the list a full-mode scan of the same build produces.
3. CA-09.3: The number of symbol files it reports as downloaded equals the number of distinct symbol files among the binaries it scanned.
4. CA-09.4: With symbol files kept, the peak disk space the scan reports equals the total size of the files left on disk.

**Status.** verified

## HU-10: Keep complete symbol files

Covers: RF-16.3

**Story.** As an advanced user who also uses a debugger, I want to keep complete symbol files when I ask for it, so that other tools can use them.

**Acceptance criteria.**

1. CA-10.1: With the full-mode option, every symbol file the scan downloads has the size the server reports for it.
2. CA-10.2: With the keep option and no mode option, the scan states that it uses full mode, and every kept file has the size the server reports for it.

**Status.** verified

## HU-11: Scan only the binaries I name

Covers: RF-16.4

**Story.** As an advanced user who knows where a feature lives, I want to scan only that binary, so that I get its features in seconds.

**Acceptance criteria.**

1. CA-11.1: A targeted scan of `explorer.exe` prints the features of that binary, states that the list covers only the named binaries, and downloads only that binary's symbol file.
2. CA-11.2: After a targeted scan, the saved list of the build is unchanged.
3. CA-11.3: A targeted scan of a name that matches no binary prints which name matched nothing and exits with a non-zero exit code.

**Status.** verified

## HU-12: Reuse what earlier scans read

Covers: RF-17.1

**Story.** As an advanced user, I want a new scan to skip the symbol files an earlier scan already read, so that after a cumulative update only the changed binaries are downloaded.

**Acceptance criteria.**

1. CA-12.1: A second scan of the same build with the rescan option downloads no symbol file, and produces a list identical to the first.
2. CA-12.2: A scan reports how many symbol files it took from earlier scans and how many it downloaded.

**Status.** verified
