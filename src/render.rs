//! Shared plain-text renderers for view/diff commands (repo, pr, issue).
//! One fetch → render shape keeps output consistent across command groups.

use crate::error::AppError;
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

pub fn status_report(report: &crate::github::status::StatusReport) -> String {
    let mut out = String::new();

    out.push_str("Assigned Issues\n");
    if report.assigned_issues.is_empty() {
        out.push_str("  Nothing here ^_^\n");
    } else {
        for item in &report.assigned_issues {
            out.push_str(&format!("  {}#{} {}\n", item.repo, item.number, item.title));
        }
    }

    out.push_str("\nReview Requests\n");
    if report.review_requests.is_empty() {
        out.push_str("  Nothing here ^_^\n");
    } else {
        for item in &report.review_requests {
            out.push_str(&format!("  {}#{} {}\n", item.repo, item.number, item.title));
        }
    }

    out.push_str("\nCreated Pull Requests\n");
    if report.my_prs.is_empty() {
        out.push_str("  Nothing here ^_^\n");
    } else {
        for item in &report.my_prs {
            out.push_str(&format!("  {}#{} {}\n", item.repo, item.number, item.title));
        }
    }

    out
}

pub fn release(r: &crate::github::release::ReleaseSummary) -> String {
    let mut out = format!(
        "{} ({})\n",
        r.name.as_deref().unwrap_or(&r.tag_name),
        r.tag_name
    );
    let kind = if r.draft {
        "Draft"
    } else if r.prerelease {
        "Pre-release"
    } else {
        "Latest"
    };
    out.push_str(&format!("  type: {}\n", kind));
    out.push_str(&format!(
        "  published: {}\n",
        r.published_at.as_deref().unwrap_or("-")
    ));
    out.push_str(&format!("  url: {}\n", r.html_url));
    if let Some(body) = &r.body {
        out.push_str(&format!("\n  {}\n", body.replace('\n', "\n  ")));
    }
    if !r.assets.is_empty() {
        out.push_str("\n  Assets:\n");
        for a in &r.assets {
            let mb = a.size as f64 / 1_048_576.0;
            out.push_str(&format!(
                "    {} ({:.1} MB, {} downloads)\n",
                a.name, mb, a.download_count
            ));
        }
    }
    out
}

pub fn gist(g: &crate::github::gist::GistSummary) -> String {
    let mut out = format!(
        "{} ({})\n",
        g.description.as_deref().unwrap_or("No description"),
        g.id
    );
    let vis = if g.public { "public" } else { "secret" };
    out.push_str(&format!("  visibility: {}\n", vis));
    out.push_str(&format!("  updated:    {}\n", g.updated_at));
    out.push_str(&format!("  url:        {}\n", g.html_url));
    out.push_str("\n  Files:\n");
    for (name, f) in &g.files {
        out.push_str(&format!(
            "    {} ({} bytes, {})\n",
            name,
            f.size.unwrap_or(0),
            f.language.as_deref().unwrap_or("-")
        ));
    }
    out
}

/// Evaluates a jq filter string on any serializable data using the embedded jaq engine.
pub fn evaluate_jq<T: serde::Serialize>(data: &T, filter_str: &str) -> Result<String, AppError> {
    use jaq_core::load::{Arena, File, Loader};
    use jaq_core::Compiler;
    use jaq_json::Val;

    let json_bytes = serde_json::to_vec(data)
        .map_err(|e| AppError::InvalidInput(format!("failed to serialize data: {e}")))?;
    let input_val = jaq_json::read::parse_single(&json_bytes)
        .map_err(|e| AppError::InvalidInput(format!("failed to parse JSON input: {e:?}")))?;

    let program = File {
        code: filter_str,
        path: (),
    };
    let defs = jaq_core::defs()
        .chain(jaq_std::defs())
        .chain(jaq_json::defs());
    let funs = jaq_core::funs()
        .chain(jaq_std::funs())
        .chain(jaq_json::funs());

    let loader = Loader::new(defs);
    let arena = Arena::default();
    let modules = loader
        .load(&arena, program)
        .map_err(|e| AppError::InvalidInput(format!("jq parse error: {e:?}")))?;
    let filter = Compiler::default()
        .with_funs(funs)
        .compile(modules)
        .map_err(|e| AppError::InvalidInput(format!("jq compile error: {e:?}")))?;

    let ctx =
        jaq_core::Ctx::<jaq_core::data::JustLut<Val>>::new(&filter.lut, jaq_core::Vars::new([]));
    let mut outputs = Vec::new();

    for item in filter.id.run((ctx, input_val)) {
        let val = item.map_err(|e| AppError::InvalidInput(format!("jq runtime error: {e:?}")))?;
        let s = format!("{val}");
        if let Ok(serde_json::Value::String(unquoted)) =
            serde_json::from_str::<serde_json::Value>(&s)
        {
            outputs.push(unquoted);
        } else {
            outputs.push(s);
        }
    }

    Ok(outputs.join("\n"))
}

/// Unified output formatter for all CLI commands.
/// Precedence:
/// 1. `jq`: If specified, filters the serialized JSON using embedded jaq.
/// 2. `json`: If true, pretty-prints JSON.
/// 3. `plain`: If true and `plain_fn` is provided, renders plain tab-separated text.
/// 4. Default: Runs `human_fn` for rich terminal output.
pub fn format_output<T: serde::Serialize>(
    data: &T,
    json: bool,
    jq: Option<&str>,
    plain: bool,
    human_fn: impl FnOnce(&T) -> String,
    plain_fn: Option<impl FnOnce(&T) -> String>,
) -> Result<String, AppError> {
    if let Some(filter) = jq {
        evaluate_jq(data, filter)
    } else if json {
        serde_json::to_string_pretty(data)
            .map_err(|e| AppError::InvalidInput(format!("failed to serialize JSON: {e}")))
    } else if plain {
        if let Some(p) = plain_fn {
            Ok(p(data))
        } else {
            Ok(human_fn(data))
        }
    } else {
        Ok(human_fn(data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jaq_evaluation() {
        let data = serde_json::json!([
            {"number": 1, "title": "First issue"},
            {"number": 2, "title": "Second issue"}
        ]);
        let res = evaluate_jq(&data, ".[].title").unwrap();
        assert_eq!(res, "First issue\nSecond issue");

        let numbers = evaluate_jq(&data, ".[].number").unwrap();
        assert_eq!(numbers, "1\n2");

        let bad_filter = evaluate_jq(&data, ".[invalid syntax");
        assert!(bad_filter.is_err());
    }

    #[test]
    fn test_format_output() {
        let data = serde_json::json!({"id": 123, "name": "gh-rs"});

        // 1. jq filter wins
        let jq_out = format_output(
            &data,
            false,
            Some(".name"),
            false,
            |_| "human".into(),
            None::<fn(&serde_json::Value) -> String>,
        )
        .unwrap();
        assert_eq!(jq_out, "gh-rs");

        // 2. json mode
        let json_out = format_output(
            &data,
            true,
            None,
            false,
            |_| "human".into(),
            None::<fn(&serde_json::Value) -> String>,
        )
        .unwrap();
        assert!(json_out.contains("\"id\": 123"));

        // 3. plain mode
        let plain_out = format_output(
            &data,
            false,
            None,
            true,
            |_| "human".into(),
            Some(|d: &serde_json::Value| format!("{}\t{}", d["id"], d["name"])),
        )
        .unwrap();
        assert_eq!(plain_out, "123\t\"gh-rs\"");

        // 4. human mode default
        let human_out = format_output(
            &data,
            false,
            None,
            false,
            |_| "human output".into(),
            None::<fn(&serde_json::Value) -> String>,
        )
        .unwrap();
        assert_eq!(human_out, "human output");
    }
}
