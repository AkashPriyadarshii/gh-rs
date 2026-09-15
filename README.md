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

## Why not just use `gh`?
Built to learn Rust with a real tool. Full command parity is intentionally out of scope — see PRD.md.

## License
MIT