> Status: approved
> Last updated: 2026-11-26
> Mode: new-project

# Decisions: published-lists

## AD-16: Store published lists as one text file per build, grouped by architecture and major build

Date: 2026-11-26
Context: published lists (RF-14, RF-15, RF-18) live in the public repository `flagscope/feature-lists`. mach2 kept hundreds of list and patch files side by side in a few folders, which made them hard to browse. The user asked for the lists to be organised by build and the repository to stay simple and follow the same rules as `flagscope`.
Alternatives considered:
- One flat folder of files named by build (mach2 style): simple to write, hard to browse once there are hundreds of builds.
- A single index file listing every build: makes lookups one request, but every contribution edits the same file, so pull requests conflict.
- JSON or CSV files: easy for programs, but harder to read and to review line by line in a pull request, and not compatible with the mach2 format the tool already reads (RF-08).
Decision: the path of a list is `<architecture>/<major>/<major>.<revision>.txt`, for example `x64/26300/26300.10075.txt`. The file is UTF-8 text: a header of `#` comment lines (build, architecture, flagscope version, scan mode, scan date), then one `Name: ID` line per feature, sorted by name. The tool finds the exact list from the build alone; for the nearest build (RF-15) it lists the files in the `<architecture>/<major>/` folder. The data is licensed CC0-1.0. Chosen by the user among the alternatives offered.
Consequences: no index file to maintain and no merge conflicts between contributions to different builds. Adding ARM64 later adds an `arm64/` folder without moving files. Reviewers read a list as plain lines in the pull request diff. The `#` header is the only addition to the mach2 line format, so readers of this format must skip comment lines.
Status: active

## AD-22: Saved lists carry the AD-16 header and a local source mark; only lists scanned here can be contributed

Date: 2026-11-27
Context: HU-06 must refuse to contribute a list that did not come from a scan on this machine (RF-18.2). Saved lists written before HU-06 were plain `Name: ID` lines with no record of where they came from, and published lists (HU-07) will also be saved locally.
Alternatives considered:
- A separate metadata file next to each saved list: keeps the list format untouched, but two files can get out of step.
- Trust any saved list: simplest, but a downloaded list could be sent back to `flagscope/feature-lists` as if it were a new scan.
Decision: a saved list starts with the AD-16 header (title, build, architecture, generator with flagscope version and scan mode, UTC scan date), then `# Source: scanned on this machine` when it came from a scan here, then the `Name: ID` lines. `flagscope contribute <folder>` copies the header and the lines to `<folder>/<architecture>/<major>/<build>.txt`, without the source line, and refuses lists without that line or without a header; it never overwrites an existing file. Readers of saved lists skip `#` lines, so loading is unchanged.
Consequences: lists saved before HU-06 must be scanned again before they can be contributed (a rescan with stored results takes under a minute). Published lists saved by HU-07 will carry a different source line and stay non-contributable. The architecture is written as `x64`, the only one flagscope supports (RNF-01).
Status: active

## AD-23: Download published lists from the raw-file URL of `main`, validate them, and save them as non-contributable local lists

Date: 2026-11-27
Context: HU-07 uses the list published in `flagscope/feature-lists` when no list is saved locally (RF-14). The file comes from the Internet, and anyone can propose a list through a pull request.
Alternatives considered:
- The GitHub REST API (`/repos/.../contents/...`): returns metadata too, but anonymous use is limited to 60 requests per hour per IP address.
- GitHub Pages or release assets: another publishing step to maintain for every merged list.
- Trust the downloaded file as it is: simplest, but a malformed or mismatched file would become the local list.
Decision: the list is fetched from `https://raw.githubusercontent.com/flagscope/feature-lists/main/<architecture>/<major>/<build>.txt` with the Windows TLS stack and a 60-second limit. HTTP 404 means "not published" and the scan follows; any other failure stops with a message that names `flagscope/feature-lists`. A downloaded file is accepted only if it is UTF-8, at most 16 MB, starts with an AD-16 header for exactly the installed build and `x64`, and contains only `Name: ID` lines with unique names. It is saved as the local list with the line `# Source: flagscope/feature-lists`, so later runs need no network and `contribute` refuses it (AD-22). `--rescan` and targeted scans never use published lists.
Consequences: a list merged a few minutes ago may still be missing from `raw.githubusercontent.com` (observed after PR #1); flagscope then scans, as for an unpublished build. Lists are trusted as far as the repository review goes: the validation catches malformed or mismatched files, not wrong IDs in a well-formed file.
Status: active
