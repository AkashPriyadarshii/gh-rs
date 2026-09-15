//! `gh-rs auth` — device-flow login, logout, status.

use crate::error::AppError;
use crate::github::auth;
use crate::github::client::{http, load_token};
use clap::Subcommand;
use reqwest::StatusCode;

#[derive(Subcommand)]
pub enum AuthArgs {
    /// Log in via GitHub OAuth device flow
    Login,
    /// Remove the stored credential
    Logout,
    /// Check login state (ok / expired / not logged in)
    Status,
}

pub async fn run(args: AuthArgs) -> Result<(), AppError> {
    match args {
        AuthArgs::Login => auth::login().await,
        AuthArgs::Logout => {
            crate::github::client::delete_token()?;
            println!("Logged out.");
            Ok(())
        }
        AuthArgs::Status => status().await,
    }
}

async fn status() -> Result<(), AppError> {
    let token = match load_token() {
        Err(AppError::AuthMissing) => {
            println!("not logged in — run `gh-rs auth login`");
            return Ok(());
        }
        Err(e) => return Err(e),
        Ok(t) => t,
    };

    let client = http();
    let resp = client
        .get("https://api.github.com/user")
        .bearer_auth(token)
        .send()
        .await?;

    match resp.status() {
        StatusCode::OK => {
            #[derive(serde::Deserialize)]
            struct Who {
                login: String,
            }
            let who: Who = resp.json().await?;
            println!("Logged in as {}.", who.login);
            Ok(())
        }
        StatusCode::UNAUTHORIZED => {
            println!("Logged out (token missing, revoked or expired) — run `gh-rs auth login`");
            Ok(())
        }
        s => Err(AppError::GitHubApi(format!("unexpected /user status {s}"))),
    }
}
