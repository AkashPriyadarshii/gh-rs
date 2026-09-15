//! Shared plain-text renderers for view/diff commands (repo, pr, issue).
//! One fetch → render shape keeps output consistent across command groups.

use crate::github::issue::IssueSummary;
use crate::github::repo::RepositorySummary;
use octocrab::models::pulls::PullRequest;

pub fn repository(r: &RepositorySummary) -> String {
    let vis = if r.private { "private" } else { "public" };
    let mut out = format!("{} ({vis})", r.full_name);
    if let Some(d) = &r.description {
        out.push_str(&format!("\n\n  {d}"));
    }
    out.push_str("\n\n");
    out.push_str(&format!(
        "  language:       {}\n",
        r.language.as_deref().unwrap_or("-")
    ));
    out.push_str(&format!("  stars:          {}\n", r.stargazers_count));
    out.push_str(&format!("  default branch: {}\n", r.default_branch));
    out.push_str(&format!(
        "  pushed:         {}\n",
        r.pushed_at.as_deref().unwrap_or("-")
    ));
    out.push_str(&format!("  clone:          {}\n", r.clone_url));
    out.push_str(&format!("  web:            {}\n", r.html_url));
    out
}

pub fn issue(issue: &IssueSummary) -> String {
    let mut out = format!(
        "#{} {} [{}] by {}\n",
        issue.number, issue.title, issue.state, issue.user.login
    );
    if let Some(body) = &issue.body {
        out.push('\n');
        out.push_str(body);
        out.push('\n');
    }
    out.push_str(&format!("  \ncomments: {}\n", issue.comments));
    out
}

pub fn pull_request(pr: &PullRequest) -> String {
    let login = pr
        .user
        .as_ref()
        .map(|u| u.login.as_str())
        .unwrap_or("unknown");
    // GitHub has only open/closed states — merged PRs report state=closed with
    // merged or merged_at set, so check those first to label Merged correctly.
    let state = if pr.merged.unwrap_or(false) || pr.merged_at.is_some() {
        "Merged".to_string()
    } else {
        pr.state
            .as_ref()
            .map(|s| format!("{s:?}"))
            .unwrap_or_else(|| "-".into())
    };
    let base = &pr.base.ref_field;
    let head = &pr.head.ref_field;
    let mut out = format!(
        "#{} {} [{}] by {} ({base} ← {head})\n",
        pr.number,
        pr.title.as_deref().unwrap_or("-"),
        state,
        login,
    );
    if let Some(body) = &pr.body {
        out.push('\n');
        out.push_str(body);
        out.push('\n');
    }
    out.push_str(&format!(
        "  \ncommits: {}  changed files: {}  additions: {}  deletions: {}\n",
        pr.commits.unwrap_or(0),
        pr.changed_files.unwrap_or(0),
        pr.additions.unwrap_or(0),
        pr.deletions.unwrap_or(0),
    ));
    out
}
