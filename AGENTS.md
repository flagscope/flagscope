# flagscope

## Repository rules

The rules for this repository live in [`CONTRIBUTING.md`](CONTRIBUTING.md). They are mandatory for agents as well as people. Read them before any change, commit, or pull request. In particular:

- Commits, pushes, and pull requests are authored by the responsible human, never under an AI identity, and every commit has a DCO sign-off.
- Everything is written in English, with precise technical terminology.
- No sensitive data and no generated or local content is ever committed (see "Repository content" in `CONTRIBUTING.md`).

## Documentation

The decisions that govern this repository live in `docs/`. Read them before changing code.

| File                              | What it holds                                        |
| --------------------------------- | ---------------------------------------------------- |
| `docs/00-context.md`              | problem, users, constraints, methodology             |
| `docs/01-scope.md`                | in scope, out of scope, assumptions, delivery order  |
| `docs/02-requirements.md`         | functional and non-functional requirements           |
| `docs/03-architecture.md`         | crate boundaries, conventions, decision index        |
| `docs/04-quality.md`              | quality attributes, measurements, verification log   |
| `docs/features/find-feature-ids/` | requirements, stories, plan, and decisions of the first feature |
| `docs/features/lighter-scans/`    | requirements, stories, and plan of the lighter-scans feature |
| `docs/features/published-lists/`  | requirements, stories, plan, and decisions of the published-lists feature |

Inside each feature folder the files are `requirements.md`, `stories.md`, `plan.md`, and `decisions.md`.

File and directory names are in English, like everything else in this repository. This overrides any tool or template convention that uses names in another language: an agent creating documentation here uses the names above.

## Rules

- No code without a story in `docs/features/<slug>/stories.md`.
- Any decision someone could later want to reverse goes in its feature's decisions file (decisions.md).
- A criterion is verified by observation. What could not be observed is recorded as blocked, not as passed.
