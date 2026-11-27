> Status: approved
> Last updated: 2026-11-27
> Mode: new-feature

# Plan: list-format-check

## Order

1. HU-18: first, because it creates the workflow in `flagscope/feature-lists` that finds the lists a pull request adds or changes, which HU-19 adds to.
2. HU-19: the line rules, on the same workflow.

HU-18 and HU-19 were implemented together on 2026-11-27, because both are rules of the same script (`.github/scripts/check_lists.py` in `flagscope/feature-lists`, branch `ci/list-format-check`). Their criteria are observed one pull request at a time, on branches cut from `main` after the workflow is merged. The pull request that adds the workflow changes no list, so its "List format" run is the first observation of a run with nothing to check.

## Dependencies

- HU-19 depends on HU-18 (same workflow and the same list of changed files).
- Every criterion needs pull requests opened on `flagscope/feature-lists` by the user, because pushing and opening pull requests is the user's action under `CONTRIBUTING.md`.
- How the check is built is recorded in AD-25, and which files it checks in AD-26.
- Before the workflow is merged, the "Repository content" rule of `flagscope/feature-lists`'s `CONTRIBUTING.md` must allow `.github/` (AD-25).

## Risks

- The code lives in `flagscope/feature-lists`, while its documents live in `flagscope/flagscope`. Commits in `flagscope/feature-lists` cite HU-18 and HU-19 so the two can be matched.
- Pull requests from forks run with a read-only token. The check only reads the files of the pull request, so this does not limit it.
- Whether the check must pass before a list can be merged (branch protection in `flagscope/feature-lists`, as HU-17 did for `flagscope/flagscope`) is an open question for the user (`docs/01-scope.md`). If the answer is yes, it becomes a third story.
- A pull request can change `check_lists.py` itself and pass the check. While the check is advisory, the reviewer catches that in the diff; if the check becomes required, the script should come from `main` instead.
- Any tool the workflow uses must meet AD-11's licence rule and be pinned to a fixed version.
