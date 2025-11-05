use super::AppState;

impl AppState {
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
}
