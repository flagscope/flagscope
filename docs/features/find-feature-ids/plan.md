> Status: approved
> Last updated: 2026-11-26
> Mode: new-project

# Plan: find-feature-ids

## Order

1. HU-01: first, because it tests the main project risk: whether public symbol files still contain feature names and IDs. Its result decides whether the rest of the plan holds.
2. HU-02: depends on HU-01. Extends extraction from one symbol file to the whole installed build.
3. HU-03: depends on HU-02. Delivers the list in a form the user can read, which completes the first version.
4. HU-05: depends on HU-02. Saved lists and clean-up make repeated runs practical.
5. HU-04: depends on HU-02. The estimate needs the list of symbol files to download, and its accuracy can be compared with the sizes HU-05 measures.

## Dependencies

- HU-02, HU-03, HU-04, and HU-05 all depend on HU-01: without names and IDs in the symbol files there is nothing to scan.
- HU-03, HU-04, and HU-05 depend on HU-02, because they work on a full scan.
- HU-03, HU-04, and HU-05 do not depend on each other.

## Risks

- Symbol files no longer contain the feature IDs, or contain names without IDs. HU-01 would fail and the approach would need to be reconsidered.
- The installed build is newer than every mach2 list, so fewer features can be cross-checked in CA-01.2. Features that exist in build 26100 and in the installed build are still enough.
- Microsoft's symbol server limits or slows down automated downloads, which would make full scans slow. HU-04 makes this visible.
- The structural decisions (workspace layout, crates, conventions) belong to the architecture phase, which comes before HU-01 is implemented.
