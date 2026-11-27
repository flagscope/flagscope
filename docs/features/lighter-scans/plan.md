> Status: approved
> Last updated: 2026-11-26
> Mode: new-project

# Plan: lighter-scans

## Order

1. HU-09: first, because it gives the largest saving (measured: about 7.7 GB to about 3.1 GB for build 26300.10075) and fixes the peak disk report of `find-feature-ids` HU-05.
2. HU-10: depends on HU-09, which makes light mode the default; full mode is then the explicit choice that keeping files needs.
3. HU-12: depends on HU-09's handling of distinct symbol files, which gives the key the stored results use.
4. HU-11: last, because it only selects binaries for the same pipeline.
5. Verification batch for `find-feature-ids` and `lighter-scans` together: the full scans run at the same time (each scan has its own symbol folder), then the short runs, then the run without network, which needs the user.

## Dependencies

- HU-10, HU-11, and HU-12 depend on HU-09.
- CA-09.1 and CA-09.2 compare a light scan with a full scan of the same build, so the batch includes one full-mode scan.
- CA-12.1 needs a light scan first, and then a second scan.

## Risks

- Light mode depends on Microsoft's symbol server answering HTTP range requests, observed on 2026-11-26; if that stops, light mode would have to fall back to full downloads.
- The stored results of HU-12 grow with every build scanned; they are small text, but nothing removes old entries yet.
- Downloads in parallel, which shorten scans, are a design decision taken during HU-09 and recorded then.
