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
