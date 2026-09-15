//! `gh-rs issue` — create, list, view, close.
//! Target repo: explicit `--repo owner/repo` wins, else derived from git origin.

use crate::cli::repo::split_repo;
use crate::error::AppError;
use crate::git;
use crate::github::issue as api;
use crate::render;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum IssueArgs {
    /// Create an issue
    Create {
        /// Title
        #[arg(long)]
        title: String,
        /// Body
        #[arg(long)]
        body: Option<String>,
        /// owner/repo (default: from git origin)
        #[arg(long)]
        repo: Option<String>,
    },
    /// List open issues
    List {
        /// owner/repo (default: from git origin)
        #[arg(long)]
        repo: Option<String>,
        /// Max rows (1-100, default 30)
        #[arg(long)]
        limit: Option<u32>,
        /// Output as JSON (agent-friendly)
        #[arg(long)]
        json: bool,
    },
    /// Show issue details
    View {
        /// Issue number
        number: u64,
        /// owner/repo (default: from git origin)
        #[arg(long)]
        repo: Option<String>,
        /// Output as JSON (agent-friendly)
        #[arg(long)]
        json: bool,
    },
    /// Close an issue
    Close {
        /// Issue number
        number: u64,
        /// owner/repo (default: from git origin)
        #[arg(long)]
        repo: Option<String>,
    },
}

pub async fn run(args: IssueArgs) -> Result<(), AppError> {
    match args {
        IssueArgs::Create { title, body, repo } => {
            let (owner, name) = resolve(repo.as_deref()).await?;
            let issue = api::create(&owner, &name, &title, body.as_deref()).await?;
            println!("Created #{} {} → {owner}/{name}", issue.number, issue.title);
            Ok(())
        }
        IssueArgs::List { repo, limit, json } => {
            let (owner, name) = resolve(repo.as_deref()).await?;
            let issues = api::list(&owner, &name, crate::github::repo::clamp_limit(limit)).await?;
            if issues.is_empty() {
                println!("No open issues in {owner}/{name}.");
                return Ok(());
            }
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&issues).unwrap_or_default()
                );
                return Ok(());
            }
            for issue in issues {
                println!(
                    "#{:<5} {:<60} {}",
                    issue.number, issue.title, issue.user.login,
                );
            }
            Ok(())
        }
        IssueArgs::View { number, repo, json } => {
            let (owner, name) = resolve(repo.as_deref()).await?;
            let issue = api::view(&owner, &name, number).await?;
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&issue).unwrap_or_default()
                );
            } else {
                print!("{}", render::issue(&issue));
            }
            Ok(())
        }
        IssueArgs::Close { number, repo } => {
            let (owner, name) = resolve(repo.as_deref()).await?;
            let issue = api::close(&owner, &name, number).await?;
            println!("Closed #{} {}.", issue.number, issue.title);
            Ok(())
        }
    }
}

async fn resolve(explicit: Option<&str>) -> Result<(String, String), AppError> {
    match explicit {
        Some(repo) => split_repo(repo),
        None => git::current_repo(),
    }
}

#[cfg(test)]
mod tests {
    use super::resolve;

    #[tokio::test]
    async fn explicit_repo_wins() {
        let (o, r) = resolve(Some("AkashPriyadarshii/kharcha")).await.unwrap();
        assert_eq!(o, "AkashPriyadarshii");
        assert_eq!(r, "kharcha");
    }

    #[tokio::test]
    async fn rejects_malformed_repo() {
        assert!(resolve(Some("norepo")).await.is_err());
    }
}
