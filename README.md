<!--
Title: gh-rs, GitHub CLI in Rust (auth, repo, pr, issue)
Description: gh-rs is a small GitHub CLI written in Rust. OAuth device flow with OS keychain storage, repo/pr/issue commands with plain text and JSON output.
Keywords: github cli, rust cli, octocrab, github api, git, pull request cli, keyring, device flow, clap
-->

<div align="center">

<img src="assets/icon-96.png" alt="gh-rs icon" width="96" />

# gh-rs

**Support:** fuel the next build — [![Buy Me a Coffee](https://img.shields.io/badge/Buy%20Me%20a%20Coffee-ffdd00?style=for-the-badge&logo=buy-me-a-coffee&logoColor=black)](https://buymeacoffee.com/AkashPriyadarshi)

**GitHub CLI in Rust. Four commands. No bloat.**

[![Crates.io](https://img.shields.io/crates/v/gh-rs.svg?style=flat-square)](https://crates.io/crates/gh-rs)
[![Downloads](https://img.shields.io/crates/d/gh-rs.svg?style=flat-square)](https://crates.io/crates/gh-rs)
[![CI](https://github.com/AkashPriyadarshii/gh-rs/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/AkashPriyadarshii/gh-rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2021-orange?style=flat-square)](https://www.rust-lang.org)

*Built by [Akash Priyadarshi](https://github.com/AkashPriyadarshii) · [Website](https://akashpriyadarshii.github.io/gh-rs/)*

[Why](#why) · [Install](#install) · [Quickstart](#quickstart) · [Commands](#commands) · [Benchmarks](#benchmarks) · [Architecture](#architecture) · [Non-goals](#non-goals)

*Fuel the next build:* 

</div>

[![crates.io](https://img.shields.io/crates/v/gh-rs?style=flat-square)](https://crates.io/crates/gh-rs) [![downloads](https://img.shields.io/crates/d/gh-rs?style=flat-square)](https://crates.io/crates/gh-rs) [![release](https://img.shields.io/github/v/release/AkashPriyadarshii/gh-rs?style=flat-square&label=release)](https://github.com/AkashPriyadarshii/gh-rs/releases)

## Why

`gh` does everything. gh-rs does four things: `auth`, `repo`, `pr`, `issue`.

- **You want a fast binary.** 12.4 MB. `--help` in ~50ms.
- **You want JSON for agents.** Every list/view takes `--json`. Pipe it to `jq`.
- **You want your token out of dotfiles.** Device flow puts it in the OS keychain. Nothing lands in `~/.config`.
- **You want readable code.** ~15 files. Each command is one `cli/` module plus one `github/` module.

## Install

```sh
# From crates.io
cargo install gh-rs

# Or build from source
git clone https://github.com/AkashPriyadarshii/gh-rs
cd gh-rs
cargo build --release
```

## Quickstart

```sh
gh-rs auth login
gh-rs repo list
```

```
AkashPriyadarshii/repomap                public  Go             2026-09-15T07:01:50Z
AkashPriyadarshii/zcat                   public  Zig            2026-09-15T06:24:20Z
AkashPriyadarshii/autoform-pro           private JavaScript     2026-09-15T05:33:23Z
```

```sh
gh-rs repo view AkashPriyadarshii/kharcha
```

```
AkashPriyadarshii/kharcha (public)

  Kharcha — India-first UPI expense tracker for Android by Akash Priyadarshi. Flutter/Dart app: every UPI payment (GPay, PhonePe, Paytm) auto-appears as an expense. Offline-first (Drift/SQLite), rule-based (no AI), optional Supabase sync.

  language:       Kotlin
  stars:          1
  default branch: main
  pushed:         2026-09-12T04:58:05Z
  clone:          https://github.com/AkashPriyadarshii/kharcha.git
  web:            https://github.com/AkashPriyadarshii/kharcha
```

## Commands

```sh
gh-rs auth login        # OAuth device flow, token goes to the OS keychain
gh-rs auth status       # ok / expired / not logged in
gh-rs auth logout       # delete the stored credential

gh-rs repo clone owner/repo [dir]
gh-rs repo create name [--public]        # private unless --public
gh-rs repo list [--limit N] [--json]     # newest push first
gh-rs repo view [owner/repo] [--json]    # bare means current repo from git origin

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

`--json` exists on list/view only. Mutating commands print human text. `--limit` is 1-100, default 30, one `per_page` request.

### Flags

| Command | Flag | Default | Meaning |
|---|---|---|---|
| `repo/pr/issue list` | `--limit N` | `30` | Rows returned, clamped to 1-100 |
| `repo/pr/issue list/view` | `--json` | off | Pretty JSON instead of text |
| `repo create` | `--public` | off (private) | Create as public repo |
| `pr create` | `--head H` | current branch | Source branch |
| `pr create` | `--base B` | repo default | Target branch |
| `pr merge` | `--method M` | `merge` | `merge`, `squash`, or `rebase` |
| `pr/issue *`, `repo view` | `--repo owner/repo` | git origin | Target repo, explicit wins |

Actual `--help` output (binary built from this tree):

```
GitHub CLI in Rust

Usage: gh-rs.exe <COMMAND>

Commands:
  auth   Manage GitHub authentication
  repo   Manage repositories
  pr     Manage pull requests
  issue  Manage issues
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

```
Manage GitHub authentication

Usage: gh-rs.exe auth <COMMAND>

Commands:
  login   Log in via GitHub OAuth device flow
  logout  Remove the stored credential
  status  Check login state (ok / expired / not logged in)
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

```
Manage repositories

Usage: gh-rs.exe repo <COMMAND>

Commands:
  clone   Clone a repository with the system git
  create  Create a repository under your account (private unless --public)
  list    List your repositories (newest push first)
  view    Show repository details (default: current repo from git origin)
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

```
Show repository details (default: current repo from git origin)

Usage: gh-rs.exe repo view [OPTIONS] [REPO]

Arguments:
  [REPO]  owner/repo (default: from git origin)

Options:
      --json  Output as JSON (agent-friendly)
  -h, --help  Print help
```

```
List your repositories (newest push first)

Usage: gh-rs.exe repo list [OPTIONS]

Options:
      --limit <LIMIT>  Max rows (1-100, default 30)
      --json           Output as JSON (agent-friendly)
  -h, --help           Print help
```

```
Manage pull requests

Usage: gh-rs.exe pr <COMMAND>

Commands:
  create  Create a pull request (head = current branch or --head, base = --base or repo default)
  list    List open pull requests
  view    Show pull request details
  diff    Show the unified diff
  merge   Merge a pull request
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

```
List open pull requests

Usage: gh-rs.exe pr list [OPTIONS]

Options:
      --repo <REPO>    owner/repo (default: from git origin)
      --limit <LIMIT>  Max rows (1-100, default 30)
      --json           Output as JSON (agent-friendly)
  -h, --help           Print help
```

```
Manage issues

Usage: gh-rs.exe issue <COMMAND>

Commands:
  create  Create an issue
  list    List open issues
  view    Show issue details
  close   Close an issue
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## Benchmarks

Windows 11, 8GB, release build, 7 runs each (warmup + outlier dropped), live token, network included. Python `subprocess` timing. gh 2.93.0 vs gh-rs v0.1. Medians.

| Command | gh | gh-rs | delta |
|---|---|---|---|
| `auth status` | 516.8ms | 516.0ms | tie, both hit GET /user |
| `repo view` | 950.7ms | 584.0ms | gh-rs ~38% faster |
| `pr list` | 642.0ms | 561.5ms | gh-rs ~12% faster |
| `issue list` | 640.0ms | 594.5ms | gh-rs ~7% faster |
| `--help`, no network | 56.9ms | 8.4ms | gh-rs ~6.8x faster startup |

Binary: gh 40.7 MB vs gh-rs 10.9 MB. Release profile (`strip`+`lto`+`cgu=1`) cut gh-rs from 14.4 MB. `panic="abort"` skipped on purpose, backtraces beat ~1 MB.

Per-command latency is dominated by TLS plus API round trip (~500ms of the ~600ms). gh-rs wins startup and size, and edges the API-bound commands on top — the octocrab client is cached per process, so one invocation makes one keyring read and one client build.

## Architecture

```
src/main.rs            # tokio current_thread entry, error print
src/error.rs           # AppError enum, all fallible paths return it
src/cli/mod.rs         # clap Command enum + dispatch
src/cli/auth.rs        # auth login/logout/status
src/cli/repo.rs        # repo clone/create/list/view + shared split_repo
src/cli/pr.rs          # pr create/list/view/diff/merge
src/cli/issue.rs       # issue create/list/view/close
src/github/mod.rs      # module list
src/github/client.rs   # keyring token, octocrab builder, cached reqwest client
src/github/auth.rs     # OAuth device flow + login probe
src/github/repo.rs     # RepositorySummary + CRUD + clamp_limit
src/github/pr.rs       # typed pulls + raw REST for create/diff
src/github/issue.rs    # IssueSummary + raw REST (octocrab Issue overflows stack)
src/git.rs             # git subprocess: clone with Bearer header, origin parse
src/render.rs          # plain-text renderers for repo/pr/issue view
```

Tokens live in the OS credential store (Windows Credential Manager, macOS Keychain). The OAuth client ID is public by design, same model as `gh`.

## Non-goals

v0.1 refuses: full `gh` parity (actions, releases, gists, secrets, search), TUI or pager, cursor pagination past `--limit 100`, `--jq` filtering, JSON on mutating commands, `panic="abort"`, GraphQL.

## Ecosystem

- [design-genius](https://github.com/AkashPriyadarshii/design-genius)
- [akash-design-engineering](https://github.com/AkashPriyadarshii/akash-design-engineering)
- [tdlib-android](https://github.com/AkashPriyadarshii/tdlib-android)
- [kharcha](https://github.com/AkashPriyadarshii/kharcha)

## Author

**Akash Priyadarshi** (Patna, Bihar, India)

[GitHub](https://github.com/AkashPriyadarshii) · [Portfolio](https://akashpriyadarshi.vercel.app) · [LinkedIn](https://linkedin.com/in/akashpriyadarshii) · [Resume](https://akashpriyadarshii.github.io/Resume/)

## Social

[X / Twitter](https://x.com/Akash__ydv001) · [Threads](https://www.threads.com/@free_dev2026) · [Instagram](https://www.instagram.com/akash.priyadarshii/) · [Reddit](https://reddit.com/user/akashpriyadarshi)

*Rust CLI for GitHub: auth, repos, pull requests, issues. Small on purpose.*
