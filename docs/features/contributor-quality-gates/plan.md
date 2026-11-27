> Status: approved
> Last updated: 2026-11-27
> Mode: new-project

# Plan: contributor-quality-gates

## Order

1. HU-13: first, because it sets up the workflow file the other checks add to, and formatting and linting are the quickest feedback for a contributor.
2. HU-14: depends on HU-13's workflow and Rust toolchain setup; adds the test job on a Windows x64 runner.
3. HU-15: independent of the Rust jobs; a short script over the pull request's commits.
4. HU-16: independent; a secret-scan job.
5. HU-17: last, because branch protection must name the checks the earlier stories create. It is a repository setting the user changes on GitHub; the assistant cannot change it.

## Dependencies

- HU-14 depends on HU-13 (same workflow and toolchain setup).
- HU-17 depends on HU-13 to HU-16 (it requires their checks).
- Every criterion needs pull requests opened on `flagscope/flagscope` by the user, because pushing and opening pull requests is the user's action under `CONTRIBUTING.md`.

## Risks

- GitHub-hosted Windows runners run Windows Server, not Windows 11. The current tests need no Windows 11 feature, but a future test that does would have to be marked to run only on a real Windows 11 machine.
- The tools used in the jobs (Rust toolchain installer, secret scanner) must meet AD-11's licence rule, and are pinned to fixed versions so that a release upstream cannot change a check silently.
- Branch protection on a repository owned by an organization on the free plan is available for public repositories; `flagscope/flagscope` is public.
- An automated format check for `flagscope/feature-lists` (path matches the header, `Name: ID` lines, sorted, unique) is not part of this feature; on 2026-11-27 the user moved it into scope as the feature `list-format-check`.
