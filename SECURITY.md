# Security Policy

Report vulnerabilities via a [private security advisory](../../security/advisories/new) or a GitHub issue. Single maintainer, best-effort response.

Scope note: `src/github/client.rs` holds a public OAuth App `client_id`. This is by design (same model as `gh`), not a leak. Tokens live in the OS keychain, never in files or logs.

Platform note: Linux token storage needs a Secret Service on D-Bus (GNOME Keyring, KWallet, KeePassXC) or `auth login` fails at the keyring layer. Keyring-backed tests are `#[ignore]`d for this reason — CI runs pure-logic tests only.
