use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use crate::app::AppState;

pub fn render(frame: &mut Frame, app_state: &AppState, area: Rect) {
    let title = if let Some(spawn) = app_state.selected_spawn() {
        format!("Session: {}", spawn.id)
    } else {
        "No spawn selected".to_string()
    };

    let mut items: Vec<ListItem> = app_state
        .session_events
        .iter()
        .rev()
        .enumerate()
        .filter_map(|(idx, event)| {
            let scroll_pos = app_state.session_scroll_offset;
            if idx >= scroll_pos && idx < scroll_pos + 100 {
                Some((idx - scroll_pos, event))
            } else {
                None
            }
        })
        .map(|(_, event)| {
            let header_style = if event.is_error {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Cyan)
            };

            let lines = vec![
                Line::from(Span::styled(
                    event.header.clone(),
                    header_style.add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::raw(format!("  {}", event.body))),
            ];

            ListItem::new(lines)
        })
        .collect();

    if items.is_empty() {
        items.push(ListItem::new(Span::styled(
            "No events",
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
