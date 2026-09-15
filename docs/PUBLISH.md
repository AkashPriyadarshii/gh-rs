# PUBLISH — private → public checklist

Repo is private, always publish-ready. Run this checklist before any visibility flip. Do NOT flip without explicit user "go" — a pushed public repo is irreversible.

## Before flip
1. **Secrets** — `rg "GH_RS_TEST_TOKEN|password|token" .` — no test tokens, no personal URLs in source, docs, or commit history.
2. **Shell history / CI logs** — confirm no token was ever echoed. `auth status` never prints tokens (by design).
3. **OAuth client_id** — public-by-design, secret-less. Safe to publish. No action.
4. **Docs** — README install/usage/feature list current; CHANGELOG-style sprint history in TASKS.md is accurate.
5. **License** — LICENSE file present, copyright line correct.
6. **Tag** — v0.1.0 tagged at a green commit (CI passing).
7. **Repo metadata** — `gh repo edit` description (formula: what + for whom + stack + features), topics (fill to ~20), homepage if any, social preview image (`assets/icon-512.png` via About settings).
8. `.gitignore` committed — target/ never in history.

## Flip
```
gh repo edit --visibility public
```

## After flip
1. Fresh clone, `cargo build --release` on Windows — clean.
2. `gh-rs auth login` + one real command round-trip from the fresh machine.
3. Confirm GitHub repo has no accidental private data (unpushed refs, artifacts).