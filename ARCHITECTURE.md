# ARCHITECTURE — gh-rs v0.1

## Stack
| Layer | Choice | Notes |
|---|---|---|
| Language | Rust | 2021 edition, stable toolchain |
| CLI parsing | clap (derive) | latest via `cargo add clap --features derive` |
| GitHub API | octocrab | latest via `cargo add octocrab` — do not pin old versions |
| Async runtime | tokio (current_thread) | latest 1.x |
| Credential storage | keyring | latest via `cargo add keyring` — v3 API differs from v2, verify at add time |
| Serialization | serde + serde_json | latest |
| Git operations | shell out to system `git` binary | n/a — no libgit2 |

## Module layout
```
src/
  main.rs        entry point, clap parse, dispatch
  cli/
    mod.rs       clap Command/Args defs
    auth.rs
    repo.rs
    pr.rs
    issue.rs
  github/
    client.rs    octocrab client init, token injection
    auth.rs      OAuth device flow
    repo.rs
    pr.rs
    issue.rs
  render.rs      shared fetch→render helper (plain text) — used by all view/diff commands
  git.rs         subprocess wrapper (clone, checkout)
  error.rs       unified error type
```

## Data flow
1. `main.rs` parses argv via clap → typed `Cli` struct
2. Dispatch to `cli::<group>::run()`
3. Builds authenticated `octocrab::Octocrab` client (token from `keyring`)
4. Calls typed API, or `_get`/`_post`/GraphQL escape hatch where no typed model exists
5. Result formatted to stdout (plain text v0.1)

## Auth flow
1. `gh-rs auth login` → OAuth device flow → user visits URL, enters code
2. Token stored via `keyring` (Windows Credential Manager / macOS Keychain)
3. Every other command loads token at startup, fails fast with clear error if missing
4. `auth status` hits `GET /user` with the stored token → ok / expired / revoked

## Error handling
- Single `AppError` enum (thiserror): AuthMissing, GitHubApi, GitSubprocess, InvalidInput
- No `unwrap()`/`expect()` outside tests and `main()`'s top-level error print
- GitHub API errors surface GitHub's actual message, not a generic wrapper
- Never log or stdout-echo the token — `auth status` prints only validity

## Testing
- Unit tests: dispatch + output formatting
- Integration tests: one per command group, gated behind `GH_RS_TEST_TOKEN` env var (skipped when unset), asserting GitHub's real error surfaces on invalid token
- No wiremock/mock-server dependency

## Gotchas
- octocrab needs a tokio runtime even for one sequential call — use `current_thread` flavor
- keyring backend differs per OS — Windows Credential Manager + macOS Keychain via OS-native backends
- Projects V2 / some GraphQL-only data isn't in octocrab's typed models — use GraphQL escape hatch, comment why per call
- OAuth device flow needs a registered OAuth App `client_id` (public, secret-less — safe to publish, same as gh does). Register "gh-rs" once before Sprint 1
- Doc comments only on non-obvious code (device flow, escape hatches) — not every public fn