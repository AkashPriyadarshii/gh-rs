//! GitHub arbitrary API request executor (`gh-rs api`).
//!
//! Provides direct access to GitHub REST and GraphQL APIs with placeholder resolution,
//! automatic pagination, and embedded jaq/jq filtering.

use crate::error::AppError;
use crate::github::client;
use serde_json::{Map, Value};

#[derive(Debug, Default)]
pub struct ApiRequest<'a> {
    pub endpoint: &'a str,
    pub method: Option<&'a str>,
    pub raw_fields: &'a [String],
    pub fields: &'a [String],
    pub headers: &'a [String],
    pub input_file: Option<&'a str>,
    pub paginate: bool,
    pub slurp: bool,
}

/// Executes an authenticated GitHub API request.
pub async fn execute(req: ApiRequest<'_>) -> Result<Value, AppError> {
    let token = client::load_token()?;
    let http = client::http();

    // Determine target URL with placeholder resolution if possible
    let (owner, repo) = crate::git::current_repo().unwrap_or_else(|_| ("".into(), "".into()));
    let branch = crate::git::current_branch().unwrap_or_default();
    let mut current_url = normalize_endpoint(req.endpoint, &owner, &repo, &branch)?;

    let has_body_data =
        !req.raw_fields.is_empty() || !req.fields.is_empty() || req.input_file.is_some();
    let method_str = req
        .method
        .unwrap_or(if has_body_data { "POST" } else { "GET" });
    let req_method = method_str
        .parse::<reqwest::Method>()
        .map_err(|e| AppError::InvalidInput(format!("invalid HTTP method {method_str}: {e}")))?;

    let mut body_map = Map::new();
    let mut query_params: Vec<(String, String)> = Vec::new();

    for f in req.raw_fields {
        let (k, v) = parse_field(f, false)?;
        if req_method == reqwest::Method::GET {
            if let Value::String(s) = v {
                query_params.push((k, s));
            }
        } else {
            body_map.insert(k, v);
        }
    }

    for f in req.fields {
        let (k, v) = parse_field(f, true)?;
        if req_method == reqwest::Method::GET {
            let s = match v {
                Value::String(s) => s,
                other => other.to_string(),
            };
            query_params.push((k, s));
        } else {
            body_map.insert(k, v);
        }
    }

    let custom_body = if let Some(path) = req.input_file {
        if path == "-" {
            use std::io::Read;
            let mut buf = Vec::new();
            std::io::stdin()
                .read_to_end(&mut buf)
                .map_err(|e| AppError::InvalidInput(format!("failed to read stdin: {e}")))?;
            Some(buf)
        } else {
            let bytes = std::fs::read(path).map_err(|e| {
                AppError::InvalidInput(format!("failed to read input file {path}: {e}"))
            })?;
            Some(bytes)
        }
    } else {
        None
    };

    let mut collected_results: Vec<Value> = Vec::new();

    loop {
        let mut target_url = current_url.clone();
        if req_method == reqwest::Method::GET && !query_params.is_empty() {
            let delimiter = if target_url.contains('?') { '&' } else { '?' };
            let qs = query_params
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("&");
            target_url.push(delimiter);
            target_url.push_str(&qs);
        }

        let mut http_req = http
            .request(req_method.clone(), &target_url)
            .header("Authorization", format!("Bearer {token}"))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28");

        for h in req.headers {
            if let Some((k, v)) = h.split_once(':') {
                http_req = http_req.header(k.trim(), v.trim());
            }
        }

        if req_method != reqwest::Method::GET {
            if let Some(bytes) = &custom_body {
                http_req = http_req.body(bytes.clone());
            } else if !body_map.is_empty() {
                http_req = http_req.json(&body_map);
            }
        }

        let resp = http_req.send().await?;
        let status = resp.status();
        let link_header = resp
            .headers()
            .get("link")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let text = resp.text().await?;
        if !status.is_success() {
            return Err(AppError::GitHubApi(format!("HTTP {status}: {text}")));
        }

        let val: Value = if text.trim().is_empty() {
            Value::Null
        } else {
            serde_json::from_str(&text).map_err(|e| {
                AppError::GitHubApi(format!("failed to parse GitHub API JSON response: {e}"))
            })?
        };

        if !req.paginate {
            return Ok(val);
        }

        collected_results.push(val);

        if let Some(links) = link_header {
            if let Some(next) = extract_next_page(&links) {
                current_url = next;
                continue;
            }
        }
        break;
    }

    if req.slurp {
        Ok(Value::Array(collected_results))
    } else {
        // Concatenate arrays if all pages are arrays
        let mut combined = Vec::new();
        let mut all_arrays = true;
        for page in &collected_results {
            if let Value::Array(items) = page {
                combined.extend(items.clone());
            } else {
                all_arrays = false;
                break;
            }
        }
        if all_arrays {
            Ok(Value::Array(combined))
        } else {
            Ok(Value::Array(collected_results))
        }
    }
}

/// Normalizes an API endpoint or URL, resolving `{owner}`, `{repo}`, and `{branch}` placeholders.
pub fn normalize_endpoint(
    endpoint: &str,
    owner: &str,
    repo: &str,
    branch: &str,
) -> Result<String, AppError> {
    #[cfg(windows)]
    {
        // Guard against MSYS/Git Bash converting "/repos/..." to "C:/Program Files/Git/repos/..."
        if std::path::Path::new(endpoint).is_absolute() && !endpoint.contains("://") {
            return Err(AppError::InvalidInput(
                "invalid API endpoint: path was converted to absolute by shell. Omit leading slash."
                    .into(),
            ));
        }
    }

    let replaced = endpoint
        .replace("{owner}", owner)
        .replace(":owner", owner)
        .replace("{repo}", repo)
        .replace(":repo", repo)
        .replace("{branch}", branch)
        .replace(":branch", branch);

    if replaced.contains("://") {
        Ok(replaced)
    } else if replaced == "graphql" {
        Ok("https://api.github.com/graphql".into())
    } else {
        let trimmed = replaced.trim_start_matches('/');
        Ok(format!("https://api.github.com/{trimmed}"))
    }
}

/// Parses RFC 5988 Link response header to extract next page URL.
pub fn extract_next_page(link_header: &str) -> Option<String> {
    for part in link_header.split(',') {
        let sections: Vec<&str> = part.split(';').map(|s| s.trim()).collect();
        if sections.len() >= 2 && sections[1].contains("rel=\"next\"") {
            let url = sections[0].trim_start_matches('<').trim_end_matches('>');
            return Some(url.to_string());
        }
    }
    None
}

/// Parses a key=value parameter into a JSON key and Value.
pub fn parse_field(field_str: &str, typed: bool) -> Result<(String, Value), AppError> {
    let (key, val) = field_str.split_once('=').ok_or_else(|| {
        AppError::InvalidInput(format!(
            "field must be in key=value format, got: {field_str}"
        ))
    })?;

    if !typed {
        return Ok((key.to_string(), Value::String(val.to_string())));
    }

    let json_val = match val {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        "null" => Value::Null,
        other => {
            if let Ok(num) = other.parse::<i64>() {
                Value::Number(num.into())
            } else if let Ok(parsed_json) = serde_json::from_str::<Value>(other) {
                parsed_json
            } else {
                Value::String(other.to_string())
            }
        }
    };

    Ok((key.to_string(), json_val))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_endpoint_placeholders() {
        let endpoint = "repos/{owner}/{repo}/releases/tags/:branch";
        let normalized = normalize_endpoint(endpoint, "octocat", "hello-world", "v1.0.0").unwrap();
        assert_eq!(
            normalized,
            "https://api.github.com/repos/octocat/hello-world/releases/tags/v1.0.0"
        );
    }

    #[test]
    fn test_normalize_endpoint_graphql() {
        assert_eq!(
            normalize_endpoint("graphql", "a", "b", "c").unwrap(),
            "https://api.github.com/graphql"
        );
    }

    #[test]
    fn test_normalize_endpoint_absolute_url() {
        assert_eq!(
            normalize_endpoint("https://custom.api.github.com/user", "a", "b", "c").unwrap(),
            "https://custom.api.github.com/user"
        );
    }

    #[test]
    fn test_extract_next_page() {
        let header = r#"<https://api.github.com/user/repos?page=2>; rel="next", <https://api.github.com/user/repos?page=5>; rel="last""#;
        assert_eq!(
            extract_next_page(header),
            Some("https://api.github.com/user/repos?page=2".into())
        );

        let last_header = r#"<https://api.github.com/user/repos?page=1>; rel="prev""#;
        assert_eq!(extract_next_page(last_header), None);
    }

    #[test]
    fn test_parse_field() {
        // String/raw field
        let (k, v) = parse_field("title=My PR", false).unwrap();
        assert_eq!(k, "title");
        assert_eq!(v, Value::String("My PR".into()));

        // Typed boolean
        let (k, v) = parse_field("draft=true", true).unwrap();
        assert_eq!(k, "draft");
        assert_eq!(v, Value::Bool(true));

        // Typed integer
        let (k, v) = parse_field("limit=50", true).unwrap();
        assert_eq!(k, "limit");
        assert_eq!(v, Value::Number(50.into()));

        // Typed null
        let (k, v) = parse_field("assignee=null", true).unwrap();
        assert_eq!(k, "assignee");
        assert_eq!(v, Value::Null);
    }
}
