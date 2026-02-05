use super::{AppState, SidebarTab};

impl AppState {
    pub fn switch_tab(&mut self) {
        self.active_tab = match self.active_tab {
            SidebarTab::Agents => SidebarTab::Spawns,
            SidebarTab::Spawns => SidebarTab::Agents,
        };
        self.sidebar_scroll_offset = 0;
    }

    pub fn next_in_sidebar(&mut self) {
        match self.active_tab {
            SidebarTab::Agents => {
                if !self.agents.is_empty() {
                    self.active_agent_idx = (self.active_agent_idx + 1) % self.agents.len();
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
            SidebarTab::Agents => {
                if !self.agents.is_empty() {
                    self.active_agent_idx = if self.active_agent_idx == 0 {
                        self.agents.len() - 1
                    } else {
                        self.active_agent_idx - 1
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

    pub fn next_spawn_global(&mut self) {
        if !self.spawns.is_empty() {
            self.selected_spawn_idx = Some(match self.selected_spawn_idx {
                None => 0,
                Some(idx) => (idx + 1) % self.spawns.len(),
            });
            self.load_spawn_activity();
        }
    }

    pub fn prev_spawn_global(&mut self) {
        if !self.spawns.is_empty() {
            self.selected_spawn_idx = Some(match self.selected_spawn_idx {
                None => self.spawns.len() - 1,
                Some(0) => self.spawns.len() - 1,
                Some(idx) => idx - 1,
            });
            self.load_spawn_activity();
        }
    }

    pub fn load_spawn_activity(&mut self) {
        if let Some(spawn) = self.selected_spawn() {
            let spawn_id = spawn.id.clone();
            self.spawn_activity = crate::db::get_spawn_activity(&spawn_id, 200).unwrap_or_default();
        } else {
            self.spawn_activity.clear();
        }
        self.spawn_activity_scroll_offset = 0;
    }

    pub fn focus_agent_by_initial(&mut self, ch: char) -> bool {
        let ch_lower = ch.to_ascii_lowercase();
        if let Some(idx) = self
            .agents
            .iter()
            .position(|a| a.identity.starts_with(ch_lower))
        {
            self.active_tab = SidebarTab::Agents;
            self.active_agent_idx = idx;
            self.all_stream = false;
            self.activity_scroll_offset = 0;
            self.sidebar_scroll_offset = 0;
            true
        } else {
            false
        }
    }
}
