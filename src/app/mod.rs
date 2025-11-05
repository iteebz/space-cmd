use crate::schema::{Channel, Message, Spawn};
use std::collections::{HashMap, HashSet};

mod autocomplete;
mod input;
mod navigation;
mod scroll;
mod unread;

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

    pub fn current_channel(&self) -> Option<&Channel> {
        self.channels.get(self.active_channel_idx)
    }

    #[allow(dead_code)]
    pub fn current_spawn(&self) -> Option<&Spawn> {
        self.spawns.get(self.active_spawn_idx)
    }

    pub fn input_line_count(&self) -> usize {
        self.input_text.lines().count().max(1)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
