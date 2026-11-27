> Status: approved
> Last updated: 2026-11-27
> Mode: new-feature

# Requirements: list-format-check

Feature for the contributor and the maintainer of `flagscope/feature-lists`: every pull request that adds or changes a list is checked automatically against the AD-16 format, so that the contributor sees format problems before a maintainer reviews the list, and the maintainer reads the list for its content, not its format.

A list is any file under an architecture folder of `flagscope/feature-lists` (today only `x64/`).

## Functional

### RF-19.1: Path matches the header

Refines: RF-19
The check must fail when a list's path is not `<architecture>/<major>/<major>.<revision>.txt` for the `Architecture:` and `Build:` values in its own header.

### RF-19.2: Required header lines

Refines: RF-19
The check must fail when a list does not start with `#` header lines that include `Build:`, `Architecture:`, `Generator:` (flagscope version and scan mode), and `Scanned:` (scan date), or when a `#` line appears after the first `Name: ID` line.

### RF-19.3: Feature lines

Refines: RF-19
The check must fail when a line after the header is not a `Name: ID` line (a name without spaces or colons, a colon and one space, and a decimal ID), when the lines are not sorted by name in the byte order of their UTF-8 text (the order flagscope writes, observed in `x64/26300/26300.10075.txt`), or when a name appears more than once.

### RF-19.4: Errors name the file and the line

Refines: RF-19
Every failure must name the file and, when the problem is on one line, the line number, and the check must report every problem it finds in the pull request, not only the first.

### RF-19.5: Only lists are checked

Refines: RF-19
A pull request that adds or changes no list must not be failed by this check.
