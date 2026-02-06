use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoHealth {
    pub is_git_repo: bool,
    pub is_clean: bool,
    pub branch: Option<String>,
    pub last_commit_age_s: Option<i64>,
    pub ci_ok: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthScore {
    pub score: u32,
    pub api_base_url: String,
    pub api_ok: bool,
    pub api_latency_ms: Option<u128>,
    pub ledger_freshness_s: Option<i64>,
    pub spawns_freshness_s: Option<i64>,
    pub repo: Option<RepoHealth>,
    pub details: Vec<String>,
}

fn parse_rfc3339_utc(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

fn age_seconds(now: DateTime<Utc>, ts: DateTime<Utc>) -> i64 {
    now.signed_duration_since(ts).num_seconds()
}

fn get_repo_health() -> Option<RepoHealth> {
    let output = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .ok()?;

    if !output.status.success() {
        return Some(RepoHealth {
            is_git_repo: false,
            is_clean: true,
            branch: None,
            last_commit_age_s: None,
            ci_ok: None,
        });
    }

    let is_git_repo = true;

    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()?;
    let is_clean = status_output.stdout.is_empty();

    let branch_output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()?;
    let branch = String::from_utf8(branch_output.stdout)
        .ok()
        .map(|s| s.trim().to_string());

    let log_output = Command::new("git")
        .args(["log", "-1", "--format=%ct"])
        .output()
        .ok()?;
    let last_commit_ts = String::from_utf8(log_output.stdout)
        .ok()
        .and_then(|s| s.trim().parse::<i64>().ok());

    let last_commit_age_s = last_commit_ts.map(|ts| {
        let now = Utc::now().timestamp();
        now - ts
    });

    let ci_ok = None;

    Some(RepoHealth {
        is_git_repo,
        is_clean,
        branch,
        last_commit_age_s,
        ci_ok,
    })
}

pub async fn calculate_health() -> HealthScore {
    let api_base_url = crate::api::api_base_url();
    let now = Utc::now();

    let started = Instant::now();
    let health = crate::api::get_health().await;
    let api_latency_ms = Some(started.elapsed().as_millis());

    let mut details = Vec::<String>::new();
    let mut score: i32 = 100;

    let api_ok = health
        .as_ref()
        .ok()
        .and_then(|v| v["database"]["connected"].as_bool())
        .unwrap_or(false);

    if !api_ok {
        score -= 60;
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
            score -= 10;
            details.push("Ledger freshness unknown (no events or parse failed).".to_string());
        }
        Some(age) if age > 3600 => {
            score -= 10;
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
            score -= 10;
            details.push("Spawns freshness unknown (no spawns or parse failed).".to_string());
        }
        Some(age) if age > 3600 => {
            score -= 10;
            details.push(format!("Spawns stale: last activity {}s ago.", age));
        }
        Some(_) => {}
    }

    let repo_health = get_repo_health();
    if let Some(repo) = &repo_health
        && repo.is_git_repo
    {
        if !repo.is_clean {
            score -= 10;
            details.push("Repo dirty: uncommitted changes.".to_string());
        }
        if let Some(age) = repo.last_commit_age_s
            && age > 86400 * 7
        {
            score -= 5;
            details.push(format!("Repo stale: last commit {} days ago.", age / 86400));
        }
        if let Some(false) = repo.ci_ok {
            score -= 30;
            details.push("CI failed: `just ci` returned non-zero exit code.".to_string());
        }
    }

    let score = score.clamp(0, 100) as u32;

    HealthScore {
        score,
        api_base_url,
        api_ok,
        api_latency_ms,
        ledger_freshness_s,
        spawns_freshness_s,
        repo: repo_health,
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
