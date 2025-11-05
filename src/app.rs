use crate::schema::{Channel, Message, Spawn};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SidebarTab {
    Channels,
    Spawns,
}

pub struct AppState {
    pub active_tab: SidebarTab,
    pub active_channel_idx: usize,
    pub active_spawn_idx: usize,
    pub expanded_spawns: HashSet<String>,

    pub channels: Vec<Channel>,
    pub messages: Vec<Message>,
    pub spawns: Vec<Spawn>,

    pub last_viewed_message_id: HashMap<String, String>,

    #[allow(dead_code)]
    pub message_scroll_offset: usize,
    #[allow(dead_code)]
    pub sidebar_scroll_offset: usize,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            active_tab: SidebarTab::Channels,
            active_channel_idx: 0,
            active_spawn_idx: 0,
            expanded_spawns: HashSet::new(),

            channels: vec![],
            messages: vec![],
            spawns: vec![],

            last_viewed_message_id: HashMap::new(),

            message_scroll_offset: 0,
            sidebar_scroll_offset: 0,
        }
    }

    pub fn switch_tab(&mut self) {
        self.active_tab = match self.active_tab {
            SidebarTab::Channels => SidebarTab::Spawns,
            SidebarTab::Spawns => SidebarTab::Channels,
        };
        self.sidebar_scroll_offset = 0;
    }

    pub fn next_in_sidebar(&mut self) {
        match self.active_tab {
            SidebarTab::Channels => {
                if !self.channels.is_empty() {
                    self.active_channel_idx = (self.active_channel_idx + 1) % self.channels.len();
                }
            }
            SidebarTab::Spawns => {
                if !self.spawns.is_empty() {
                    self.active_spawn_idx = (self.active_spawn_idx + 1) % self.spawns.len();
                }
            }
        }
    }

    pub fn prev_in_sidebar(&mut self) {
        match self.active_tab {
            SidebarTab::Channels => {
                if !self.channels.is_empty() {
                    self.active_channel_idx = if self.active_channel_idx == 0 {
                        self.channels.len() - 1
                    } else {
                        self.active_channel_idx - 1
                    };
                }
            }
            SidebarTab::Spawns => {
                if !self.spawns.is_empty() {
                    self.active_spawn_idx = if self.active_spawn_idx == 0 {
                        self.spawns.len() - 1
                    } else {
                        self.active_spawn_idx - 1
                    };
                }
            }
        }
    }

    pub fn toggle_spawn_expansion(&mut self) {
        if !self.spawns.is_empty() {
            let spawn_id = self.spawns[self.active_spawn_idx].id.clone();
            if self.expanded_spawns.contains(&spawn_id) {
                self.expanded_spawns.remove(&spawn_id);
            } else {
                self.expanded_spawns.insert(spawn_id);
            }
        }
    }

    pub fn current_channel(&self) -> Option<&Channel> {
        self.channels.get(self.active_channel_idx)
    }

    pub fn mark_channel_read(&mut self) {
        if let Some(channel) = self.current_channel()
            && let Some(msg) = self.messages.last()
        {
            self.last_viewed_message_id
                .insert(channel.channel_id.clone(), msg.message_id.clone());
        }
    }

    pub fn is_channel_unread(&self, channel_id: &str) -> bool {
        if let Some(last_viewed) = self.last_viewed_message_id.get(channel_id) {
            self.messages
                .iter()
                .any(|m| m.channel_id == channel_id && m.message_id > *last_viewed)
        } else {
            self.channels.iter().any(|ch| ch.channel_id == channel_id)
        }
    }

    #[allow(dead_code)]
    pub fn current_spawn(&self) -> Option<&Spawn> {
        self.spawns.get(self.active_spawn_idx)
    }

    #[allow(dead_code)]
    pub fn scroll_messages_down(&mut self) {
        self.message_scroll_offset = self.message_scroll_offset.saturating_add(1);
    }

    #[allow(dead_code)]
    pub fn scroll_messages_up(&mut self) {
        self.message_scroll_offset = self.message_scroll_offset.saturating_sub(1);
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_new() {
        let state = AppState::new();
        assert_eq!(state.active_tab, SidebarTab::Channels);
        assert_eq!(state.active_channel_idx, 0);
        assert_eq!(state.active_spawn_idx, 0);
        assert!(state.expanded_spawns.is_empty());
    }

    #[test]
    fn test_switch_tab() {
        let mut state = AppState::new();
        assert_eq!(state.active_tab, SidebarTab::Channels);
        state.switch_tab();
        assert_eq!(state.active_tab, SidebarTab::Spawns);
        state.switch_tab();
        assert_eq!(state.active_tab, SidebarTab::Channels);
    }

    #[test]
    fn test_next_in_sidebar_channels() {
        let mut state = AppState::new();
        state.channels = vec![
            Channel {
                channel_id: "ch1".to_string(),
                name: "general".to_string(),
                topic: None,
                created_at: "2025-11-05T00:00:00Z".to_string(),
                pinned_at: None,
            },
            Channel {
                channel_id: "ch2".to_string(),
                name: "tasks".to_string(),
                topic: None,
                created_at: "2025-11-05T00:00:00Z".to_string(),
                pinned_at: None,
            },
        ];
        state.active_tab = SidebarTab::Channels;

        assert_eq!(state.active_channel_idx, 0);
        state.next_in_sidebar();
        assert_eq!(state.active_channel_idx, 1);
        state.next_in_sidebar();
        assert_eq!(state.active_channel_idx, 0);
    }

    #[test]
    fn test_toggle_spawn_expansion() {
        let mut state = AppState::new();
        state.spawns = vec![Spawn {
            id: "spawn1".to_string(),
            agent_id: "hailot".to_string(),
            session_id: None,
            channel_id: None,
            constitution_hash: None,
            is_task: false,
            status: "running".to_string(),
            pid: Some(1234),
            created_at: "2025-11-05T00:00:00Z".to_string(),
            ended_at: None,
        }];

        assert!(!state.expanded_spawns.contains("spawn1"));
        state.toggle_spawn_expansion();
        assert!(state.expanded_spawns.contains("spawn1"));
        state.toggle_spawn_expansion();
        assert!(!state.expanded_spawns.contains("spawn1"));
    }

    #[test]
    fn test_scroll_messages() {
        let mut state = AppState::new();
        assert_eq!(state.message_scroll_offset, 0);
        state.scroll_messages_down();
        assert_eq!(state.message_scroll_offset, 1);
        state.scroll_messages_down();
        assert_eq!(state.message_scroll_offset, 2);
        state.scroll_messages_up();
        assert_eq!(state.message_scroll_offset, 1);
        state.scroll_messages_up();
        assert_eq!(state.message_scroll_offset, 0);
        state.scroll_messages_up();
        assert_eq!(state.message_scroll_offset, 0);
    }
}
