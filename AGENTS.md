# AGENTS.md — gh-rs v0.1

## Build / test / run
- Build: `cargo build`
- Test: `cargo test`
- Run: `cargo run -- <command>`
- Lint: `cargo clippy -- -D warnings`
- Format: `cargo fmt`

## Conventions
- Edition 2021, stable toolchain
- One module per command group (see ARCHITECTURE.md)
- Errors via `Result<T, AppError>`, never `unwrap()`/`expect()` outside tests
- Never echo or log the auth token — `auth status` prints only validity
- Commits: conventional commits (`feat:`, `fix:`, `chore:`)

## Forbidden
- No hardcoded tokens/API keys/credentials anywhere in source
- No plaintext credential storage — `keyring` only
- No new dependency without checking: maintained <6mo, MIT/Apache-2.0 license
- No scope creep past PRD.md's v0.1 command list

## Skills — when to use
Installed at ~/.agents/skills/ (global). Read SKILL.md before first use.
- rust-async-patterns — any tokio/async code (octocrab client, device-flow polling)
- rust-best-practices — AppError design, Rust idioms, code review
- rust-testing — test strategy incl. env-gated integration tests (GH_RS_TEST_TOKEN)
- gh-cli (trailofbits/skills) — gh behavior + security-aware use reference for parity on auth/repo/pr/issue
- create-cli, unix-cli-best-practices — CLI structure, exit codes, stdout/stderr discipline
- configuring-oauth2-authorization-flow, testing-oauth2-implementation-flaws — auth paths
- performing-oauth-scope-minimization-review — keep scope at `repo` minimum
- github-actions-multiplatform-release — release CI when publishing

## Reference
PRD.md → what/why. ARCHITECTURE.md → stack/layout/flow. TASKS.md → current sprint.

## Repo status
Public since v0.1.0 (Sep 2026). Security rules above are the reason — keep them strict.