use crate::schema::{Activity, Agent, DaemonStatus, Spawn, TailEntry};
use crate::source;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RightPane {
    Stream,
    Ledger,
}

mod autocomplete;
mod input;
mod navigation;
mod scroll;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SidebarTab {
    Agents,
    Spawns,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AutocompleteMode {
    Agent,
    File,
}

pub struct AppState {
    pub paused: bool,
    pub all_stream: bool,
    pub right_pane: RightPane,
    pub source_mode: source::Mode,
    pub active_tab: SidebarTab,
    pub active_agent_idx: usize,
    pub active_spawn_idx: usize,
    pub selected_spawn_idx: Option<usize>,
    pub expanded_spawns: HashSet<String>,

    pub agents: Vec<Agent>,
    pub spawns: Vec<Spawn>,
    pub activity: Vec<Activity>,
    pub spawn_activity: Vec<Activity>,
    pub stream: Vec<TailEntry>,
    pub ledger: Vec<Activity>,
    pub agent_identities: HashMap<String, String>,
    pub daemon: DaemonStatus,

    pub activity_scroll_offset: usize,
    pub sidebar_scroll_offset: usize,
    pub spawn_activity_scroll_offset: usize,
    pub stream_scroll_offset: usize,
    pub ledger_scroll_offset: usize,

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
            paused: false,
            all_stream: false,
            right_pane: RightPane::Stream,
            source_mode: source::Mode::Db,
            active_tab: SidebarTab::Spawns,
            active_agent_idx: 0,
            active_spawn_idx: 0,
            selected_spawn_idx: None,
            expanded_spawns: HashSet::new(),

            agents: vec![],
            spawns: vec![],
            activity: vec![],
            spawn_activity: vec![],
            stream: vec![],
            ledger: vec![],
            agent_identities: HashMap::new(),
            daemon: DaemonStatus::default(),

            activity_scroll_offset: 0,
            sidebar_scroll_offset: 0,
            spawn_activity_scroll_offset: 0,
            stream_scroll_offset: 0,
            ledger_scroll_offset: 0,

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

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn toggle_all_stream(&mut self) {
        self.all_stream = !self.all_stream;
        self.activity_scroll_offset = 0;
        self.stream_scroll_offset = 0;
    }

    pub fn resolve_identity<'a>(&'a self, agent_id: &'a str) -> &'a str {
        self.agent_identities
            .get(agent_id)
            .map(|s| s.as_str())
            .unwrap_or(&agent_id[..agent_id.len().min(8)])
    }

    pub fn active_agent(&self) -> Option<&Agent> {
        self.agents.get(self.active_agent_idx)
    }

    pub fn selected_spawn(&self) -> Option<&Spawn> {
        self.selected_spawn_idx.and_then(|idx| self.spawns.get(idx))
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
