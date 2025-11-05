use crate::schema::{Channel, Message, Spawn};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SidebarTab {
    Channels,
    Spawns,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AutocompleteMode {
    Agent,
    File,
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

    pub input_text: String,
    pub input_history: Vec<String>,
    pub history_idx: Option<usize>,
    pub input_scroll_offset: usize,

    pub autocomplete_mode: Option<AutocompleteMode>,
    pub autocomplete_list: Vec<String>,
    pub autocomplete_idx: usize,
    pub autocomplete_query: String,
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

            input_text: String::new(),
            input_history: Vec::new(),
            history_idx: None,
            input_scroll_offset: 0,

            autocomplete_mode: None,
            autocomplete_list: Vec::new(),
            autocomplete_idx: 0,
            autocomplete_query: String::new(),
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

    pub fn scroll_messages_down(&mut self) {
        let max_scroll = self.messages.len().saturating_sub(1);
        self.message_scroll_offset = (self.message_scroll_offset + 1).min(max_scroll);
    }

    pub fn scroll_messages_up(&mut self) {
        self.message_scroll_offset = self.message_scroll_offset.saturating_sub(1);
    }

    #[allow(dead_code)]
    pub fn reset_message_scroll(&mut self) {
        self.message_scroll_offset = 0;
    }

    pub fn add_char(&mut self, ch: char) {
        self.input_text.push(ch);
        self.history_idx = None;
    }

    pub fn backspace(&mut self) {
        self.input_text.pop();
        self.history_idx = None;
    }

    pub fn history_prev(&mut self) {
        if self.input_history.is_empty() {
            return;
        }

        match self.history_idx {
            None => {
                self.history_idx = Some(0);
                self.input_text = self.input_history[0].clone();
            }
            Some(idx) if idx + 1 < self.input_history.len() => {
                self.history_idx = Some(idx + 1);
                self.input_text = self.input_history[idx + 1].clone();
            }
            _ => {}
        }
    }

    pub fn history_next(&mut self) {
        match self.history_idx {
            Some(idx) if idx > 0 => {
                self.history_idx = Some(idx - 1);
                self.input_text = self.input_history[idx - 1].clone();
            }
            Some(0) => {
                self.history_idx = None;
                self.input_text.clear();
            }
            _ => {}
        }
    }

    pub fn submit_input(&mut self) -> Option<String> {
        if self.input_text.is_empty() {
            return None;
        }

        let cmd = self.input_text.clone();
        self.input_history.insert(0, cmd.clone());
        self.input_text.clear();
        self.history_idx = None;
        self.input_scroll_offset = 0;

        Some(cmd)
    }

    #[allow(dead_code)]
    pub fn input_line_count(&self) -> usize {
        self.input_text.lines().count().max(1)
    }

    pub fn detect_and_trigger_autocomplete(&mut self) {
        if self.input_text.is_empty() {
            return;
        }

        let last_word_start = self
            .input_text
            .rfind(|c: char| c.is_whitespace())
            .map(|i| i + 1)
            .unwrap_or(0);

        let last_word = &self.input_text[last_word_start..];

        if let Some(query) = last_word.strip_prefix('@') {
            self.autocomplete_mode = Some(AutocompleteMode::Agent);
            self.autocomplete_query = query.to_string();
            self.load_agent_autocomplete();
        } else if let Some(query) = last_word.strip_prefix('/')
            && !query.is_empty()
        {
            self.autocomplete_mode = Some(AutocompleteMode::File);
            self.autocomplete_query = query.to_string();
            self.load_file_autocomplete();
        }
    }

    pub fn load_agent_autocomplete(&mut self) {
        if let Ok(agents) = crate::db::get_agents() {
            self.autocomplete_list = agents.iter().map(|a| a.agent_id.clone()).collect();
            self.filter_autocomplete();
        }
    }

    pub fn load_file_autocomplete(&mut self) {
        use std::fs;

        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let space_dir = format!("{}/.space", home);

        if let Ok(entries) = fs::read_dir(&space_dir) {
            let mut files = Vec::new();
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata()
                    && let Some(name) = entry.path().file_name().and_then(|n| n.to_str())
                    && metadata.is_file()
                {
                    files.push(name.to_string());
                }
            }
            self.autocomplete_list = files;
            self.filter_autocomplete();
        }
    }

    pub fn filter_autocomplete(&mut self) {
        let query = self.autocomplete_query.to_lowercase();
        self.autocomplete_list
            .retain(|item| item.to_lowercase().contains(&query));
        self.autocomplete_idx = 0;
    }

    pub fn autocomplete_next(&mut self) {
        if !self.autocomplete_list.is_empty() {
            self.autocomplete_idx = (self.autocomplete_idx + 1) % self.autocomplete_list.len();
        }
    }

    pub fn autocomplete_prev(&mut self) {
        if !self.autocomplete_list.is_empty() {
            self.autocomplete_idx = if self.autocomplete_idx == 0 {
                self.autocomplete_list.len() - 1
            } else {
                self.autocomplete_idx - 1
            };
        }
    }

    pub fn autocomplete_select(&mut self) {
        if self.autocomplete_list.is_empty() {
            return;
        }

        let selection = self.autocomplete_list[self.autocomplete_idx].clone();
        let trigger = match self.autocomplete_mode {
            Some(AutocompleteMode::Agent) => "@",
            Some(AutocompleteMode::File) => "/",
            None => return,
        };

        let last_word_start = self
            .input_text
            .rfind(|c: char| c.is_whitespace())
            .map(|i| i + 1)
            .unwrap_or(0);

        self.input_text.truncate(last_word_start);
        self.input_text.push_str(trigger);
        self.input_text.push_str(&selection);
        self.input_text.push(' ');

        self.autocomplete_mode = None;
        self.autocomplete_list.clear();
        self.autocomplete_idx = 0;
        self.autocomplete_query.clear();
    }

    pub fn cancel_autocomplete(&mut self) {
        self.autocomplete_mode = None;
        self.autocomplete_list.clear();
        self.autocomplete_idx = 0;
        self.autocomplete_query.clear();
    }
}

pub fn format_elapsed_time(iso_timestamp: &str) -> String {
    let created = parse_iso_timestamp(iso_timestamp);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let elapsed = now.saturating_sub(created);

    if elapsed < 60 {
        format!("{}s", elapsed)
    } else if elapsed < 3600 {
        format!("{}m{}s", elapsed / 60, elapsed % 60)
    } else {
        format!("{}h{}m", elapsed / 3600, (elapsed % 3600) / 60)
    }
}

fn parse_iso_timestamp(iso_str: &str) -> u64 {
    let iso_str = iso_str.replace('T', " ").replace('Z', "");
    let parts: Vec<&str> = iso_str.split(' ').collect();

    if parts.len() < 2 {
        return SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    let date_parts: Vec<&str> = parts[0].split('-').collect();
    let time_parts: Vec<&str> = parts[1].split(':').collect();

    if date_parts.len() < 3 || time_parts.len() < 3 {
        return SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    let year = date_parts[0].parse::<i32>().unwrap_or(1970);
    let month = date_parts[1].parse::<u32>().unwrap_or(1);
    let day = date_parts[2].parse::<u32>().unwrap_or(1);
    let hour = time_parts[0].parse::<u32>().unwrap_or(0);
    let minute = time_parts[1].parse::<u32>().unwrap_or(0);
    let second = time_parts[2]
        .split('.')
        .next()
        .unwrap_or("0")
        .parse::<u32>()
        .unwrap_or(0);

    let days_since_epoch =
        (year - 1970) * 365 + (year - 1969) / 4 - (year - 1901) / 100 + (year - 1601) / 400;
    let days_in_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut days = days_since_epoch;

    for m in 1..month.min(13) as usize {
        days += days_in_month[m - 1];
        if m == 2 && year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
            days += 1;
        }
    }

    days += day as i32 - 1;

    (days as u64 * 86400) + (hour as u64 * 3600) + (minute as u64 * 60) + second as u64
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
        state.messages = vec![
            Message {
                message_id: "m1".to_string(),
                channel_id: "ch1".to_string(),
                agent_id: "hailot".to_string(),
                content: "msg1".to_string(),
                created_at: "2025-11-05T00:00:00Z".to_string(),
            },
            Message {
                message_id: "m2".to_string(),
                channel_id: "ch1".to_string(),
                agent_id: "hailot".to_string(),
                content: "msg2".to_string(),
                created_at: "2025-11-05T00:01:00Z".to_string(),
            },
        ];

        assert_eq!(state.message_scroll_offset, 0);
        state.scroll_messages_down();
        assert_eq!(state.message_scroll_offset, 1);
        state.scroll_messages_down();
        assert_eq!(state.message_scroll_offset, 1);
        state.scroll_messages_up();
        assert_eq!(state.message_scroll_offset, 0);
        state.scroll_messages_up();
        assert_eq!(state.message_scroll_offset, 0);
    }
}
