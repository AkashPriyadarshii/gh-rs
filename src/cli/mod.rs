//! CLI: clap definitions + dispatch. One module per command group.

pub mod auth;
pub mod issue;
pub mod pr;
pub mod repo;

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
    /// Manage GitHub authentication
    Auth {
        #[command(subcommand)]
        command: auth::AuthArgs,
    },
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
    /// Manage issues
    Issue {
        #[command(subcommand)]
        command: issue::IssueArgs,
    },
}

pub async fn run(cli: Cli) -> Result<(), AppError> {
    match cli.command {
        Command::Auth { command } => auth::run(command).await,
        Command::Repo { command } => repo::run(command).await,
        Command::Pr { command } => pr::run(command).await,
        Command::Issue { command } => issue::run(command).await,
    }
}
