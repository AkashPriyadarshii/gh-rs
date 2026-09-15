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
gh-rs repo list
gh-rs repo view owner/repo

gh-rs pr create --title T [--body B] [--head H] [--base B] [--repo owner/repo]
gh-rs pr list [--repo owner/repo]           # --repo optional; git origin used otherwise
gh-rs pr view <number> [--repo owner/repo]
gh-rs pr diff <number> [--repo owner/repo]
gh-rs pr merge <number> [--method merge|squash|rebase] [--repo owner/repo]

gh-rs issue create --title T [--body B] [--repo owner/repo]
gh-rs issue list [--repo owner/repo]
gh-rs issue view <number> [--repo owner/repo]
gh-rs issue close <number> [--repo owner/repo]
```

Tokens live in the OS credential store (Windows Credential Manager / macOS Keychain) — never in files, logs, or the repo.

## Benchmarks

Measured on Windows 11 (release builds, 3 runs each, network included). gh 2.93.0 vs gh-rs v0.1. Cold runs include TLS handshake; subsequent runs are warm.

| Command | gh | gh-rs | Notes |
|---|---|---|---|
| `auth status` | 0.85s | 0.77s | both hit GET /user |
| `repo view` | 0.79–0.96s | 0.70–0.85s | gh-rs ~15% faster (slimmer decode) |
| `pr list` | 0.97–1.23s | 0.64–1.46s | gh-rs warm ~35% faster; first run slower (fresh TLS) |
| `issue list` | 0.95–1.08s | 0.67–0.72s | gh-rs ~30% faster (slim IssueSummary vs full model) |
| `--help` (no network) | 0.14s | 0.03s | gh-rs ~5x faster startup |

Binary size: gh 40.7 MB vs gh-rs 14.4 MB (~65% smaller — single-purpose, no TUI/pager/extension runtime).

Why gh-rs wins on API calls: it decodes only what it prints (slim local structs), skips gh's pager/color/TTY probing, and has no config-file discovery. Difference is tens of milliseconds of wall time dominated by network — don't oversell it.

## Why not just use `gh`?
Built to learn Rust with a real tool. Full command parity is intentionally out of scope — see PRD.md.

## License
MIT