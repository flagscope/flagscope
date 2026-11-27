> Status: approved
> Last updated: 2026-11-26
> Mode: new-project

# User stories: published-lists

## HU-06: Prepare my scanned list for contribution

Covers: RF-18.1, RF-18.2, RNF-01.2

**Story.** As a contributor, I want flagscope to write my scanned list in the layout and format of `flagscope/feature-lists`, so that I can submit it through a pull request without editing it.

**Acceptance criteria.**

1. CA-06.1: After a scan of build 26300.10075, the contribution command given a directory writes `x64/26300/26300.10075.txt` under that directory.
2. CA-06.2: That file starts with the `#` header lines of AD-16 (list title, build, architecture, generator with flagscope version and scan mode, scan date) and continues with the same `Name: ID` lines as the saved list, sorted by name.
3. CA-06.3: When the local list of the installed build did not come from a scan on this machine, the contribution command writes nothing and prints why.

**Status.** verified

## HU-07: Get the list of my build without scanning

Covers: RF-14.1, RF-14.2, RF-14.3, RF-14.4, RNF-01.2

**Story.** As an advanced user with a slow connection, I want flagscope to download the published list of my build, so that I can find features without downloading gigabytes of symbol files.

**Acceptance criteria.**

1. CA-07.1: With no local list and a list for the installed build in `flagscope/feature-lists`, `flagscope scan` prints that list, states that it came from `flagscope/feature-lists`, and downloads no symbol file.
2. CA-07.2: The next run, with the network disconnected, prints the same list and states that it came from the local saved list.
3. CA-07.3: With a local list present, `flagscope scan` states that it used the local list and does not contact `flagscope/feature-lists`.
4. CA-07.4: With the network disconnected and no local list, the tool prints a message naming `flagscope/feature-lists` as unreachable, exits with a non-zero exit code, and leaves no list behind.

**Status.** in progress

## HU-08: Use the nearest published build

Covers: RF-15.1, RF-15.2, RNF-01.2

**Story.** As an advanced user whose exact build has no published list yet, I want to be offered the list of the closest build, so that I can choose between an almost-right list now and a scan.

**Acceptance criteria.**

1. CA-08.1: With no list for the installed build but a list for another revision of the same major build, `flagscope scan` names that build, states that it is not the installed build, names the option that uses it and the option that scans, and downloads neither the list nor any symbol file.
2. CA-08.2: With the option that uses the nearest build, the tool prints that build's list and states, before and after the list, the build it belongs to.
3. CA-08.3: After CA-08.2, the next run does not treat that list as the list of the installed build.
4. CA-08.4: With no published list for the installed major build, the tool does not offer lists of other major builds.

**Status.** pending
