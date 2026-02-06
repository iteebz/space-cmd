use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoHealth {
    pub path: String,
    pub is_git_repo: bool,
    pub is_clean: Option<bool>,
    pub branch: Option<String>,
    pub last_commit_age_s: Option<i64>,
    pub ci_ok: Option<bool>,
    pub ci_duration_ms: Option<u128>,
    pub details: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthScore {
    pub score: u32,
    pub api_score: u32,
    pub repos_score: u32,
    pub api_base_url: String,
    pub api_ok: bool,
    pub api_latency_ms: Option<u128>,
    pub ledger_freshness_s: Option<i64>,
    pub spawns_freshness_s: Option<i64>,
    pub repos: Vec<RepoHealth>,
    pub details: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RepoHealthOptions {
    pub repos: Vec<PathBuf>,
    pub repos_dir: Option<PathBuf>,
    pub run_ci: bool,
    pub timeout_s: u64,
}

impl Default for RepoHealthOptions {
    fn default() -> Self {
        Self {
            repos: Vec::new(),
            repos_dir: None,
            run_ci: false,
            timeout_s: 120,
        }
    }
}

fn parse_rfc3339_utc(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

fn age_seconds(now: DateTime<Utc>, ts: DateTime<Utc>) -> i64 {
    now.signed_duration_since(ts).num_seconds()
}

fn discover_repos(options: &RepoHealthOptions) -> Vec<PathBuf> {
    let mut repos = Vec::<PathBuf>::new();
    repos.extend(options.repos.iter().cloned());
    if let Some(dir) = &options.repos_dir
        && let Ok(entries) = std::fs::read_dir(dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join(".git").is_dir() {
                repos.push(path);
            }
        }
    }
    if repos.is_empty() {
        repos.push(std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    }
    repos.sort();
    repos.dedup();
    repos
}

async fn git_is_repo(repo: &Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(repo)
        .output()
        .await
        .ok()
        .is_some_and(|o| o.status.success())
}

async fn git_is_clean(repo: &Path) -> Option<bool> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo)
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(output.stdout.is_empty())
}

async fn git_branch(repo: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(repo)
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|s| s.trim().to_string())
}

async fn git_last_commit_age(repo: &Path) -> Option<i64> {
    let output = Command::new("git")
        .args(["log", "-1", "--format=%ct"])
        .current_dir(repo)
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let ts = String::from_utf8(output.stdout)
        .ok()
        .and_then(|s| s.trim().parse::<i64>().ok())?;
    Some(Utc::now().timestamp() - ts)
}

async fn has_just_ci(repo: &Path) -> bool {
    let justfile = repo.join("justfile");
    let Ok(contents) = tokio::fs::read_to_string(justfile).await else {
        return false;
    };
    contents
        .lines()
        .any(|line| line == "ci:" || line.starts_with("ci: "))
}

async fn run_just_ci(repo: &Path, timeout_s: u64) -> (Option<bool>, Option<u128>) {
    if !has_just_ci(repo).await {
        return (None, None);
    }
    let started = Instant::now();
    let run = async {
        Command::new("just")
            .arg("ci")
            .current_dir(repo)
            .output()
            .await
    };
    let output = tokio::time::timeout(std::time::Duration::from_secs(timeout_s), run).await;
    let duration_ms = started.elapsed().as_millis();
    match output {
        Ok(Ok(out)) => (Some(out.status.success()), Some(duration_ms)),
        _ => (Some(false), Some(duration_ms)),
    }
}

async fn repo_health_for(repo: &Path, options: &RepoHealthOptions) -> RepoHealth {
    let mut details = Vec::<String>::new();
    let is_git_repo = git_is_repo(repo).await;

    if !is_git_repo {
        return RepoHealth {
            path: repo.display().to_string(),
            is_git_repo: false,
            is_clean: None,
            branch: None,
            last_commit_age_s: None,
            ci_ok: None,
            ci_duration_ms: None,
            details: vec!["Not a git repository.".to_string()],
        };
    }

    let is_clean = git_is_clean(repo).await;
    let branch = git_branch(repo).await;
    let last_commit_age_s = git_last_commit_age(repo).await;
    let (ci_ok, ci_duration_ms) = if options.run_ci {
        run_just_ci(repo, options.timeout_s).await
    } else {
        (None, None)
    };

    if let Some(false) = is_clean {
        details.push("Working tree dirty.".to_string());
    }
    if let Some(age) = last_commit_age_s
        && age > 86400 * 7
    {
        details.push(format!("Repo stale: last commit {} days ago.", age / 86400));
    }
    if options.run_ci {
        match ci_ok {
            Some(false) => details.push("CI failed: `just ci` returned non-zero.".to_string()),
            None => details.push("CI unknown: missing `justfile` `ci:` recipe.".to_string()),
            Some(true) => {}
        }
    }

    RepoHealth {
        path: repo.display().to_string(),
        is_git_repo,
        is_clean,
        branch,
        last_commit_age_s,
        ci_ok,
        ci_duration_ms,
        details,
    }
}

fn repo_score(repo: &RepoHealth, run_ci: bool) -> (u32, Vec<String>) {
    let mut score: i32 = 100;
    let mut details = Vec::<String>::new();

    if !repo.is_git_repo {
        score -= 10;
        details.push("Not a git repo.".to_string());
        return (score.clamp(0, 100) as u32, details);
    }

    if let Some(false) = repo.is_clean {
        score -= 10;
        details.push("Dirty working tree.".to_string());
    }
    if let Some(age) = repo.last_commit_age_s
        && age > 86400 * 7
    {
        score -= 5;
        details.push("No commits in 7d.".to_string());
    }
    if run_ci {
        match repo.ci_ok {
            Some(false) => {
                score -= 30;
                details.push("`just ci` failed.".to_string());
            }
            None => {
                score -= 5;
                details.push("No `just ci` recipe.".to_string());
            }
            Some(true) => {}
        }
    }

    (score.clamp(0, 100) as u32, details)
}

pub async fn calculate_health(options: RepoHealthOptions) -> HealthScore {
    let api_base_url = crate::api::api_base_url();
    let now = Utc::now();

    let started = Instant::now();
    let health = crate::api::get_health().await;
    let api_latency_ms = Some(started.elapsed().as_millis());

    let mut details = Vec::<String>::new();
    let mut api_score: i32 = 100;

    let api_ok = health
        .as_ref()
        .ok()
        .and_then(|v| v["database"]["connected"].as_bool())
        .unwrap_or(false);

    if !api_ok {
        api_score -= 60;
        details.push("API health check failed (or DB disconnected).".to_string());
        if let Err(e) = health {
            details.push(format!("health error: {}", e));
        }
    }

    let ledger_freshness_s = crate::api::get_ledger_activity(1)
        .await
        .ok()
        .and_then(|mut items| items.pop())
        .and_then(|a| parse_rfc3339_utc(&a.created_at))
        .map(|ts| age_seconds(now, ts));

    match ledger_freshness_s {
        None => {
            api_score -= 10;
            details.push("Ledger freshness unknown (no events or parse failed).".to_string());
        }
        Some(age) if age > 3600 => {
            api_score -= 10;
            details.push(format!("Ledger stale: last event {}s ago.", age));
        }
        Some(_) => {}
    }

    let spawns_freshness_s = crate::api::get_spawns()
        .await
        .ok()
        .and_then(|spawns| {
            spawns
                .into_iter()
                .filter_map(|s| s.last_active_at.or(Some(s.created_at)))
                .filter_map(|ts| parse_rfc3339_utc(&ts))
                .max()
        })
        .map(|ts| age_seconds(now, ts));

    match spawns_freshness_s {
        None => {
            api_score -= 10;
            details.push("Spawns freshness unknown (no spawns or parse failed).".to_string());
        }
        Some(age) if age > 3600 => {
            api_score -= 10;
            details.push(format!("Spawns stale: last activity {}s ago.", age));
        }
        Some(_) => {}
    }

    let api_score = api_score.clamp(0, 100) as u32;

    let repos = discover_repos(&options);
    let mut repos_health = Vec::<RepoHealth>::new();
    let mut repos_score = 100u32;
    for repo in repos {
        let mut rh = repo_health_for(&repo, &options).await;
        let (score, score_details) = repo_score(&rh, options.run_ci);
        rh.details.extend(score_details);
        let degraded = score < 100;
        repos_score = repos_score.min(score);
        if degraded {
            details.push(format!(
                "Repo degraded: {} ({}/100)",
                rh.path.as_str(),
                score
            ));
        }
        repos_health.push(rh);
    }

    let score = api_score.min(repos_score);

    HealthScore {
        score,
        api_score,
        repos_score,
        api_base_url,
        api_ok,
        api_latency_ms,
        ledger_freshness_s,
        spawns_freshness_s,
        repos: repos_health,
        details,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn parse_rfc3339_utc_accepts_zulu() {
        let ts = parse_rfc3339_utc("2026-02-06T12:34:56Z").expect("parse");
        assert_eq!(ts.year(), 2026);
        assert_eq!(ts.month(), 2);
        assert_eq!(ts.day(), 6);
    }

    #[test]
    fn parse_rfc3339_utc_accepts_offset() {
        let ts = parse_rfc3339_utc("2026-02-06T12:34:56-05:00").expect("parse");
        assert_eq!(ts.year(), 2026);
        assert_eq!(ts.month(), 2);
        assert_eq!(ts.day(), 6);
    }
}
