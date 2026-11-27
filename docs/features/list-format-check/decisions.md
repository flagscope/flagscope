> Status: approved
> Last updated: 2026-11-27
> Mode: new-feature

# Decisions: list-format-check

## AD-25: Check lists with a Python standard-library script in a GitHub Actions workflow of `flagscope/feature-lists`

Date: 2026-11-27
Context: RF-19 must check every pull request to `flagscope/feature-lists` that adds or changes a list. flagscope already validates downloaded lists in Rust (AD-23), but it builds only on Windows, it has no released binary yet, and AD-23 does not check the path or the order of the lines.
Alternatives considered:
- A `flagscope check-list` command, built from `flagscope/flagscope` source in the workflow: one implementation of the format, but every list pull request would compile the workspace on a Windows runner for several minutes, and `flagscope/feature-lists` would break whenever `main` of `flagscope/flagscope` does.
- Downloading a released flagscope binary: one implementation and no build, but there is no release yet (RF-10 and the release come later).
- A shell script with `grep` and `sort`: no interpreter needed, but byte-order sorting, line numbers, and reporting every problem become hard to read and to review.
Decision: `.github/workflows/lists.yml` in `flagscope/feature-lists` runs on pull requests to `main` with read-only permissions and one job, "List format", on `ubuntu-latest`. It checks out the pull request with `actions/checkout` pinned by commit (as AD-24) and runs `.github/scripts/check_lists.py` with the runner's `python3`, standard library only (PSF licence, compatible with AD-11's rule). The script prints one `file:line: problem` line per problem, checks every list before it exits, and exits non-zero if it found any.
Consequences: the format is defined twice, in Rust (AD-23) and in Python, and a change to AD-16 must change both; the published list must pass both. The check costs seconds and no compilation. The header is checked as strictly as `ListHeader::parse` in `flagscope-core` reads it (the title line, then `Build`, `Architecture`, `Generator`, and `Scanned` in that order), IDs and build numbers must fit in a `u32`, and a CR before LF is ignored as `flagscope-core` ignores it, so that every list the check passes is one flagscope accepts on download. The workflow runs the script from the pull request's own checkout, so a pull request can change the script and pass; while the check is advisory, the reviewer reads such a change in the diff. `CONTRIBUTING.md` of `flagscope/feature-lists` says only lists and files that describe the repository are committed, so that rule must name `.github/` before the workflow is merged.
Status: active

## AD-26: A list is every added or modified file in a top-level folder that does not start with a dot

Date: 2026-11-27
Context: RF-19.5 says a pull request that changes no list must not fail, and the check must find the lists a pull request adds or changes. Today the only architecture folder is `x64/`; ARM64 may add `arm64/` later (AD-16).
Alternatives considered:
- Only files under `x64/`: exact today, but adding `arm64/` would need a change to the check, and a list put in a misspelled folder (`X64/`) would not be checked at all.
- Every `.txt` file: a list saved under another extension would not be checked.
- A `paths` filter in the workflow, so that the check does not run on other pull requests: also meets RF-19.5, but a check that does not run cannot be required before merging, if the user later asks for that.
Decision: the workflow passes the files the pull request adds or modifies (`git diff --name-only --no-renames --diff-filter=d` against the base branch, so a moved list counts as added) to the script, which checks each one whose path has a top-level folder not starting with `.`. Deleted files and files at the top level (`README.md`, `LICENSE`) are not checked. The workflow runs on every pull request and passes when there is nothing to check. A list in a folder that does not match its `Architecture:` header fails RF-19.1; a list whose header names the same unknown architecture as its folder (for example `X64/` and `Architecture: X64`) passes, and is left to the reviewer.
Consequences: any new top-level folder for something other than lists would be checked as lists; today the repository has none, and `CONTRIBUTING.md` allows none.
Status: active
