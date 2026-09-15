# STATE — gh-rs

## Current
Phase: Sprint 1 complete — auth + repo live-verified (device flow, keyring, list, view, clone, logout lifecycle). At S1 checkpoint. pr (S2) next if checkpoint passes.

## Progress
- [x] S0 setup
- [x] S1 auth + repo
- [ ] S2 pr: not started
- [ ] S3 issue: not started

## Not live-tested
- repo create (side effect on account — needs one manual run)

## Decisions locked
- Plain text output v0.1 (no TUI/pager)
- Repo create defaults private; --public opts out
- octocrab 0.54 (no serde-visible Repository struct — local RepositorySummary used)
- octocrab 0.54 dropped user_agent() — default "octocrab" UA is valid
- reqwest needs `form` + `json` features (cargo add without defaults)
- clap nested subcommands: named field + #[command(subcommand)]
- Private repo, publish-ready; flip only on explicit go