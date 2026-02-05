use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default)]
pub struct DaemonStatus {
    pub running: bool,
    pub pid: Option<i32>,
    pub enabled: bool,
    pub concurrency: i32,
    pub active_count: usize,
    pub last_skip: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Agent {
    pub id: String,
    pub identity: String,
    pub agent_type: String,
    pub model: Option<String>,
    pub created_at: String,
    pub archived_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Spawn {
    pub id: String,
    pub agent_id: String,
    pub project_id: Option<String>,
    pub caller_spawn_id: Option<String>,
    pub source: Option<String>,
    pub status: String,
    pub error: Option<String>,
    pub pid: Option<i32>,
    pub session_id: Option<String>,
    pub summary: Option<String>,
    pub created_at: String,
    pub last_active_at: Option<String>,
    pub resume_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Activity {
    pub id: i64,
    pub agent_id: String,
    pub spawn_id: Option<String>,
    pub primitive: String,
    pub primitive_id: String,
    pub action: String,
    pub field: Option<String>,
    pub after: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TailEntry {
    pub spawn: String,
    pub agent: String,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub content: Option<String>,
    pub name: Option<String>,
    pub args: Option<String>,
    pub ctx_pct: Option<u32>,
}
