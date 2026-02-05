use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use crate::app::AppState;
use crate::schema::TailEntry;

fn format_entry<'a>(entry: &'a TailEntry) -> Vec<Span<'a>> {
    let spawn_short = &entry.spawn[..entry.spawn.len().min(8)];

    match entry.entry_type.as_str() {
        "tool" => {
            let name = entry.name.as_deref().unwrap_or("?");
            let args = entry.args.as_deref().unwrap_or("");
            let truncated = if args.len() > 50 {
                format!("{}...", &args[..47])
            } else {
                args.to_string()
            };
            let ctx = entry
                .ctx_pct
                .map(|p| format!(" {}%", p))
                .unwrap_or_default();

            vec![
                Span::styled(
                    format!("{} ", spawn_short),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{:8} ", entry.agent),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(name.to_string(), Style::default().fg(Color::Yellow)),
                Span::styled(format!(" {}", truncated), Style::default().fg(Color::White)),
                Span::styled(ctx, Style::default().fg(Color::DarkGray)),
            ]
        }
        "text" => {
            let content = entry.content.as_deref().unwrap_or("");
            let first_line = content.lines().next().unwrap_or("");
            let truncated = if first_line.len() > 60 {
                format!("{}...", &first_line[..57])
            } else {
                first_line.to_string()
            };

            vec![
                Span::styled(
                    format!("{} ", spawn_short),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{:8} ", entry.agent),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(truncated, Style::default().fg(Color::White)),
            ]
        }
        _ => {
            vec![Span::styled(
                format!("{} {} {}", spawn_short, entry.agent, entry.entry_type),
                Style::default().fg(Color::DarkGray),
            )]
        }
    }
}

fn stream_title(app_state: &AppState, count: usize) -> String {
    if app_state.all_stream {
        format!("Stream (all, {})", count)
    } else if let Some(agent) = app_state.active_agent() {
        format!("Stream ({}, {})", agent.identity, count)
    } else {
        format!("Stream ({})", count)
    }
}

pub fn render(frame: &mut Frame, app_state: &AppState, area: Rect) {
    let count = app_state.stream.len();
    let title = stream_title(app_state, count);

    let items: Vec<ListItem> = app_state
        .stream
        .iter()
        .skip(app_state.stream_scroll_offset)
        .take(area.height.saturating_sub(2) as usize)
        .map(|entry| ListItem::new(Line::from(format_entry(entry))))
        .collect();

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

#[cfg(test)]
mod tests {
    use super::stream_title;
    use crate::app::AppState;

    #[test]
    fn all_stream_title_includes_all_tag() {
        let mut state = AppState::new();
        state.all_stream = true;
        assert_eq!(stream_title(&state, 42), "Stream (all, 42)");
    }
}
