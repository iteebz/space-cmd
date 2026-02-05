use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use crate::app::AppState;

const TIME_SLICE_START: usize = 11;
const TIME_SLICE_END: usize = 19;

pub fn render(frame: &mut Frame, app_state: &AppState, area: Rect) {
    let title = match app_state.selected_spawn() {
        Some(spawn) => {
            let identity = app_state.resolve_identity(&spawn.agent_id);
            format!("Spawn: {} [{}]", identity, spawn.status)
        }
        None => "No spawn selected".to_string(),
    };

    let items: Vec<ListItem> = if app_state.spawn_activity.is_empty() {
        vec![ListItem::new(Span::styled(
            "No activity",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        app_state
            .spawn_activity
            .iter()
            .skip(app_state.spawn_activity_scroll_offset)
            .take(area.height.saturating_sub(2) as usize)
            .map(|act| {
                let timestamp = act
                    .created_at
                    .get(TIME_SLICE_START..TIME_SLICE_END)
                    .unwrap_or("??:??:??");

                let action_color = match act.action.as_str() {
                    "created" => Color::Green,
                    "started" => Color::Cyan,
                    "completed" => Color::Blue,
                    "failed" => Color::Red,
                    "claimed" => Color::Yellow,
                    _ => Color::White,
                };

                let detail = match act.after.as_deref() {
                    Some(after) if after.len() > 50 => format!(" {}...", &after[..47]),
                    Some(after) => format!(" {}", after),
                    None => String::new(),
                };

                let line = Line::from(vec![
                    Span::styled(
                        format!("{} ", timestamp),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        format!("{} ", act.primitive),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(
                        act.action.clone(),
                        Style::default()
                            .fg(action_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(detail, Style::default().fg(Color::DarkGray)),
                ]);

                ListItem::new(line)
            })
            .collect()
    };

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
