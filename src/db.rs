use crate::schema::{Agent, Channel, Message, Spawn, Transcript};
use rusqlite::{Connection, Result, params};
use std::env;

const MIN_SCHEMA_VERSION: i32 = 1;

/// Get the space-os database path
/// Priority: $SPACE_DB env var, then ~/.space/space.db
fn get_db_path() -> String {
    env::var("SPACE_DB").unwrap_or_else(|_| {
        let home = env::var("HOME").expect("HOME not set");
        format!("{}/.space/space.db", home)
    })
}

/// Check schema compatibility on startup
/// Fails fast if space-os schema is too old
pub fn check_schema_version() -> Result<(), String> {
    let db_path = get_db_path();
    let conn = Connection::open(&db_path).map_err(|e| format!("Failed to open space.db: {}", e))?;

    let version: i32 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(|e| format!("Failed to read schema version: {}", e))?;

    if version < MIN_SCHEMA_VERSION {
        return Err(format!(
            "space-os schema v{} required, found v{}. Run `space upgrade`",
            MIN_SCHEMA_VERSION, version
        ));
    }

    Ok(())
}

/// Fetch all non-archived channels
#[allow(dead_code)]
pub fn get_channels() -> Result<Vec<Channel>> {
    let db_path = get_db_path();
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare(
        "SELECT channel_id, name, topic, created_at, pinned_at
         FROM channels
         WHERE archived_at IS NULL
         ORDER BY created_at DESC",
    )?;

    let channels = stmt
        .query_map([], |row| {
            Ok(Channel {
                channel_id: row.get(0)?,
                name: row.get(1)?,
                topic: row.get(2)?,
                created_at: row.get(3)?,
                pinned_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(channels)
}

/// Fetch messages for a specific channel
pub fn get_channel_messages(channel_id: &str) -> Result<Vec<Message>> {
    let db_path = get_db_path();
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare(
        "SELECT message_id, channel_id, agent_id, content, created_at
         FROM messages
         WHERE channel_id = ?
         ORDER BY created_at DESC
         LIMIT 500",
    )?;

    let messages = stmt
        .query_map(params![channel_id], |row| {
            Ok(Message {
                message_id: row.get(0)?,
                channel_id: row.get(1)?,
                agent_id: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(messages)
}

/// Fetch all non-archived agents
#[allow(dead_code)]
pub fn get_agents() -> Result<Vec<Agent>> {
    let db_path = get_db_path();
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare(
        "SELECT agent_id, identity, constitution, model, role, spawn_count, created_at, last_active_at
         FROM agents
         WHERE archived_at IS NULL
         ORDER BY last_active_at DESC NULLS LAST",
    )?;

    let agents = stmt
        .query_map([], |row| {
            Ok(Agent {
                agent_id: row.get(0)?,
                identity: row.get(1)?,
                constitution: row.get(2)?,
                model: row.get(3)?,
                role: row.get(4)?,
                spawn_count: row.get(5)?,
                created_at: row.get(6)?,
                last_active_at: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(agents)
}

pub fn get_spawns() -> Result<Vec<Spawn>> {
    let db_path = get_db_path();
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare(
        "SELECT id, agent_id, session_id, channel_id, constitution_hash, is_task, status, pid, created_at, ended_at
         FROM spawns
         ORDER BY created_at DESC
         LIMIT 100",
    )?;

    let spawns = stmt
        .query_map([], |row| {
            Ok(Spawn {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                session_id: row.get(2)?,
                channel_id: row.get(3)?,
                constitution_hash: row.get(4)?,
                is_task: row.get::<_, i32>(5)? != 0,
                status: row.get(6)?,
                pid: row.get(7)?,
                created_at: row.get(8)?,
                ended_at: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(spawns)
}

pub fn get_transcripts(session_id: &str, limit: usize) -> Result<Vec<Transcript>> {
    let db_path = get_db_path();
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare(
        "SELECT session_id, message_index, role, content, timestamp
         FROM transcripts
         WHERE session_id = ?
         ORDER BY message_index DESC
         LIMIT ?",
    )?;

    let transcripts = stmt
        .query_map(params![session_id, limit as i32], |row| {
            Ok(Transcript {
                session_id: row.get(0)?,
                message_index: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                timestamp: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(transcripts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_struct() {
        let ch = Channel {
            channel_id: "ch1".to_string(),
            name: "general".to_string(),
            topic: Some("dev".to_string()),
            created_at: "2025-11-05T12:34:56Z".to_string(),
            pinned_at: None,
        };
        assert_eq!(ch.name, "general");
    }

    #[test]
    fn test_message_struct() {
        let msg = Message {
            message_id: "m1".to_string(),
            channel_id: "ch1".to_string(),
            agent_id: "hailot".to_string(),
            content: "test".to_string(),
            created_at: "2025-11-05T12:34:56Z".to_string(),
        };
        assert_eq!(msg.agent_id, "hailot");
    }

    #[test]
    fn test_agent_struct() {
        let ag = Agent {
            agent_id: "a1".to_string(),
            identity: "zealot".to_string(),
            constitution: None,
            model: "claude-3-5-sonnet".to_string(),
            role: Some("executor".to_string()),
            spawn_count: 5,
            created_at: "2025-11-05T12:34:56Z".to_string(),
            last_active_at: None,
        };
        assert_eq!(ag.spawn_count, 5);
    }
}
