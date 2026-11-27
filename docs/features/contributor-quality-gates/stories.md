> Status: approved
> Last updated: 2026-11-27
> Mode: new-project

# User stories: contributor-quality-gates

The criteria are observed on real pull requests opened for the purpose and closed without merging. Their branches are deleted afterwards.

## HU-13: Formatting and linting are checked on every pull request

Covers: RNF-05.1, RNF-05.2

**Story.** As a contributor, I want formatting and lint problems reported on my pull request, so that I can fix them before a maintainer reviews it.

**Acceptance criteria.**

1. CA-13.1: A pull request whose code is formatted and has no linter warning shows the formatting-and-lint check as passed.
2. CA-13.2: A pull request with one formatting difference shows that check as failed, and its log names the file.
3. CA-13.3: A pull request with one linter warning shows that check as failed, and its log names the warning.
4. CA-13.4: The log of the "Tests" check shows the library crates (`flagscope-core`, `flagscope-symbols`, `flagscope-scanner`) built and tested in a step of their own, without any user-interface crate. (reworded on 2026-11-27 with the user's approval: the step runs in "Tests", AD-24)

**Status.** verified

## HU-14: Tests by level run on every pull request

Covers: RNF-07.1

**Story.** As a maintainer, I want every pull request to run all the automated tests on Windows x64, so that I never merge a change that breaks them.

**Acceptance criteria.**

1. CA-14.1: A pull request whose tests pass shows the test check as passed, and its log shows the unit, integration, and end-to-end tests run on a Windows x64 runner.
2. CA-14.2: A pull request with one failing test shows the test check as failed, and its log names the test.

**Status.** verified

## HU-15: Sign-off and human authorship are checked on every pull request

Covers: RNF-09.1

**Story.** As a maintainer, I want every commit of a pull request checked for its DCO sign-off and for AI identities, so that the rules in `CONTRIBUTING.md` hold without reading each commit by hand.

**Acceptance criteria.**

1. CA-15.1: A pull request whose commits all carry a sign-off and name no AI tool shows the authorship check as passed.
2. CA-15.2: A pull request with one commit without a `Signed-off-by:` line shows the check as failed, and its log names that commit.
3. CA-15.3: A pull request with one commit whose `Co-authored-by:` names an AI tool shows the check as failed, and its log names that commit.
4. CA-15.4: Merge commits created by GitHub when merging earlier pull requests do not make the check fail.

**Status.** verified

## HU-16: Secrets are caught before they reach the default branch

Covers: RNF-06.1

**Story.** As a maintainer of a public repository, I want a pull request that adds a secret to fail, so that no credential is merged.

**Acceptance criteria.**

1. CA-16.1: A pull request without secrets shows the secret-scan check as passed.
2. CA-16.2: A pull request that adds a file containing a fake credential in a well-known format shows the check as failed, and its log names the file. The fake credential is not a real one and is removed with the branch.

**Status.** verified

## HU-17: The default branch only takes checked pull requests

Covers: RNF-09.2

**Story.** As a maintainer, I want GitHub to refuse direct pushes and pull requests with failing checks on the default branch, so that the rules hold even when someone forgets them.

**Acceptance criteria.**

1. CA-17.1: A direct `git push` to `main` is rejected by GitHub.
2. CA-17.2: A pull request with a failing check cannot be merged from the GitHub page.

**Status.** verified
