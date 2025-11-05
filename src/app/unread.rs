use super::AppState;

impl AppState {
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
}
