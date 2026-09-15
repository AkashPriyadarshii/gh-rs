# gh-rs

GitHub CLI written in Rust. Personal daily-driver + Rust learning project — scoped to `auth`/`repo`/`pr`/`issue`, not a full `gh` replacement.

## Status
v0.1 complete — all 4 command groups live-tested. Repo is private but publish-ready.

## Install
```
git clone https://github.com/AkashPriyadarshii/gh-rs
cd gh-rs
cargo build --release
```

## Usage
```
gh-rs auth login        # OAuth device flow — token stored in OS keychain
gh-rs auth status       # ok / expired / not logged in
gh-rs auth logout

gh-rs repo clone owner/repo [dir]
gh-rs repo create name [--public]   # private by default
gh-rs repo list [--limit N] [--json]           # newest push first, N=1-100 default 30
gh-rs repo view [owner/repo] [--json]          # bare = current repo from git origin

gh-rs pr create --title T [--body B] [--head H] [--base B] [--repo owner/repo]
gh-rs pr list [--repo owner/repo] [--limit N] [--json]
gh-rs pr view <number> [--repo owner/repo] [--json]
gh-rs pr diff <number> [--repo owner/repo]    # raw unified diff, always text
gh-rs pr merge <number> [--method merge|squash|rebase] [--repo owner/repo]

gh-rs issue create --title T [--body B] [--repo owner/repo]
gh-rs issue list [--repo owner/repo] [--limit N] [--json]
gh-rs issue view <number> [--repo owner/repo] [--json]
gh-rs issue close <number> [--repo owner/repo]
```

`--json` is list/view only (agent-friendly pretty JSON). Mutating commands stay human text; `pr diff` is already raw. `--limit` maps to a single `per_page` request (max 100, no cursor walker in v0.1).

Tokens live in the OS credential store (Windows Credential Manager / macOS Keychain) — never in files, logs, or the repo.

## Benchmarks

Measured on Windows 11 (release builds, 3 runs each, network included, Sep 2026 re-run after `[profile.release]` strip+lto+cgu=1). gh 2.93.0 vs gh-rs v0.1. Network dominates both — deltas are tens of ms, don't oversell.

| Command | gh | gh-rs | Notes |
|---|---|---|---|
| `auth status` | 699–768ms | 622–766ms | tie; both hit GET /user |
| `repo view` | 684–749ms | 783–798ms | gh ~7% faster this run (earlier run had gh-rs ~15% ahead — noise) |
| `pr list` | 706–812ms | 636–723ms | gh-rs ~10% faster |
| `issue list` | 714–877ms | 741–752ms | tie; gh variance higher |
| `--help` (no network) | 102–105ms | 49–65ms | gh-rs ~2x faster startup |

Binary size: gh 40.7 MB vs gh-rs 12.4 MB (~70% smaller — single-purpose, no TUI/pager/extension runtime). Release profile (`strip/lto/cgu=1`) cut gh-rs from 14.4 MB → 12.4 MB; `panic="abort"` deliberately skipped (backtraces beat ~1MB).

Honest read: both CLIs spend ~700ms on TLS+API round trip. gh-rs wins startup and binary size clearly; per-command API latency is a wash within noise. Slim local structs (RepositorySummary/IssueSummary) keep decode cheap but the network owns the total.

## Why not just use `gh`?
Built to learn Rust with a real tool. Full command parity is intentionally out of scope — see PRD.md.

## License
MIT