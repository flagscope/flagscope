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

## Verification log

| Story | Criterion | Result | Date | Evidence |
| ----- | --------- | ------ | ---- | -------- |
| HU-01 | CA-01.1: downloads the PDB of one system binary from Microsoft's symbol server and prints at least one `Name: ID` entry | pass | 2026-11-26 | Deleted any local copy first, then ran `cargo run -p flagscope-scanner --example extract_one -- C:\Windows\System32\shell32.dll <symbols dir>` on build 26300.10075 x64. It printed `Downloaded ...\shell32.pdb (12816384 bytes)`, 261 `Name: ID` lines, and `261 features, 0 names without a readable descriptor`. |
| HU-01 | CA-01.2: at least three printed entries match name and ID in a mach2 list for build 26100 | pass | 2026-11-26 | Compared the 261 printed entries with mach2 `features/ge_release/amd64/26100.2314_WNC.txt` (3,399 entries): 40 names in common, 40 with the same ID, 0 different. Examples: `AccountAndRegionalPolicyEnforcement: 47557358`, `AppDefaultHashRotation: 43229420`, `AppDefaultHashRotationUpdateHashes: 27623730`. |
| HU-01 | CA-01.3: prints the size of the downloaded symbol file | pass | 2026-11-26 | Same run printed `(12816384 bytes)`. |
| HU-01 | CA-01.4: formatter zero differences, linter zero warnings | pass | 2026-11-26 | `cargo fmt --all --check` exited 0 with no output; `cargo clippy --workspace --all-targets -- -D warnings` finished with no warnings, after removing two unused-dependency warnings Cargo reported. |
