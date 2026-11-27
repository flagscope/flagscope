> Status: draft
> Last updated: 2026-11-26
> Mode: new-project

# Quality

## Quality attributes

| Attribute (ISO/IEC 25010:2023) | Why it matters here | Requirement |
| ------------------------------ | ------------------- | ----------- |
| Compatibility (platform) | The feature APIs and the scanned binaries exist only on Windows 11 x64. | RNF-01 |
| Interaction capability (translatability) | The application is English first and must be translatable later. | RNF-02 |
| Reliability (recoverability) | A wrong feature change can make Windows unstable; every change must be reversible. | RNF-03 |
| Security (least privilege) | Downloaded files are parsed; that must never happen with administrator rights. | RNF-04 |
| Maintainability (modularity, reusability, analysability, testability) | The code must grow and accept community contributions. | RNF-05, RNF-07 |
| Security (confidentiality of the repository) | The repository is public. | RNF-06 |
| Performance efficiency (resource use) | Symbol files are large; disk use must stay within a budget the user sets after measuring. | RNF-08 |

## Measurements

| Date | What | Value | Source |
| ---- | ---- | ----- | ------ |
| 2026-11-26 | Size of `shell32.pdb` for build 26300.10075 | 12,816,384 bytes | HU-01 run (CA-01.3) |
| 2026-11-26 | Full scan of build 26300.10075: duration | 3,214 s (53.6 min), sequential downloads | HU-02 run |
| 2026-11-26 | Full scan of build 26300.10075: symbol files downloaded | 7,952.3 MB from 2,413 PDBs (the AD-13 sample had estimated about 3.2 GB) | HU-02 run |
| 2026-11-26 | Full scan of build 26300.10075: result | 7,402 features; 6,371 binaries examined; 76 without a public PDB; 0 unreadable PDBs; 0 conflicting names; 36 names without a readable descriptor | HU-02 run |
| 2026-11-26 | Full scan vs mach2 `26100.2314_WNC` | 783 names in common: 776 same ID; 4 where mach2 has ID 0 and the scan found an ID; 3 different IDs (`DmaSsoAADCompliance`, `FrameServerDX12Support`, `LocalKDC`), not yet explained | HU-02 run |
| 2026-11-26 | Full scan, default options (HU-05 code): duration and peak disk | 3,081 s; peak 354.8 MB (the largest single PDB, `windows.ui.xaml.pdb`); 0 symbol files left | HU-05 verification, run 1 |
| 2026-11-26 | Full scan keeping symbol files: duration and disk | 3,485 s; 2,413 downloads, 2,383 distinct files kept (6.2 GiB); the scan reported a peak of 7,952.3 MB because it counts the 30 repeated downloads of an identical PDB, which overwrite the same file | HU-05 verification, run 3 |
| 2026-11-26 | Loading the saved list | 0 s, no symbol directory created | HU-05 verification, run 2 |

## Verification log

| Story | Criterion | Result | Date | Evidence |
| ----- | --------- | ------ | ---- | -------- |
| HU-01 | CA-01.1: downloads the PDB of one system binary from Microsoft's symbol server and prints at least one `Name: ID` entry | pass | 2026-11-26 | Deleted any local copy first, then ran `cargo run -p flagscope-scanner --example extract_one -- C:\Windows\System32\shell32.dll <symbols dir>` on build 26300.10075 x64. It printed `Downloaded ...\shell32.pdb (12816384 bytes)`, 261 `Name: ID` lines, and `261 features, 0 names without a readable descriptor`. |
| HU-01 | CA-01.2: at least three printed entries match name and ID in a mach2 list for build 26100 | pass | 2026-11-26 | Compared the 261 printed entries with mach2 `features/ge_release/amd64/26100.2314_WNC.txt` (3,399 entries): 40 names in common, 40 with the same ID, 0 different. Examples: `AccountAndRegionalPolicyEnforcement: 47557358`, `AppDefaultHashRotation: 43229420`, `AppDefaultHashRotationUpdateHashes: 27623730`. |
| HU-01 | CA-01.3: prints the size of the downloaded symbol file | pass | 2026-11-26 | Same run printed `(12816384 bytes)`. |
| HU-01 | CA-01.4: formatter zero differences, linter zero warnings | pass | 2026-11-26 | `cargo fmt --all --check` exited 0 with no output; `cargo clippy --workspace --all-targets -- -D warnings` finished with no warnings, after removing two unused-dependency warnings Cargo reported. |
| HU-02 | CA-02.1: on Windows 11 x64 a scan produces a feature list and shows the build number `winver` shows | pass | 2026-11-26 | `flagscope scan` (release build) exited 0 and printed `Windows build 26300.10075` followed by 7,402 `Name: ID` lines. Observed by the user: `winver` on the same machine shows "Version 26H2 (OS Build 26300.10075)". `ver` printed `10.0.26300.10075`. |
| HU-02 | CA-02.2: the list contains no duplicate feature name | pass | 2026-11-26 | Counted repeated names in the 7,402 printed lines: 0. All lines match `Name: ID`. |
| HU-02 | CA-02.3: the scan completes from a console not running as administrator | pass | 2026-11-26 | The console had integrity level Medium (`whoami /groups`: S-1-16-8192), `IsInRole(Administrator)` was False and `net session` was denied; the scan from that console exited 0 after 3,214 s. |
| HU-02 | CA-02.4: with the network disconnected and no saved list, prints a message naming the missing connection, exits non-zero, leaves no list file | pass | 2026-11-26 | Observed by the user, with the network disconnected and no saved list: `flagscope scan` printed `Error: no network connection. Cannot reach Microsoft's symbol server (https://msdl.microsoft.com/download/symbols/helppane.pdb/...)` and `No list was saved.`; `$LASTEXITCODE` was 1; `Test-Path $env:LOCALAPPDATAlagscope` was False. |
| HU-02 | CA-02.1, CA-02.3, CA-02.4 | blocked | 2026-11-26 | Reopened: HU-03 rewrote the `scan` command flow in `flagscope-cli/src/main.rs`, which these passes covered. To be observed again with the next full scan (CA-02.1, CA-02.3) and a new run without network (CA-02.4). CA-02.2 stays `pass`: the merge that guarantees unique names is in `flagscope-scanner` and did not change. |
| HU-03 | CA-03.1: writes the list to the given file, every line `Name: ID` | blocked | 2026-11-26 | Needs a completed scan (about 54 minutes and 8 GB at the moment). The writer is covered by a unit test (`write_list_writes_one_line_per_feature_in_order`), which is not an observation of the tool. |
| HU-03 | CA-03.2: console entries equal the file entries, in the same order | blocked | 2026-11-26 | Needs a completed scan. |
| HU-03 | CA-03.3: an existing file is not overwritten unless the overwrite option is given | blocked | 2026-11-26 | Partly observed: with an existing file and no `--overwrite`, `flagscope scan --output <file>` printed `Error: <file> already exists. Choose another file, or pass --overwrite to replace it.` / `Nothing was scanned or written.`, exited 1, and the file still held its original content. Replacing the file with `--overwrite` needs a completed scan. The end-to-end test `existing_output_file_is_not_overwritten_without_the_option` checks the refusal on every test run. |
| HU-02 | CA-02.1 (re-observed after HU-03 and HU-05) | pass | 2026-11-26 | `flagscope scan --output <file>` exited 0 and printed `Windows build 26300.10075` and 7,402 `Name: ID` lines; `winver` shows 26300.10075 (observed by the user earlier the same day, same machine and build). |
| HU-02 | CA-02.3 (re-observed after HU-03 and HU-05) | pass | 2026-11-26 | The same scan, from a console at integrity level Medium (`Mandatory Label\Medium Mandatory Level`), exited 0 after 3,081 s. |
| HU-02 | CA-02.4 (to re-observe after HU-03 and HU-05) | blocked | 2026-11-26 | Needs the network disconnected on the user's machine; postponed by the user until a download finishes. |
| HU-03 | CA-03.1: writes the list to the given file, every line `Name: ID` | pass | 2026-11-26 | `flagscope scan --output list.txt --overwrite` wrote 7,402 lines, all matching `Name: ID` (0 other lines). |
| HU-03 | CA-03.2: console entries equal the file entries, in the same order | pass | 2026-11-26 | The 7,402 entry lines printed to the console by that run are byte-identical to `list.txt`; the first scan's console output is byte-identical to the saved list. |
| HU-03 | CA-03.3: an existing file is not overwritten unless the overwrite option is given | pass | 2026-11-26 | Without `--overwrite` (current code): printed `Error: <file> already exists. Choose another file, or pass --overwrite to replace it.` and `Nothing was scanned or written.`, exit 1, original content kept. With `--overwrite`: a file containing `stale` was replaced by the 7,402-line list. |
| HU-05 | CA-05.1: a second run returns the saved list, says so, downloads nothing | pass | 2026-11-26 | Second `flagscope scan` took 0 s and printed `7402 features in build 26300.10075, from the saved list ...\lists³00.10075.txt.` / `Nothing was downloaded.`; no symbol directory existed afterwards. |
| HU-05 | CA-05.2: with the rescan option, downloads again and saves a new list | blocked | 2026-11-26 | Partly observed: `--keep-symbol-files`, which implies `--rescan` (AD-17), downloaded 2,413 symbol files with a list already saved and rewrote the saved list (modified 18:30:48). A run with `--rescan` itself has not been observed; it needs a full scan. |
| HU-05 | CA-05.3: after a default scan, no downloaded symbol file remains | pass | 2026-11-26 | After the first scan, `%LOCALAPPDATA%lagscope\symbols` did not exist and no `.pdb` file was found under `%LOCALAPPDATA%lagscope`. |
| HU-05 | CA-05.4: with the keep option, every downloaded symbol file remains | pass | 2026-11-26 | 2,413 downloads, 2,383 `.pdb` files on disk under `symbols\scan-25020`. Checked: the 30 extra downloads are repeated downloads of 26 PDB identities shared by several binaries (for example `OSClient.NET.pdb`, used by 5 binaries), written to the same path; every distinct file downloaded is on disk. |
| HU-05 | CA-05.5: every scan prints the peak additional disk space it used | pass | 2026-11-26 | Both scans printed `Peak disk space used by symbol files:` (354.8 MB and 7,952.3 MB). Known defect: with `--keep-symbol-files` the peak counts repeated downloads of the same PDB, so it overstates disk use (7,952.3 MB reported, 2,383 files on disk); fixed by downloading each identical PDB once in `lighter-scans`. |
