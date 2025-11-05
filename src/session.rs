use crate::parser::SessionMessage;

pub struct SessionLine {
    pub header: String,
    pub body: String,
    pub is_error: bool,
}

pub struct SessionRenderer;

impl SessionRenderer {
    pub fn render(msg: &SessionMessage) -> SessionLine {
        match msg.msg_type.as_str() {
            "message" => Self::render_message(msg),
            "text" => Self::render_text(msg),
            "tool_call" => Self::render_tool_call(msg),
            "tool_result" => Self::render_tool_result(msg),
            _ => Self::render_unknown(msg),
        }
    }

    fn render_message(msg: &SessionMessage) -> SessionLine {
        let time = Self::format_timestamp(&msg.timestamp);
        let role = msg.message_role().unwrap_or_else(|| "unknown".to_string());
        let content = msg.message_content().unwrap_or_default();

        let header = format!("{} | {}", time, Self::color_role(&role));
        let body = content;

        SessionLine {
            header,
            body,
            is_error: false,
        }
    }

    fn render_text(msg: &SessionMessage) -> SessionLine {
        let time = Self::format_timestamp(&msg.timestamp);
        let content = msg.content_as_str();

        let header = format!("{} | [Response]", time);
        let body = content;

        SessionLine {
            header,
            body,
            is_error: false,
        }
    }

    fn render_tool_call(msg: &SessionMessage) -> SessionLine {
        let time = Self::format_timestamp(&msg.timestamp);
        let tool_name = msg.tool_name().unwrap_or_else(|| "unknown".to_string());
        let input = msg.tool_input().map(|v| v.to_string()).unwrap_or_default();

        let header = format!("{} | [Tool] {}", time, tool_name);
        let body = format!("input: {}", input);

        SessionLine {
            header,
            body,
            is_error: false,
        }
    }

    fn render_tool_result(msg: &SessionMessage) -> SessionLine {
        let time = Self::format_timestamp(&msg.timestamp);
        let output = msg
            .tool_result_output()
            .unwrap_or_else(|| "no output".to_string());

        let header = format!("{} | [Result]", time);
        let is_error = Self::is_error_result(&output);

        SessionLine {
            header,
            body: output,
            is_error,
        }
    }

    fn render_unknown(msg: &SessionMessage) -> SessionLine {
        let time = Self::format_timestamp(&msg.timestamp);
        let content = msg.content_as_str();

        let header = format!("{} | [{}]", time, msg.msg_type);
        let body = content;

        SessionLine {
            header,
            body,
            is_error: false,
        }
    }

    fn format_timestamp(ts: &Option<String>) -> String {
        match ts {
            Some(ts_str) => {
                if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(ts_str) {
                    datetime.format("%H:%M:%S").to_string()
                } else {
                    "??:??:??".to_string()
                }
            }
            None => "??:??:??".to_string(),
        }
    }

    fn color_role(role: &str) -> String {
        match role {
            "user" => format!("\u{1b}[36m{}\u{1b}[0m", role.to_uppercase()),
            "assistant" => format!("\u{1b}[32m{}\u{1b}[0m", role),
            _ => role.to_string(),
        }
    }

    fn is_error_result(output: &str) -> bool {
        let lower = output.to_lowercase();
        lower.contains("error") || lower.contains("failed") || lower.contains("exception")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn render_message_user() {
        let msg = SessionMessage {
            msg_type: "message".to_string(),
            timestamp: Some("2025-11-05T10:00:00Z".to_string()),
            content: json!({"role": "user", "content": "analyze this"}),
        };

        let rendered = SessionRenderer::render(&msg);
        assert_eq!(rendered.body, "analyze this");
        assert!(!rendered.is_error);
    }

    #[test]
    fn render_tool_call() {
        let msg = SessionMessage {
            msg_type: "tool_call".to_string(),
            timestamp: Some("2025-11-05T10:00:01Z".to_string()),
            content: json!({"name": "read_file", "input": {"path": "main.rs"}}),
        };

        let rendered = SessionRenderer::render(&msg);
        assert!(rendered.header.contains("[Tool] read_file"));
        assert!(rendered.body.contains("path"));
    }

    #[test]
    fn render_tool_result_error() {
        let msg = SessionMessage {
            msg_type: "tool_result".to_string(),
            timestamp: Some("2025-11-05T10:00:02Z".to_string()),
            content: json!({"output": "Error: file not found"}),
        };

        let rendered = SessionRenderer::render(&msg);
        assert!(rendered.is_error);
    }

    #[test]
    fn render_text() {
        let msg = SessionMessage {
            msg_type: "text".to_string(),
            timestamp: Some("2025-11-05T10:00:03Z".to_string()),
            content: json!("This is the response"),
        };

        let rendered = SessionRenderer::render(&msg);
        assert!(rendered.header.contains("[Response]"));
        assert_eq!(rendered.body, "This is the response");
    }
}
