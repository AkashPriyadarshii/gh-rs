//! CLI: clap definitions + dispatch. One module per command group.

pub mod api;
pub mod auth;
pub mod gist;
pub mod issue;
pub mod pr;
pub mod release;
pub mod repo;
pub mod status;

use crate::error::AppError;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gh-rs", version, about = "GitHub CLI in Rust")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Make an authenticated GitHub API request
    Api(api::ApiArgs),
    /// Manage GitHub authentication
    Auth {
        #[command(subcommand)]
        command: auth::AuthArgs,
    },
    /// Manage gists
    Gist {
        #[command(subcommand)]
        command: gist::GistArgs,
    },
    /// Print relevant issues, PRs, and review requests across repositories
    Status(status::StatusArgs),
    /// Manage repositories
    Repo {
        #[command(subcommand)]
        command: repo::RepoArgs,
    },
    /// Manage pull requests
    Pr {
        #[command(subcommand)]
        command: pr::PrArgs,
    },
    /// Manage releases
    Release {
        #[command(subcommand)]
        command: release::ReleaseArgs,
    },
    /// Manage issues
    Issue {
        #[command(subcommand)]
        command: issue::IssueArgs,
    },
}

pub async fn run(cli: Cli) -> Result<(), AppError> {
    match cli.command {
        Command::Api(args) => api::run(args).await,
        Command::Auth { command } => auth::run(command).await,
        Command::Gist { command } => gist::run(command).await,
        Command::Status(args) => status::run(args).await,
        Command::Repo { command } => repo::run(command).await,
        Command::Pr { command } => pr::run(command).await,
        Command::Release { command } => release::run(command).await,
        Command::Issue { command } => issue::run(command).await,
    }
}
