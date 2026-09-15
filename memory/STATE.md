# STATE — gh-rs

## Current
Phase: v0.1 complete — auth + repo + pr + issue all live-verified. Audit fixes applied (issue-PR filter, device-flow error decode, clone Bearer header, merged-state render, origin URL trailing-slash parse, shared split_repo, CI concurrency, bare-URL doc). Benchmarks in README. Next: tag v0.1.0 + publish-readiness sweep on explicit go.

## Progress
- [x] S0 setup
- [x] S1 auth + repo
- [x] S2 pr: create/list/view/diff/merge live-tested (scratch gh-pr-test-1 deleted)
- [x] S3 issue: create/list/view/close live-tested (scratch gh-issue-test-1 deleted)
- [x] Final pass items 1+2: README usage sections, repo view via render::repository
- [x] Benchmarks: gh 2.93.0 vs gh-rs (auth/repo/pr/issue + --help + binary size)

## Live-tested
- repo create: ✓ private-by-default, view/list/clone verified, test repo deleted. Note: deletion used system `gh` — repo delete is not a v0.1 command.
- pr: ✓ create(#1) → list → view → diff → squash-merge → list-empty on gh-pr-test-1; invalid method guard. Scratch repo deleted (404-confirmed).
- issue: ✓ create(#1-5) → list → view → close-all → list-empty on gh-issue-test-1; 404 guard on nonexistent. Scratch repo deleted (404-confirmed).

## Decisions locked
- Plain text output v0.1 (no TUI/pager)
- Repo create defaults private; --public opts out
- octocrab 0.54 (no serde-visible Repository struct — local RepositorySummary used)
- octocrab 0.54 dropped user_agent() — default "octocrab" UA is valid
- reqwest needs `form` + `json` features (cargo add without defaults)
- clap nested subcommands: named field + #[command(subcommand)]
- Private repo, publish-ready; flip only on explicit go