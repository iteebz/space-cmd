use crate::schema::{Activity, Agent, DaemonStatus, Spawn, TailEntry};
use std::collections::HashMap;

pub struct Source;

impl Source {
    pub fn connect() -> Self {
        Self
    }

    pub async fn get_agents(&self) -> Vec<Agent> {
        crate::api::get_agents().await.unwrap_or_default()
    }

    pub async fn get_agent_identities(&self) -> HashMap<String, String> {
        crate::api::get_agent_identities().await.unwrap_or_default()
    }

    pub async fn get_spawns(&self) -> Vec<Spawn> {
        crate::api::get_spawns().await.unwrap_or_default()
    }

    pub async fn get_activity(&self, limit: usize) -> Vec<Activity> {
        crate::api::get_activity(limit).await.unwrap_or_default()
    }

    pub async fn get_agent_activity(&self, agent_id: &str, limit: usize) -> Vec<Activity> {
        crate::api::get_agent_activity(agent_id, limit)
            .await
            .unwrap_or_default()
    }

    pub async fn get_ledger_activity(&self, limit: usize) -> Vec<Activity> {
        crate::api::get_ledger_activity(limit)
            .await
            .unwrap_or_default()
    }

    pub async fn get_spawn_activity(&self, spawn_id: &str, limit: usize) -> Vec<Activity> {
        crate::api::get_spawn_activity(spawn_id, limit)
            .await
            .unwrap_or_default()
    }

    pub async fn get_daemon_status(&self, active_count: usize) -> DaemonStatus {
        crate::api::get_daemon_status(active_count).await
    }

    pub async fn get_tail(&self, limit: usize) -> Vec<TailEntry> {
        crate::api::get_tail(limit).await
    }

    pub async fn get_agent_tail(&self, agent: &str, limit: usize) -> Vec<TailEntry> {
        crate::api::get_agent_tail(agent, limit).await
    }
}
