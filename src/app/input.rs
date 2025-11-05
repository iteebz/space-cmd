use super::AppState;

impl AppState {
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
}
