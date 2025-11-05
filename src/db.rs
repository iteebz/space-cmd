use rusqlite::{Connection, Result};
use std::env;

/// Message from a bridge channel
#[derive(Debug, Clone)]
pub struct Message {
    pub agent_id: String,
    pub content: String,
    pub created_at: String,
}

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

    const MIN_VERSION: i32 = 1; // Adjust as schema evolves
    if version < MIN_VERSION {
        return Err(format!(
            "space-os schema v{} required, found v{}. Run `space upgrade`",
            MIN_VERSION, version
        ));
    }

    Ok(())
}

/// Fetch messages for a specific channel
pub fn get_channel_messages(channel_id: &str) -> Result<Vec<Message>> {
    let db_path = get_db_path();
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare(
        "SELECT agent_id, content, created_at
         FROM messages
         WHERE channel_id = ?
         ORDER BY created_at",
    )?;

    let messages = stmt
        .query_map([channel_id], |row| {
            Ok(Message {
                agent_id: row.get(0)?,
                content: row.get(1)?,
                created_at: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(messages)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_struct() {
        let msg = Message {
            agent_id: "zealot-1".to_string(),
            content: "Test message".to_string(),
            created_at: "2025-11-05T12:34:56Z".to_string(),
        };
        assert_eq!(msg.agent_id, "zealot-1");
        assert_eq!(msg.content, "Test message");
    }
}
