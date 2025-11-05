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

    pub fn next_spawn_global(&mut self) {
        if !self.spawns.is_empty() {
            self.selected_spawn_idx = Some(match self.selected_spawn_idx {
                None => 0,
                Some(idx) => (idx + 1) % self.spawns.len(),
            });
        }
    }

    pub fn prev_spawn_global(&mut self) {
        if !self.spawns.is_empty() {
            self.selected_spawn_idx = Some(match self.selected_spawn_idx {
                None => self.spawns.len() - 1,
                Some(0) => self.spawns.len() - 1,
                Some(idx) => idx - 1,
            });
        }
    }

    pub fn selected_spawn(&self) -> Option<&crate::schema::Spawn> {
        self.selected_spawn_idx.and_then(|idx| self.spawns.get(idx))
    }

    pub fn load_session_events(&mut self) {
        if let Some(spawn) = self.selected_spawn() {
            if let Some(session_id) = &spawn.session_id {
                self.session_events = self.load_session_lines(session_id);
            } else {
                self.session_events.clear();
            }
        } else {
            self.session_events.clear();
        }
    }

    fn sessions_directory(&self) -> std::path::PathBuf {
        use std::path::PathBuf;

        match std::env::var("SPACE_DB") {
            Ok(db_path) => PathBuf::from(db_path)
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| {
                    PathBuf::from(format!(
                        "{}/.space",
                        std::env::var("HOME").unwrap_or_default()
                    ))
                }),
            Err(_) => PathBuf::from(format!(
                "{}/.space/sessions",
                std::env::var("HOME").unwrap_or_default()
            )),
        }
    }

    fn load_session_lines(&self, session_id: &str) -> Vec<crate::session::SessionLine> {
        use crate::parser::SessionMessage;
        use crate::session::SessionRenderer;
        use std::fs;

        let sessions_dir = self.sessions_directory();
        let mut lines = Vec::new();

        if let Ok(entries) = fs::read_dir(&sessions_dir) {
            for provider_dir in entries.filter_map(Result::ok) {
                let path = provider_dir.path();
                if !path.is_dir() {
                    continue;
                }

                let session_path = path.join(format!("{}.jsonl", session_id));
                if let Ok(content) = fs::read_to_string(&session_path) {
                    for line in content.lines() {
                        if !line.is_empty()
                            && let Ok(msg) = SessionMessage::parse(line)
                        {
                            let rendered = SessionRenderer::render(&msg);
                            lines.push(rendered);
                        }
                    }
                }
            }
        }

        lines
    }
}
