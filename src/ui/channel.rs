use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use crate::app::AppState;

pub fn render(frame: &mut Frame, app_state: &AppState, area: Rect) {
    let title = if let Some(channel) = app_state.current_channel() {
        format!("Channel: {}", channel.name)
    } else {
        "No channel selected".to_string()
    };

    let mut items: Vec<ListItem> = app_state
        .messages
        .iter()
        .rev()
        .enumerate()
        .filter_map(|(idx, msg)| {
            let scroll_pos = app_state.message_scroll_offset;
            if idx >= scroll_pos && idx < scroll_pos + 100 {
                Some((idx - scroll_pos, msg))
            } else {
                None
            }
        })
        .map(|(_, msg)| {
            let timestamp = msg.created_at.get(11..19).unwrap_or("??:??:??");
            let agent_color = if msg.agent_id == "human" {
                Color::Green
            } else {
                Color::Cyan
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{} ", timestamp),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{:12}", msg.agent_id),
                    Style::default()
                        .fg(agent_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(" {}", msg.content)),
            ]);

            ListItem::new(line)
        })
        .collect();

    if items.is_empty() {
        items.push(ListItem::new(Span::styled(
            "No messages",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .title_alignment(Alignment::Left)
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(list, area);
}
