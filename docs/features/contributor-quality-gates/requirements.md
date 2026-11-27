> Status: approved
> Last updated: 2026-11-27
> Mode: new-project

# Requirements: contributor-quality-gates

Feature for the contributor and the maintainer: every pull request to `flagscope/flagscope` is checked automatically against the rules in `CONTRIBUTING.md` that a machine can check, before a human reviews it.

## Non-functional

### RNF-05.1: Formatting and linting on every pull request

Refines: RNF-05
Every pull request to the default branch must run the formatter check and the linter with the workspace configuration, and the check must fail when the formatter reports a difference or the linter reports a warning.

### RNF-05.2: Library crates without the user-interface crate

Refines: RNF-05
Every pull request must build and test the library crates on their own, without any user-interface crate.

### RNF-06.1: Secret scanning on every pull request

Refines: RNF-06
Every pull request must run a secret scan over the commits it adds, and the check must fail when the scan finds a secret.

### RNF-07.1: Tests by level on every pull request

Refines: RNF-07
Every pull request must run the unit tests, the integration tests of the library crates, and the end-to-end tests of the command-line interface on Windows x64, and the check must fail when any of them fails.

### RNF-09.1: Sign-off and human authorship on every pull request

Refines: RNF-09
Every pull request must check each commit it adds, except merge commits created by GitHub: the commit must carry a `Signed-off-by:` line, and no `Author`, `Committer`, `Signed-off-by`, or `Co-authored-by` may name an AI tool. The check must fail and name the commit otherwise.

### RNF-09.2: Checks required before merging

Refines: RNF-09
The default branch must accept changes only through pull requests whose checks have passed.
