# TASKS — gh-rs v0.1

## Sprint 0 — setup
- [x] `cargo init`, add clap/octocrab/tokio/keyring/serde via `cargo add`
- [x] Register "gh-rs" OAuth App (public client_id, scope `repo`) — client_id Ov23liNghdlHihgqbM5V
- [x] error.rs: AppError enum
- [x] .gitignore (target/, *.log, .env), LICENSE (MIT, copyright line)
- [x] .github/workflows/ci.yml: fmt --check, clippy -D warnings, test on every push
- [x] docs/PUBLISH.md, memory/STATE.md

## Sprint 1 — auth + repo
- [x] github/client.rs: Octocrab client builder, token injection
- [x] Device-flow roundtrip FIRST, alone: login → store in keyring → status ✅
- [x] auth login (OAuth device flow, store in keyring)
- [x] auth logout
- [x] auth status (state: ok/expired/revoked via GET /user)
- [x] repo clone <owner/repo>
- [x] repo create <name> — live-tested: private-by-default ✓ view ✓ list ✓ clone ✓, cleanup via gh repo delete
- [x] repo list
- [x] repo view <owner/repo>
- [x] Tests: serde decode units + #[ignore] keyring round-trip + missing-auth
- [x] README v0.1: install + usage for auth/repo
- [ ] **CHECKPOINT — honest review:** does `gh-rs auth` + `gh-rs repo` feel better than `gh` daily? Yes → continue to Sprint 2. No → ship S1 as portfolio piece, call it v0.1, stop.

## Sprint 2 — pr
- [x] pr create / list / view / diff / merge — live-tested end-to-end (private gh-pr-test-1: create → list → view → diff → squash-merge → list-empty; invalid method guard; scratch repo deleted)
- [x] Tests

## Sprint 3 — issue
- [x] issue create / list / view / close — live-tested end-to-end (private gh-issue-test-1: create #1-5 → list → view → close all → list-empty; 404 guard on nonexistent; scratch repo deleted)
- [x] Tests
- [x] README final pass (usage + benchmarks)
- [x] tag v0.1.0 (pushed Sep 15 2026)

## Audit batch (post-S3 review)
- [x] issue list filters PRs (pull_request key) — live-verified on gh-verify-audit (#1 shown, PR #2 excluded)
- [x] device-flow poll decodes RFC 8628 errors before status check
- [x] repo clone sends Bearer extraHeader from keyring (private repos, zero git-credential setup)
- [x] render::pull_request labels merged PRs [Merged] — live-verified (#2 Open → squash → [Merged])
- [x] git origin parse: trailing slash + .git order, shared parse_origin + empty-segment guard
- [x] shared cli::repo::split_repo (pr/issue resolve deduped)
- [x] http() cached via OnceLock; CI concurrency + workflow_dispatch; bare-URL doc fix
- [x] Cargo [profile.release]: strip/lto/cgu=1 (14.4 → 12.4 MB); panic=abort rejected (backtraces)
- [x] STATE.md synced; benchmarks re-run Sep 2026 (honest: API latency wash, startup/size wins)

## v0.1 final batch (--json / --limit / repo ergonomics)
- [x] --json on repo/pr/issue list+view (Serialize on summaries, octocrab PullRequest already Serialize; pretty print; live-verified gh-json-probe: issue/pr/repo shapes parse)
- [x] --limit on repo/pr/issue list (clamp_limit 1-100 default 30; single per_page request; live: --limit 2/5/500→100 rows)
- [x] repo view bare positional defaults to git origin (live-verified inside kharcha clone)
- [x] repo list sort=pushed DESC (API) + local belt-and-braces sort; newest repo surfaces first
- [x] ponytail markers: no cursor pagination (all three lists), ps-argv clone note kept
- [x] README usage + flags documented

## Deferred to v0.2 (explicitly rejected for v0.1)
- Full cursor pagination (Link-header walker); --jq engine; JSON on mutating cmds; panic=abort; 80%+ coverage + GH_RS_TEST_TOKEN harness