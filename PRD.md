# PRD — gh-rs

## What
CLI GitHub client in Rust. Personal daily-driver + Rust systems-programming learning project + portfolio piece.

## Who
Single user: Akash (@AkashPriyadarshii). No external users assumed for v0.1.

## Why
Learn Rust via a real, useful tool instead of toy exercises. Full parity with `gh` is not the goal.

## Scope Progression
- **v0.1**: `auth` (OAuth device flow, keyring), `repo` (clone, create, list, view), `pr` (create, list, view, diff, merge), `issue` (create, list, view, close).
- **v0.2**: Universal `--json`, embedded `--jq` (pure Rust `jaq`), multi-page walker (`--limit 1-1000`), `--plain` TSV mode.
- **v0.3**: `api` (arbitrary REST/GraphQL escape hatch with RFC 5988 Link pagination), `status` (cross-repo dashboard), `release` (list, view, create, delete, download), `gist` (list, view, create, delete).

## Explicitly Out of Scope
- Interactive TUI widgets, heavy background daemons, telemetry, `panic="abort"`.
- Multi-user / team features
- Linux — best-effort only, Windows + macOS are the real targets

## Success Criteria
- Replaces `gh` daily driver for the 4 command groups above
- Public repo, portfolio-ready
- Ships as 3 sprints, not one big-bang release

## Constraints
- GitHub API free tier only (5000 req/hr authenticated)
- Targets: Windows + macOS