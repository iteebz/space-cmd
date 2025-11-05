use serde_json::{Value, json};

#[derive(Debug, Clone)]
pub struct SessionMessage {
    pub msg_type: String,
    pub timestamp: Option<String>,
    pub content: Value,
}

impl SessionMessage {
    pub fn parse(line: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json: Value = serde_json::from_str(line)?;

        let msg_type = json
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let timestamp = json
            .get("timestamp")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let content = json.get("content").cloned().unwrap_or(json!(null));

        Ok(SessionMessage {
            msg_type,
            timestamp,
            content,
        })
    }

    pub fn is_tool_call(&self) -> bool {
        self.msg_type == "tool_call"
    }

    pub fn is_tool_result(&self) -> bool {
        self.msg_type == "tool_result"
    }

    pub fn is_message(&self) -> bool {
        self.msg_type == "message"
    }

    pub fn is_text(&self) -> bool {
        self.msg_type == "text"
    }

    pub fn content_as_str(&self) -> String {
        match &self.content {
            Value::String(s) => s.clone(),
            Value::Object(_) | Value::Array(_) => self.content.to_string(),
            Value::Null => String::new(),
            _ => self.content.to_string(),
        }
    }

    pub fn tool_name(&self) -> Option<String> {
        if self.is_tool_call() {
            self.content
                .get("name")
                .or_else(|| self.content.get("tool_name"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        } else {
            None
        }
    }

    pub fn tool_input(&self) -> Option<Value> {
        if self.is_tool_call() {
            self.content.get("input").cloned()
        } else {
            None
        }
    }

    pub fn tool_result_output(&self) -> Option<String> {
        if self.is_tool_result() {
            self.content
                .get("output")
                .or_else(|| self.content.get("result"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        } else {
            None
        }
    }

    pub fn message_role(&self) -> Option<String> {
        if self.is_message() {
            self.content
                .get("role")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        } else {
            None
        }
    }

    pub fn message_content(&self) -> Option<String> {
        if self.is_message() {
            self.content
                .get("content")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_message_event() {
        let line = r#"{"type": "message", "timestamp": "2025-11-05T10:00:00Z", "content": {"role": "user", "content": "analyze this"}}"#;
        let msg = SessionMessage::parse(line).unwrap();
        assert_eq!(msg.msg_type, "message");
        assert_eq!(msg.message_role(), Some("user".to_string()));
        assert_eq!(msg.message_content(), Some("analyze this".to_string()));
    }

    #[test]
    fn parse_tool_call_event() {
        let line = r#"{"type": "tool_call", "timestamp": "2025-11-05T10:00:01Z", "content": {"name": "read_file", "input": {"path": "src/main.rs"}}}"#;
        let msg = SessionMessage::parse(line).unwrap();
        assert!(msg.is_tool_call());
        assert_eq!(msg.tool_name(), Some("read_file".to_string()));
        assert!(msg.tool_input().is_some());
    }

    #[test]
    fn parse_tool_result_event() {
        let line = r#"{"type": "tool_result", "timestamp": "2025-11-05T10:00:02Z", "content": {"output": "file contents here"}}"#;
        let msg = SessionMessage::parse(line).unwrap();
        assert!(msg.is_tool_result());
        assert_eq!(
            msg.tool_result_output(),
            Some("file contents here".to_string())
        );
    }

    #[test]
    fn parse_text_event() {
        let line =
            r#"{"type": "text", "timestamp": "2025-11-05T10:00:03Z", "content": "response text"}"#;
        let msg = SessionMessage::parse(line).unwrap();
        assert!(msg.is_text());
        assert_eq!(msg.content_as_str(), "response text");
    }

    #[test]
    fn parse_missing_timestamp() {
        let line = r#"{"type": "message", "content": {"role": "assistant", "content": "hello"}}"#;
        let msg = SessionMessage::parse(line).unwrap();
        assert!(msg.timestamp.is_none());
    }
}
