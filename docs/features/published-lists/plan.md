> Status: approved
> Last updated: 2026-11-26
> Mode: new-project

# Plan: published-lists

## Order

1. HU-06: first, because `flagscope/feature-lists` is empty. Until a real scanned list is published there, nothing can be downloaded, so HU-07 and HU-08 cannot be observed.
2. Publishing the first list: the user scans build 26300.10075, prepares it with HU-06, and submits it to `flagscope/feature-lists` through a pull request. This is a step of the user's, not a story.
3. HU-07: depends on that first published list.
4. HU-08: depends on HU-07 for downloading published lists, and on two published revisions of the same major build. This situation appears naturally after the next cumulative update of the user's Windows, when the new revision has no list yet.

## Dependencies

- HU-07 and HU-08 need the saved-list mechanism of `find-feature-ids` HU-05.
- HU-06 needs to know whether the local list came from a scan; HU-05's saved list does not record that yet, so HU-06 adds it.
- HU-08's criteria can only be observed once two revisions of the same major build are published.

## Risks

- HU-08 may stay blocked for weeks, until the next cumulative update.
- Without automated checks in `flagscope/feature-lists`, reviewers check contributed lists by reading them. Whether to add an automated format check to that repository is an open question for the user.
- GitHub limits unauthenticated API requests to 60 per hour per IP address. Finding the nearest build lists a folder through that API, at most one request per run.
