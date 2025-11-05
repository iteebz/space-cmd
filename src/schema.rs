/// Schema types mirroring space-os database (space/core/migrations/001_foundation.sql)
/// These represent direct reads from ~/.space/space.db
/// CRITICAL: space-cmd MUST follow space-os schema. If schema changes,
/// space-cmd fails fast on version mismatch (PRAGMA user_version).
use serde::{Deserialize, Serialize};

/// Message in a channel (messages table)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Message {
    pub message_id: String,
    pub channel_id: String,
    pub agent_id: String,
    pub content: String,
    pub created_at: String,
}

/// Channel for agent communication (channels table)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Channel {
    pub channel_id: String,
    pub name: String,
    pub topic: Option<String>,
    pub created_at: String,
    pub pinned_at: Option<String>,
}

/// Agent registered in space-os (agents table)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Agent {
    pub agent_id: String,
    pub identity: String,
    pub constitution: Option<String>,
    pub model: String,
    pub role: Option<String>,
    pub spawn_count: i32,
    pub created_at: String,
    pub last_active_at: Option<String>,
}

/// Spawn task (spawns table)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Spawn {
    pub id: String,
    pub agent_id: String,
    pub session_id: Option<String>,
    pub channel_id: Option<String>,
    pub constitution_hash: Option<String>,
    pub is_task: bool,
    pub status: String,
    pub pid: Option<i32>,
    pub created_at: String,
    pub ended_at: Option<String>,
}
