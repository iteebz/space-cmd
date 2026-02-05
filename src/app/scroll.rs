use super::AppState;

impl AppState {
    pub fn scroll_activity_down(&mut self) {
        let max_scroll = self.activity.len().saturating_sub(1);
        self.activity_scroll_offset = (self.activity_scroll_offset + 1).min(max_scroll);
    }

    pub fn scroll_activity_up(&mut self) {
        self.activity_scroll_offset = self.activity_scroll_offset.saturating_sub(1);
    }

    pub fn scroll_spawn_activity_down(&mut self) {
        let max_scroll = self.spawn_activity.len().saturating_sub(1);
        self.spawn_activity_scroll_offset = (self.spawn_activity_scroll_offset + 1).min(max_scroll);
    }

    pub fn scroll_spawn_activity_up(&mut self) {
        self.spawn_activity_scroll_offset = self.spawn_activity_scroll_offset.saturating_sub(1);
    }
}
