# PRD — gh-rs

## What
CLI GitHub client in Rust. Personal daily-driver + Rust systems-programming learning project + portfolio piece.

## Who
Single user: Akash (@AkashPriyadarshii). No external users assumed for v0.1.

## Why
Learn Rust via a real, useful tool instead of toy exercises. Full parity with `gh` is not the goal.

## v0.1 Scope (ponytail-scoped)
- `auth`: login (OAuth device flow), logout, status
- `repo`: clone, create, list, view
- `pr`: create, list, view, diff, merge
- `issue`: create, list, view, close

## Explicitly Out of Scope (v0.1)
- release, workflow/run, codespace, extension, attestation, project, discussion, gist, org, ssh-key, ruleset — full parity ≈ 65-70 solo-weeks, cut
- Multi-user / team features
- Linux — best-effort only, Windows + macOS are the real targets

## Success Criteria
- Replaces `gh` daily driver for the 4 command groups above
- Public repo, portfolio-ready
- Ships as 3 sprints, not one big-bang release

## Constraints
- ₹0 budget, GitHub API free tier only (5000 req/hr authenticated — plenty for 1 user)
- Dev machine: Windows 11, 8GB RAM