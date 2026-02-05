use crate::schema::{Activity, Agent, Spawn};
use rusqlite::{Connection, Result, params};
use std::collections::HashMap;
use std::env;

const MIN_SCHEMA_VERSION: i32 = 1;

fn get_db_path() -> String {
    env::var("SPACE_DB").unwrap_or_else(|_| {
        let home = env::var("HOME").expect("HOME not set");
        format!("{}/.space/space.db", home)
    })
}

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

pub fn get_agents() -> Result<Vec<Agent>> {
    let conn = Connection::open(get_db_path())?;

    let mut stmt = conn.prepare(
        "SELECT id, identity, type, model, created_at, archived_at
         FROM agents
         WHERE archived_at IS NULL AND deleted_at IS NULL
         ORDER BY identity",
    )?;

    stmt.query_map([], |row| {
        Ok(Agent {
            id: row.get(0)?,
            identity: row.get(1)?,
            agent_type: row.get(2)?,
            model: row.get(3)?,
            created_at: row.get(4)?,
            archived_at: row.get(5)?,
        })
    })?
    .collect()
}

pub fn get_agent_identities() -> Result<HashMap<String, String>> {
    let conn = Connection::open(get_db_path())?;

    let mut stmt = conn.prepare("SELECT id, identity FROM agents")?;

    let pairs: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<Vec<_>>>()?;

    Ok(pairs.into_iter().collect())
}

pub fn get_spawns() -> Result<Vec<Spawn>> {
    let conn = Connection::open(get_db_path())?;

    let mut stmt = conn.prepare(
        "SELECT id, agent_id, project_id, caller_spawn_id, source, status, error, pid, session_id, summary, created_at, last_active_at, resume_count
         FROM spawns
         ORDER BY created_at DESC
         LIMIT 100",
    )?;

    stmt.query_map([], |row| {
        Ok(Spawn {
            id: row.get(0)?,
            agent_id: row.get(1)?,
            project_id: row.get(2)?,
            caller_spawn_id: row.get(3)?,
            source: row.get(4)?,
            status: row.get(5)?,
            error: row.get(6)?,
            pid: row.get(7)?,
            session_id: row.get(8)?,
            summary: row.get(9)?,
            created_at: row.get(10)?,
            last_active_at: row.get(11)?,
            resume_count: row.get::<_, Option<i32>>(12)?.unwrap_or(0),
        })
    })?
    .collect()
}

pub fn get_activity(limit: usize) -> Result<Vec<Activity>> {
    let conn = Connection::open(get_db_path())?;

    let mut stmt = conn.prepare(
        "SELECT id, agent_id, spawn_id, primitive, primitive_id, action, field, after, created_at
         FROM activity
         ORDER BY created_at DESC
         LIMIT ?",
    )?;

    stmt.query_map(params![limit as i32], |row| {
        Ok(Activity {
            id: row.get(0)?,
            agent_id: row.get(1)?,
            spawn_id: row.get(2)?,
            primitive: row.get(3)?,
            primitive_id: row.get(4)?,
            action: row.get(5)?,
            field: row.get(6)?,
            after: row.get(7)?,
            created_at: row.get(8)?,
        })
    })?
    .collect()
}

pub fn get_agent_activity(agent_id: &str, limit: usize) -> Result<Vec<Activity>> {
    let conn = Connection::open(get_db_path())?;

    let mut stmt = conn.prepare(
        "SELECT id, agent_id, spawn_id, primitive, primitive_id, action, field, after, created_at
         FROM activity
         WHERE agent_id = ?
         ORDER BY created_at DESC
         LIMIT ?",
    )?;

    stmt.query_map(params![agent_id, limit as i32], |row| {
        Ok(Activity {
            id: row.get(0)?,
            agent_id: row.get(1)?,
            spawn_id: row.get(2)?,
            primitive: row.get(3)?,
            primitive_id: row.get(4)?,
            action: row.get(5)?,
            field: row.get(6)?,
            after: row.get(7)?,
            created_at: row.get(8)?,
        })
    })?
    .collect()
}

pub fn get_spawn_activity(spawn_id: &str, limit: usize) -> Result<Vec<Activity>> {
    let conn = Connection::open(get_db_path())?;

    let mut stmt = conn.prepare(
        "SELECT id, agent_id, spawn_id, primitive, primitive_id, action, field, after, created_at
         FROM activity
         WHERE spawn_id = ?
         ORDER BY created_at DESC
         LIMIT ?",
    )?;

    stmt.query_map(params![spawn_id, limit as i32], |row| {
        Ok(Activity {
            id: row.get(0)?,
            agent_id: row.get(1)?,
            spawn_id: row.get(2)?,
            primitive: row.get(3)?,
            primitive_id: row.get(4)?,
            action: row.get(5)?,
            field: row.get(6)?,
            after: row.get(7)?,
            created_at: row.get(8)?,
        })
    })?
    .collect()
}

#[cfg(test)]
mod tests {
    use crate::schema::{Agent, Spawn};

    #[test]
    fn test_agent_struct() {
        let ag = Agent {
            id: "a1".to_string(),
            identity: "zealot".to_string(),
            agent_type: "ai".to_string(),
            model: Some("claude-3-5-sonnet".to_string()),
            created_at: "2025-11-05T12:34:56Z".to_string(),
            archived_at: None,
        };
        assert_eq!(ag.identity, "zealot");
    }

    #[test]
    fn test_spawn_struct() {
        let sp = Spawn {
            id: "s1".to_string(),
            agent_id: "a1".to_string(),
            project_id: None,
            caller_spawn_id: None,
            source: Some("manual".to_string()),
            status: "active".to_string(),
            error: None,
            pid: Some(1234),
            session_id: None,
            summary: None,
            created_at: "2025-11-05T12:34:56Z".to_string(),
            last_active_at: None,
            resume_count: 0,
        };
        assert_eq!(sp.status, "active");
    }
}
