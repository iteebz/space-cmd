use super::{AppState, SidebarTab};

impl AppState {
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
}
