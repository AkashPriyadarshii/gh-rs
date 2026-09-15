# gh-rs

GitHub CLI written in Rust. Personal daily-driver + Rust learning project — scoped to `auth`/`repo`/`pr`/`issue`, not a full `gh` replacement.

## Status
v0.1 in progress — see TASKS.md. Repo is private but publish-ready.

## Install
```
git clone https://github.com/AkashPriyadarshii/gh-rs
cd gh-rs
cargo build --release
```

## Usage
```
gh-rs auth login
gh-rs repo clone owner/repo
gh-rs pr list
gh-rs issue create
```

## Why not just use `gh`?
Built to learn Rust with a real tool. Full command parity is intentionally out of scope — see PRD.md.

## License
MIT