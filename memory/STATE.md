# STATE — gh-rs

## Current
Phase: v0.1.0 tagged + pushed (Sep 15 2026). Repo PUBLIC. Icon set shipped (README header renders, social preview deferred to owner click). Next: fresh-clone build check + README polish review.

## Verify log (Sep 15 2026, post-audit)
- fmt + clippy -D warnings + 11 tests pass, cargo doc zero warnings
- auth status / repo view regression: clean
- issue list PR filter: #1 real issue shown, PR #2 excluded ✓
- pr view merged render: [Open] → squash-merge → [Merged] ✓
- issue close + scratch repo deleted (404-confirmed)

## Icon log (Sep 15 2026)
- Nothing-style mark: OLED tile, dot-matrix `>_`, one yellow cursor pixel (badge accent)
- `assets/icon.svg` source + PNG 16/32/96/256/512 + `favicon.ico`; README header wired (96px)
- Pixel-verified: chevron/underscore off-white, cursor #EAB308, tile #0A0A0A

## Final batch log (Sep 15 2026, --json/--limit/repo ergonomics)
- 13 tests pass (clamp_limit 5 cases, summary serialize), clippy -D clean
- repo view bare inside kharcha clone → header correct ✓
- repo list --limit 2 --json parses; order pushed DESC (repomap/zcat/autoform top) ✓
- --limit 500 clamps to 100 (38 rows = full account) ✓
- issue/pr --json shapes parse (number/title/keys) ✓
- PR-filter holds with live PR present (issue list shows only #1) ✓
- scratch gh-json-probe deleted (404-confirmed)

## Progress
- [x] S0 setup
- [x] S1 auth + repo
- [x] S2 pr: create/list/view/diff/merge live-tested (scratch gh-pr-test-1 deleted)
- [x] S3 issue: create/list/view/close live-tested (scratch gh-issue-test-1 deleted)
- [x] Final pass items 1+2: README usage sections, repo view via render::repository
- [x] Benchmarks: gh 2.93.0 vs gh-rs (auth/repo/pr/issue + --help + binary size)

## Live-tested
- repo create: ✓ private-by-default, view/list/clone verified, test repo deleted. Note: deletion used system `gh` — repo delete is not a v0.1 command.
- pr: ✓ create(#1) → list → view → diff → squash-merge → list-empty on gh-pr-test-1; invalid method guard. Scratch repo deleted (404-confirmed).
- issue: ✓ create(#1-5) → list → view → close-all → list-empty on gh-issue-test-1; 404 guard on nonexistent. Scratch repo deleted (404-confirmed).

## Decisions locked
- Plain text output v0.1 (no TUI/pager)
- Repo create defaults private; --public opts out
- octocrab 0.54 (no serde-visible Repository struct — local RepositorySummary used)
- octocrab 0.54 dropped user_agent() — default "octocrab" UA is valid
- reqwest needs `form` + `json` features (cargo add without defaults)
- clap nested subcommands: named field + #[command(subcommand)]
- Private repo, publish-ready; flip only on explicit go