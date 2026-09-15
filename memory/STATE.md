# STATE — gh-rs

## Current
Phase: Sprint 0 complete (code + docs + CI). Blocked on OAuth App registration (manual, needs browser). Ready for Sprint 1.

## Progress
- [x] S0 setup: cargo init (edition 2021), deps, error.rs, ci.yml, LICENSE, .gitignore, docs
- [ ] S1 auth + repo: not started — blocked on OAuth App client_id
- [ ] S2 pr: not started
- [ ] S3 issue: not started

## OAuth App — pending (only S0 blocker)
Register public OAuth App at https://github.com/settings/developers → OAuth Apps → New. Name `gh-rs`, Homepage URL any (e.g. https://github.com/AkashPriyadarshii/gh-rs), Authorization callback URL: http://localhost (device flow ignores it). Scope `repo`. Then supply `client_id` → hardcode as constant.

## Decisions locked
- Plain text output v0.1 (no TUI/pager)
- octocrab 0.54.2, reqwest 0.13 (json), tokio 1.53 (current_thread), keyring 4.2 (windows-native default), clap 4.6 derive, thiserror 2
- Private repo, publish-ready; flip only on explicit go