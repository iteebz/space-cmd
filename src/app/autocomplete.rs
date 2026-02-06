use super::{AppState, AutocompleteMode};

impl AppState {
    fn last_word_start(&self) -> usize {
        self.input_text
            .rfind(|c: char| c.is_whitespace())
            .map(|i| i + 1)
            .unwrap_or(0)
    }

    pub fn detect_and_trigger_autocomplete(&mut self) {
        if self.input_text.is_empty() {
            return;
        }

        let last_word_start = self.last_word_start();
        let last_word = &self.input_text[last_word_start..];

        if let Some(query) = last_word.strip_prefix('@') {
            self.autocomplete_mode = Some(AutocompleteMode::Agent);
            self.autocomplete_query = query.to_string();
        } else if let Some(query) = last_word.strip_prefix('/')
            && !query.is_empty()
        {
            self.autocomplete_mode = Some(AutocompleteMode::File);
            self.autocomplete_query = query.to_string();
            self.load_file_autocomplete();
        }
    }

    pub async fn load_agent_autocomplete(&mut self) {
        if let Ok(agents) = crate::api::get_agents().await {
            self.autocomplete_list = agents.iter().map(|a| a.identity.clone()).collect();
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

        let start = self.last_word_start();
        self.input_text.truncate(start);
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
