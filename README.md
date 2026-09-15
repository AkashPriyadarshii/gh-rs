# gh-rs

GitHub CLI written in Rust. Personal daily-driver + Rust learning project — scoped to `auth`/`repo`/`pr`/`issue`, not a full `gh` replacement.

## Status
Sprint 1 done — `auth` + `repo` working. `pr` (S2) and `issue` (S3) pending. Repo is private but publish-ready.

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
```

Tokens live in the OS credential store (Windows Credential Manager / macOS Keychain) — never in files, logs, or the repo.

## Why not just use `gh`?
Built to learn Rust with a real tool. Full command parity is intentionally out of scope — see PRD.md.

## License
MIT