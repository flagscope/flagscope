# Contributing to flagscope

These rules are mandatory for every contributor, human or automated. Pull requests that break them are not merged.

## License and sign-off

- The project is licensed under GPL-3.0-only (see `LICENSE`). By contributing, you agree that your contribution is licensed under the same terms.
- Every commit carries a Developer Certificate of Origin sign-off (`git commit -s`), which adds a `Signed-off-by:` line with your real name and an email you control. A GitHub no-reply address is accepted. Commits without it are rejected.

## Repository content

Only content that adds value to the project is committed: source code, tests, documentation, and the configuration needed to build, check, and release it.

- **Never committed: sensitive data.** This includes credentials, tokens, API keys, private keys and certificates (`.pem`, `.key`, `.pfx`), `.env` files, personal data, and machine-specific paths or user names. A secret that reaches a commit is treated as leaked: revoke and rotate it first, then remove it from the history. Deleting it in a later commit is not enough.
- **Never committed: generated or local content.** This includes build output (`target/`), downloaded symbol files (`*.pdb`), scanner caches, logs, crash dumps, editor and IDE settings, and operating-system files. These are listed in `.gitignore`.
- **Never committed: third-party binaries or data without a license that allows redistribution.** In particular, Microsoft's symbol files are never committed.
- `.gitignore` only lists what this project produces or what commonly ends up in it by accident. Anything specific to your own editor or tools goes in your global git ignore file (`core.excludesFile`).
- `Cargo.lock` is committed, because the repository builds an executable.
- Secret scanning runs on every push and pull request. A pull request where it finds anything is not merged.

## Authorship and AI tools

- Every commit, push, pull request, review, and release is the responsibility of a human author, under that person's own name.
- AI tools may assist, but they are never the author, the committer, or the account that opens a pull request. Commits do not carry AI identities in `Author`, `Committer`, `Signed-off-by`, or `Co-authored-by`.
- The human author has read and understands every line they submit, including code produced with AI assistance.
- Every rule in this document applies to AI-assisted work exactly as it applies to work written by hand. Before committing, the human author checks the AI output against each rule: content, language, terminology, authorship, sign-off, and the documentation-first workflow. Automated checks enforce what can be enforced; the rest is the author's responsibility.
- AI agents working in this repository read `AGENTS.md` and this document before acting, and stop to ask a human when an action would break a rule.

## Review and merging

- Every pull request needs at least one approving review from a maintainer before it is merged. Collaborators with write access may approve, under their responsibility.
- Automated AI review may run on pull requests, but it is advisory only and never replaces human approval.
- Changes reach the default branch only through pull requests.

## Language

- Everything in the repository is written in English: code, comments, documentation, commits, and pull requests.
- User-facing strings are never hard-coded in the user interface. They go through the localization layer so the application can be translated.

## Terminology

Precise technical terminology is mandatory in code, comments, documentation, commits, and pull requests. Sources apply in this order:

1. General software engineering: **ISO/IEC/IEEE 24765:2017** (Systems and software engineering — Vocabulary; online version: SEVOCAB).
2. Rust and Cargo: the glossaries of **The Cargo Book** and **The Rust Reference**.
3. Project domain terms: the glossary in `docs/00-contexto.md`. A new term is added there before it is used.

## Code

- The logic (one or more library crates) and the user interface (a binary crate using GPUI) are separate crates in one Cargo workspace.
- Code is formatted, modular, and reusable, and comments are clean. The exact conventions (formatting, lints, documentation) are recorded in `docs/` once the architecture phase is approved.

## Safety of the user's system

- Every action that changes system state records the previous state first and can be reverted.
- Only the elevated process applies changes. Everything else, including downloading and parsing symbol files, runs without administrator rights.

## Documentation-first workflow

- No code without a story in `docs/features/<slug>/historias.md`.
- Any decision someone could later want to reverse is recorded in its feature's decisions file (decisiones.md).
- An acceptance criterion is verified by observation. What could not be observed is recorded as blocked, not as passed.
