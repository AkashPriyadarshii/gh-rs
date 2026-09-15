//! OAuth device authorization grant (device flow) + auth-state probe.
//! Endpoint docs: https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps#device-flow
//! Not in octocrab's typed API → reqwest escape hatch.

use crate::error::AppError;
use crate::github::client::{http, store_token, CLIENT_ID};
use std::time::Duration;
use tokio::time::sleep;

const DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const ACCESS_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:device_code";

#[derive(serde::Deserialize, Debug)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
}

#[derive(serde::Deserialize, Debug)]
struct AccessTokenResponse {
    access_token: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

/// Run the device flow to completion: request a code, print it, poll until the
/// user authorizes in the browser, then persist the token via keyring.
pub async fn login() -> Result<(), AppError> {
    let client = http();

    let device: DeviceCodeResponse = client
        .post(DEVICE_CODE_URL)
        .header("Accept", "application/json")
        .form(&[("client_id", CLIENT_ID), ("scope", "repo")])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    println!("! First copy your one-time code: {}", device.user_code);
    println!(
        "- Open {} in your browser and paste the code",
        device.verification_uri
    );
    println!("- Waiting for authorization…");

    let mut interval = Duration::from_secs(device.interval.max(5));
    let deadline = Duration::from_secs(device.expires_in);
    let mut waited = Duration::ZERO;

    loop {
        sleep(interval).await;
        waited += interval;
        if waited > deadline {
            return Err(AppError::InvalidInput(
                "device code expired — run `gh-rs auth login` again".into(),
            ));
        }

        let resp: AccessTokenResponse = client
            .post(ACCESS_TOKEN_URL)
            .header("Accept", "application/json")
            .form(&[
                ("client_id", CLIENT_ID),
                ("device_code", &device.device_code),
                ("grant_type", GRANT_TYPE),
            ])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        if let Some(token) = resp.access_token {
            let login = probe_login(&client, &token).await?;
            store_token(&token)?;
            println!("Logged in as {login}.");
            return Ok(());
        }

        match resp.error.as_deref() {
            Some("authorization_pending") => {} // keep polling
            Some("slow_down") => {
                interval += Duration::from_secs(5);
            }
            Some(other) => {
                let detail = resp.error_description.as_deref().unwrap_or(other);
                return Err(AppError::GitHubApi(detail.to_string()));
            }
            None => return Err(AppError::GitHubApi("empty access-token response".into())),
        }
    }
}

/// Resolve the account name for a fresh device-flow token (GET /user).
/// Rejects revoked/expired tokens before persisting them.
async fn probe_login(client: &reqwest::Client, token: &str) -> Result<String, AppError> {
    #[derive(serde::Deserialize)]
    struct Who {
        login: String,
    }
    let who: Who = client
        .get("https://api.github.com/user")
        .bearer_auth(token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(who.login)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Pure serde decode — CI-safe, no network/keyring.
    #[test]
    fn parses_device_code_response() {
        let json = r#"{"device_code":"dc123","user_code":"ABCD-EFGH","verification_uri":"https://github.com/login/device","expires_in":900,"interval":5}"#;
        let parsed: DeviceCodeResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.user_code, "ABCD-EFGH");
        assert_eq!(parsed.interval, 5);
        assert_eq!(parsed.expires_in, 900);
    }

    #[test]
    fn parses_pending_and_success_tokens() {
        let pending: AccessTokenResponse =
            serde_json::from_str(r#"{"error":"authorization_pending"}"#).unwrap();
        assert_eq!(pending.access_token, None);
        assert_eq!(pending.error.as_deref(), Some("authorization_pending"));

        let ok: AccessTokenResponse = serde_json::from_str(
            r#"{"access_token":"tok123","token_type":"bearer","scope":"repo"}"#,
        )
        .unwrap();
        assert_eq!(ok.access_token.as_deref(), Some("tok123"));
    }
}
