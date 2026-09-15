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
- [ ] issue create / list / view / close
- [ ] Tests
- [ ] README final pass, tag v0.1.0