> Status: approved
> Last updated: 2026-11-27
> Mode: new-feature

# User stories: list-format-check

The check runs in `flagscope/feature-lists`. The criteria are observed on real pull requests to that repository, opened for the purpose by the user and closed without merging; their branches are deleted afterwards. The well-formed list in the criteria is `x64/26300/26300.10075.txt`, already published.

## HU-18: A contributed list is checked for its place and its header

Covers: RF-19.1, RF-19.2, RF-19.4, RF-19.5

**Story.** As a contributor to `flagscope/feature-lists`, I want my pull request to tell me when my list is in the wrong place or has an incomplete header, so that I can fix it before a maintainer reviews it.

**Acceptance criteria.**

1. CA-18.1: A pull request that adds a copy of the published list under its correct path for another build (with `Build:` changed to match) shows the format check as passed.
2. CA-18.2: A pull request that adds that list under a path whose build differs from its `Build:` line shows the format check as failed, and its log names the file and both builds.
3. CA-18.3: A pull request that adds a list without its `Scanned:` line shows the format check as failed, and its log names the file and the missing line.
4. CA-18.4: A pull request that changes only `README.md` does not show the format check as failed.

**Status.** verified

## HU-19: A contributed list is checked line by line

Covers: RF-19.3, RF-19.4

**Story.** As a maintainer of `flagscope/feature-lists`, I want every feature line of a contributed list checked for its form, its order, and duplicate names, so that I review the content of a list, not its format.

**Acceptance criteria.**

1. CA-19.1: A pull request that changes the published list so that one line has no ID shows the format check as failed, and its log names the file and that line number.
2. CA-19.2: A pull request that swaps two adjacent feature lines of the published list shows the format check as failed, and its log names the file and the line out of order.
3. CA-19.3: A pull request that duplicates one feature line of the published list shows the format check as failed, and its log names the file and the duplicated name.
4. CA-19.4: A pull request that makes the three changes above in one list shows the format check as failed, and its log names all three problems.

**Status.** verified
