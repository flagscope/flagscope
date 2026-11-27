> Status: approved
> Last updated: 2026-11-27
> Mode: new-project

# Decisions: contributor-quality-gates

## AD-24: Check pull requests with one GitHub Actions workflow of four jobs, with every tool pinned

Date: 2026-11-27
Context: RNF-05.1, RNF-05.2, RNF-06.1, RNF-07.1, and RNF-09.1 must be checked on every pull request to `main`, and HU-17 will require those checks before merging.
Alternatives considered:
- One job with every step: fewer runners, but GitHub shows one check, so a formatting error and a failing test look the same on the pull request.
- Third-party actions for the toolchain (for example `dtolnay/rust-toolchain`), DCO (the DCO GitHub App), and secrets (`gitleaks/gitleaks-action`): less script, but more code from outside the project runs with the repository's token, the DCO App does not check AI identities, and `gitleaks-action` requires a paid licence for organizations.
- Running the Rust jobs on Linux: faster runners, but the scanner reads the Windows registry and Windows binaries, and the tests must run on the platform flagscope supports.
Decision: `.github/workflows/checks.yml` runs on pull requests to `main` with read-only permissions and four jobs, each a separate check: "Formatting and lints" (`cargo fmt --check`, `cargo clippy -D warnings`, Windows), "Tests" (library crates on their own, then the command-line crate, Windows), "Sign-off and authorship" (`.github/scripts/check-commits.sh` over the pull request's commits, Linux), and "Secret scan" (gitleaks over the pull request's commits, Linux). The only action used is `actions/checkout` (MIT), pinned by commit; Rust is installed with the runner's `rustup` at the workspace's `rust-version`; gitleaks (MIT) is downloaded at a fixed version and its SHA-256 is checked. The authorship script exempts only merge commits whose committer is `GitHub <noreply@github.com>`, and matches AI tools by a list of names (`claude`, `anthropic`, `copilot`, `chatgpt`, `openai`, `gemini`, `codex`, `devin`).
Consequences: each check fails on its own and names the problem. Two Windows jobs compile the workspace, which costs runner minutes; there is no build cache yet. Updating a tool means changing a pinned version or checksum in the workflow. The AI name list must be extended by hand; a human whose address contains one of those words would need another address. The library-crates step that RNF-05.2 asks for runs in the "Tests" check, not in "Formatting and lints".
Status: active
