//! GitHub API glue: shared HTTP client, credential access (keyring), octocrab builder.
//! Specialized models live in sibling modules under the same crate.
//! Non-standard endpoints (device flow, etc.) use a raw `reqwest` escape hatch —
//! comment why per call.

use crate::error::AppError;
use octocrab::Octocrab;

/// OAuth App client_id — public by design (same model as `gh`). Not a secret.
pub const CLIENT_ID: &str = "Ov23liNghdlHihgqbM5V";

pub const KEYRING_SERVICE: &str = "gh-rs";
pub const KEYRING_USER: &str = "github.com";

/// Shared unauthenticated HTTP client for endpoints outside octocrab's typed API
/// (device flow, status probe). User-Agent is required by GitHub.
pub fn http() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("gh-rs/0.1.0")
        .build()
        .expect("reqwest client build is infallible with no special config")
}

pub fn load_token() -> Result<String, AppError> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)?;
    entry.get_password().map_err(|e| match e {
        keyring::Error::NoEntry => AppError::AuthMissing,
        other => AppError::Keyring(other.to_string()),
    })
}

pub fn store_token(token: &str) -> Result<(), AppError> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)?;
    entry.set_password(token)?;
    Ok(())
}

pub fn delete_token() -> Result<(), AppError> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()), // already logged out
        Err(e) => Err(AppError::Keyring(e.to_string())),
    }
}

/// Authenticated octocrab client. Token comes from the OS credential store.
pub fn api_client() -> Result<Octocrab, AppError> {
    let token = load_token()?;
    Octocrab::builder()
        .personal_token(token)
        // octocrab 0.54 sends its own "octocrab" UA — valid for GitHub's UA requirement.
        .build()
        .map_err(AppError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Real OS credential-store round trip — `#[ignore]`d because it writes the
    // actual keyring (local dev machines only): cargo test -- --ignored
    #[test]
    #[ignore]
    fn keyring_round_trip() {
        store_token("test-token-round-trip").unwrap();
        assert_eq!(load_token().unwrap(), "test-token-round-trip");
        delete_token().unwrap();
        assert!(matches!(load_token(), Err(AppError::AuthMissing)));
    }

    #[test]
    #[ignore]
    fn missing_auth_fails_fast() {
        // Requires clean keyring (no real token stored).
        let _ = delete_token();
        assert!(matches!(api_client(), Err(AppError::AuthMissing)));
    }
}
