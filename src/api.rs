use crate::schema::{Activity, Agent, DaemonStatus, Spawn, TailEntry};
use std::collections::HashMap;
use std::env;
use std::sync::OnceLock;
use std::time::Duration;

const DEFAULT_BASE: &str = "http://localhost:8228";
const TIMEOUT: Duration = Duration::from_millis(800);

pub fn api_base_url() -> String {
    env::var("SPACE_API_URL").unwrap_or_else(|_| DEFAULT_BASE.to_string())
}

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn client() -> &'static reqwest::Client {
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(TIMEOUT)
            .build()
            .expect("failed to create http client")
    })
}

#[derive(Debug)]
pub enum ApiError {
    Network(String),
    Decode(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Network(e) => write!(f, "network: {}", e),
            ApiError::Decode(e) => write!(f, "decode: {}", e),
        }
    }
}

impl std::error::Error for ApiError {}

type Result<T> = std::result::Result<T, ApiError>;

async fn get_json<T: serde::de::DeserializeOwned>(path: &str) -> Result<T> {
    let url = format!("{}{}", api_base_url(), path);
    let response = client()
        .get(&url)
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

    response
        .json::<T>()
        .await
        .map_err(|e| ApiError::Decode(e.to_string()))
}

pub async fn get_health() -> Result<serde_json::Value> {
    get_json("/api/health").await
}

pub async fn health_ok() -> bool {
    get_health()
        .await
        .map(|v| v["database"]["connected"].as_bool().unwrap_or(false))
        .unwrap_or(false)
}

pub async fn get_agents() -> Result<Vec<Agent>> {
    let raw: Vec<serde_json::Value> = get_json("/api/agents").await?;
    Ok(raw
        .into_iter()
        .map(|v| Agent {
            id: v["id"].as_str().unwrap_or("").to_string(),
            identity: v["identity"].as_str().unwrap_or("").to_string(),
            agent_type: v["type"].as_str().unwrap_or("ai").to_string(),
            model: v["model"].as_str().map(String::from),
            constitution: v["constitution"].as_str().map(String::from),
            avatar_path: v["avatar_path"].as_str().map(String::from),
            color: v["color"].as_str().map(String::from),
            created_at: v["created_at"].as_str().unwrap_or("").to_string(),
            archived_at: v["archived_at"].as_str().map(String::from),
        })
        .collect())
}

pub async fn get_agent_identities() -> Result<HashMap<String, String>> {
    let agents = get_agents().await?;
    Ok(agents.into_iter().map(|a| (a.id, a.identity)).collect())
}

pub async fn get_spawns() -> Result<Vec<Spawn>> {
    let raw: Vec<serde_json::Value> = get_json("/api/spawns").await?;
    Ok(raw
        .into_iter()
        .map(|v| Spawn {
            id: v["id"].as_str().unwrap_or("").to_string(),
            agent_id: v["agent_id"].as_str().unwrap_or("").to_string(),
            project_id: v["project_id"].as_str().map(String::from),
            caller_spawn_id: v["caller_spawn_id"].as_str().map(String::from),
            source: v["source"].as_str().map(String::from),
            status: v["status"].as_str().unwrap_or("done").to_string(),
            error: v["error"].as_str().map(String::from),
            pid: v["pid"].as_i64().map(|p| p as i32),
            session_id: v["session_id"].as_str().map(String::from),
            summary: v["summary"].as_str().map(String::from),
            trace_hash: v["trace_hash"].as_str().map(String::from),
            created_at: v["created_at"].as_str().unwrap_or("").to_string(),
            last_active_at: v["last_active_at"].as_str().map(String::from),
        })
        .collect())
}

pub async fn get_activity(limit: usize) -> Result<Vec<Activity>> {
    let raw: Vec<serde_json::Value> = get_json(&format!("/api/ledger?limit={}", limit)).await?;
    Ok(raw.into_iter().filter_map(ledger_to_activity).collect())
}

pub async fn get_agent_activity(agent_id: &str, limit: usize) -> Result<Vec<Activity>> {
    let all = get_activity(limit * 2).await?;
    Ok(all
        .into_iter()
        .filter(|a| a.agent_id == agent_id)
        .take(limit)
        .collect())
}

pub async fn get_ledger_activity(limit: usize) -> Result<Vec<Activity>> {
    get_activity(limit).await
}

pub async fn get_spawn_activity(spawn_id: &str, _limit: usize) -> Result<Vec<Activity>> {
    let events: serde_json::Value =
        get_json(&format!("/api/spawns/{}/events?limit=200", spawn_id)).await?;
    let items = events["events"].as_array().cloned().unwrap_or_default();
    Ok(items
        .into_iter()
        .enumerate()
        .map(|(i, v)| Activity {
            id: i as i64,
            agent_id: v["agent_id"].as_str().unwrap_or("").to_string(),
            spawn_id: Some(spawn_id.to_string()),
            primitive: v["type"].as_str().unwrap_or("event").to_string(),
            primitive_id: spawn_id.to_string(),
            action: v["type"].as_str().unwrap_or("").to_string(),
            field: v["name"].as_str().map(String::from),
            after: v["content"]
                .as_str()
                .map(String::from)
                .or_else(|| v["args"].as_str().map(String::from)),
            created_at: v["timestamp"].as_str().unwrap_or("").to_string(),
        })
        .collect())
}

fn ledger_to_activity(v: serde_json::Value) -> Option<Activity> {
    Some(Activity {
        id: 0,
        agent_id: v["agent_id"].as_str()?.to_string(),
        spawn_id: None,
        primitive: v["type"].as_str()?.to_string(),
        primitive_id: v["id"].as_str()?.to_string(),
        action: v["status"].as_str().unwrap_or("created").to_string(),
        field: None,
        after: v["content"].as_str().map(String::from),
        created_at: v["created_at"].as_str()?.to_string(),
    })
}

pub async fn get_daemon_status(_active_count: usize) -> DaemonStatus {
    get_json::<DaemonStatus>("/api/swarm/daemon")
        .await
        .unwrap_or_default()
}

pub async fn get_tail(limit: usize) -> Vec<TailEntry> {
    get_json::<Vec<TailEntry>>(&format!("/api/swarm/tail?limit={}", limit))
        .await
        .unwrap_or_default()
}

pub async fn get_agent_tail(agent: &str, limit: usize) -> Vec<TailEntry> {
    get_json::<Vec<TailEntry>>(&format!("/api/swarm/tail?limit={}&agent={}", limit, agent))
        .await
        .unwrap_or_default()
}

pub async fn get_human_agent() -> Result<Option<Agent>> {
    let agents = get_agents().await?;
    Ok(agents.into_iter().find(|a| a.agent_type == "human"))
}

pub async fn create_task(content: &str, creator_id: &str) -> Result<serde_json::Value> {
    let url = format!("{}{}", api_base_url(), "/api/tasks");
    let body = serde_json::json!({
        "content": content,
    });

    let response = client()
        .post(&url)
        .header("SPACE_IDENTITY", creator_id)
        .json(&body)
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

    response
        .json()
        .await
        .map_err(|e| ApiError::Decode(e.to_string()))
}
