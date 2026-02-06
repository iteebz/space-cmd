use crate::schema::{Activity, Agent, DaemonStatus, Spawn, TailEntry};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Api,
    Db,
}

pub struct Source {
    pub mode: Mode,
}

impl Source {
    pub fn connect() -> Self {
        if crate::api::health_ok() {
            Self { mode: Mode::Api }
        } else {
            let _ = crate::db::check_schema_version();
            Self { mode: Mode::Db }
        }
    }

    pub fn get_agents(&self) -> Vec<Agent> {
        match self.mode {
            Mode::Api => crate::api::get_agents().unwrap_or_default(),
            Mode::Db => crate::db::get_agents().unwrap_or_default(),
        }
    }

    pub fn get_agent_identities(&self) -> HashMap<String, String> {
        match self.mode {
            Mode::Api => crate::api::get_agent_identities().unwrap_or_default(),
            Mode::Db => crate::db::get_agent_identities().unwrap_or_default(),
        }
    }

    pub fn get_spawns(&self) -> Vec<Spawn> {
        match self.mode {
            Mode::Api => crate::api::get_spawns().unwrap_or_default(),
            Mode::Db => crate::db::get_spawns().unwrap_or_default(),
        }
    }

    pub fn get_activity(&self, limit: usize) -> Vec<Activity> {
        match self.mode {
            Mode::Api => crate::api::get_activity(limit).unwrap_or_default(),
            Mode::Db => crate::db::get_activity(limit).unwrap_or_default(),
        }
    }

    pub fn get_agent_activity(&self, agent_id: &str, limit: usize) -> Vec<Activity> {
        match self.mode {
            Mode::Api => crate::api::get_agent_activity(agent_id, limit).unwrap_or_default(),
            Mode::Db => crate::db::get_agent_activity(agent_id, limit).unwrap_or_default(),
        }
    }

    pub fn get_ledger_activity(&self, limit: usize) -> Vec<Activity> {
        match self.mode {
            Mode::Api => crate::api::get_ledger_activity(limit).unwrap_or_default(),
            Mode::Db => crate::db::get_ledger_activity(limit).unwrap_or_default(),
        }
    }

    pub fn get_spawn_activity(&self, spawn_id: &str, limit: usize) -> Vec<Activity> {
        match self.mode {
            Mode::Api => crate::api::get_spawn_activity(spawn_id, limit).unwrap_or_default(),
            Mode::Db => crate::db::get_spawn_activity(spawn_id, limit).unwrap_or_default(),
        }
    }

    pub fn get_daemon_status(&self, active_count: usize) -> DaemonStatus {
        crate::db::get_daemon_status(active_count)
    }

    pub fn get_tail(&self, limit: usize) -> Vec<TailEntry> {
        crate::db::get_tail(limit)
    }

    pub fn get_agent_tail(&self, agent: &str, limit: usize) -> Vec<TailEntry> {
        crate::db::get_agent_tail(agent, limit)
    }
}
