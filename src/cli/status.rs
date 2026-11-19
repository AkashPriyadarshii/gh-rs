//! `gh-rs status` — Print relevant issues, PRs, and review requests across repos.

use crate::error::AppError;
use crate::github::status as api;
use crate::render;
use clap::Args;

#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Limit status results to a specific organization
    #[arg(short = 'o', long)]
    pub org: Option<String>,

    /// Repositories to exclude in owner/repo format
    #[arg(short = 'e', long = "exclude")]
    pub exclude: Vec<String>,

    /// Output as JSON (agent-friendly)
    #[arg(long)]
    pub json: bool,

    /// Filter JSON output with embedded jaq/jq
    #[arg(long)]
    pub jq: Option<String>,

    /// Output plain tab-separated values without headers
    #[arg(long)]
    pub plain: bool,
}

pub async fn run(args: StatusArgs) -> Result<(), AppError> {
    let report = api::fetch_status(args.org.as_deref(), &args.exclude).await?;

    let out = render::format_output(
        &report,
        args.json,
        args.jq.as_deref(),
        args.plain,
        render::status_report,
        Some(|rep: &api::StatusReport| -> String {
            let mut lines = Vec::new();
            for it in &rep.assigned_issues {
                lines.push(format!("issue\t{}\t{}\t{}", it.repo, it.number, it.title));
            }
            for it in &rep.review_requests {
                lines.push(format!("review\t{}\t{}\t{}", it.repo, it.number, it.title));
            }
            for it in &rep.my_prs {
                lines.push(format!("pr\t{}\t{}\t{}", it.repo, it.number, it.title));
            }
            lines.join("\n")
        }),
    )?;

    println!("{out}");
    Ok(())
}
