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
        /// Output as JSON (agent-friendly)
        #[arg(long)]
        json: bool,
        /// Filter JSON output with embedded jaq/jq
        #[arg(long)]
        jq: Option<String>,
    },
    /// List open issues
    List {
        /// owner/repo (default: from git origin)
        #[arg(long)]
        repo: Option<String>,
        /// Max rows (1-1000, default 30)
        #[arg(long)]
        limit: Option<usize>,
        /// Output as JSON (agent-friendly)
        #[arg(long)]
        json: bool,
        /// Filter JSON output with embedded jaq/jq
        #[arg(long)]
        jq: Option<String>,
        /// Output plain tab-separated values without headers
        #[arg(long)]
        plain: bool,
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
        /// Filter JSON output with embedded jaq/jq
        #[arg(long)]
        jq: Option<String>,
        /// Output plain tab-separated values
        #[arg(long)]
        plain: bool,
    },
    /// Close an issue
    Close {
        /// Issue number
        number: u64,
        /// owner/repo (default: from git origin)
        #[arg(long)]
        repo: Option<String>,
        /// Output as JSON (agent-friendly)
        #[arg(long)]
        json: bool,
        /// Filter JSON output with embedded jaq/jq
        #[arg(long)]
        jq: Option<String>,
    },
}

pub async fn run(args: IssueArgs) -> Result<(), AppError> {
    match args {
        IssueArgs::Create {
            title,
            body,
            repo,
            json,
            jq,
        } => {
            let (owner, name) = resolve(repo.as_deref()).await?;
            let issue = api::create(&owner, &name, &title, body.as_deref()).await?;
            let out = render::format_output(
                &issue,
                json,
                jq.as_deref(),
                false,
                |i| format!("Created #{} {} → {owner}/{name}", i.number, i.title),
                None::<fn(&api::IssueSummary) -> String>,
            )?;
            println!("{out}");
            Ok(())
        }
        IssueArgs::List {
            repo,
            limit,
            json,
            jq,
            plain,
        } => {
            let (owner, name) = resolve(repo.as_deref()).await?;
            let issues = api::list(&owner, &name, crate::github::repo::clamp_limit(limit)).await?;
            if issues.is_empty() && !json && jq.is_none() {
                println!("No open issues in {owner}/{name}.");
                return Ok(());
            }
            let out = render::format_output(
                &issues,
                json,
                jq.as_deref(),
                plain,
                |list| {
                    let mut lines = Vec::new();
                    for issue in list {
                        lines.push(format!(
                            "#{:<5} {:<60} {}",
                            issue.number, issue.title, issue.user.login,
                        ));
                    }
                    lines.join("\n")
                },
                Some(|list: &Vec<api::IssueSummary>| {
                    let mut lines = Vec::new();
                    for issue in list {
                        lines.push(format!(
                            "{}\t{}\t{}\t{}",
                            issue.number, issue.title, issue.state, issue.user.login
                        ));
                    }
                    lines.join("\n")
                }),
            )?;
            if !out.is_empty() {
                println!("{out}");
            }
            Ok(())
        }
        IssueArgs::View {
            number,
            repo,
            json,
            jq,
            plain,
        } => {
            let (owner, name) = resolve(repo.as_deref()).await?;
            let issue = api::view(&owner, &name, number).await?;
            let out = render::format_output(
                &issue,
                json,
                jq.as_deref(),
                plain,
                render::issue,
                Some(|i: &api::IssueSummary| {
                    format!("{}\t{}\t{}\t{}", i.number, i.title, i.state, i.user.login)
                }),
            )?;
            println!("{out}");
            Ok(())
        }
        IssueArgs::Close {
            number,
            repo,
            json,
            jq,
        } => {
            let (owner, name) = resolve(repo.as_deref()).await?;
            let issue = api::close(&owner, &name, number).await?;
            let out = render::format_output(
                &issue,
                json,
                jq.as_deref(),
                false,
                |i| format!("Closed #{} {}.", i.number, i.title),
                None::<fn(&api::IssueSummary) -> String>,
            )?;
            println!("{out}");
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
