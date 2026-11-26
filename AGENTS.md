# flagscope

## Repository rules

The rules for this repository live in [`CONTRIBUTING.md`](CONTRIBUTING.md). They are mandatory for agents as well as people. Read them before any change, commit, or pull request. In particular:

- Commits, pushes, and pull requests are authored by the responsible human, never under an AI identity, and every commit has a DCO sign-off.
- Everything is written in English, with precise technical terminology.
- No sensitive data and no generated or local content is ever committed (see "Repository content" in `CONTRIBUTING.md`).

## Documentation

The decisions that govern this repository live in `docs/`. Read them before changing code.

| File                  | What it holds                            |
| --------------------- | ---------------------------------------- |
| `docs/00-contexto.md` | problem, users, constraints, methodology |
| `docs/01-alcance.md`  | in scope, out of scope, assumptions      |

This table grows as each phase is approved (requirements, architecture, quality, and the per-feature folders).

## Rules

- No code without a story in `docs/features/<slug>/historias.md`.
- Any decision someone could later want to reverse goes in its feature's decisions file (decisiones.md).
- A criterion is verified by observation. What could not be observed is recorded as blocked, not as passed.
