# STATE — gh-rs

## Current
Phase: Sprint 0 complete. Ready for Sprint 1 (device flow round-trip first).

## Progress
- [x] S0 setup: cargo init (edition 2021), deps, error.rs, ci.yml, LICENSE, .gitignore, docs
- [ ] S1 auth + repo: not started — blocked on OAuth App client_id
- [ ] S2 pr: not started
- [ ] S3 issue: not started

## OAuth App — DONE
Registered: `gh-rs`, client_id `Ov23liNghdlHihgqbM5V` (public by design). Device flow enabled. Scope requested at login: `repo`. client_id goes into `github/client.rs` as a constant in Sprint 1. Ignore the client secret — not needed for device flow, never store it.

## Decisions locked
- Plain text output v0.1 (no TUI/pager)
- octocrab 0.54.2, reqwest 0.13 (json), tokio 1.53 (current_thread), keyring 4.2 (windows-native default), clap 4.6 derive, thiserror 2
- Private repo, publish-ready; flip only on explicit go