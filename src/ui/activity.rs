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

fn format_activity_line<'a>(
    app_state: &'a AppState,
    act: &'a crate::schema::Activity,
) -> Vec<Span<'a>> {
    let timestamp = act
        .created_at
        .get(TIME_SLICE_START..TIME_SLICE_END)
        .unwrap_or("??:??:??");

    let identity = app_state.resolve_identity(&act.agent_id);

    let action_color = match act.action.as_str() {
        "created" => Color::Green,
        "started" => Color::Cyan,
        "completed" => Color::Blue,
        "failed" => Color::Red,
        "archived" => Color::DarkGray,
        "claimed" => Color::Yellow,
        _ => Color::White,
    };

    let detail = match (act.field.as_deref(), act.after.as_deref()) {
        (Some(field), Some(after)) => {
            let truncated = if after.len() > 60 {
                format!("{}...", &after[..57])
            } else {
                after.to_string()
            };
            format!(" {}={}", field, truncated)
        }
        _ => String::new(),
    };

    vec![
        Span::styled(
            format!("{} ", timestamp),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            format!("{:12}", identity),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {}", act.primitive),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            format!(" {}", act.action),
            Style::default().fg(action_color),
        ),
        Span::raw(detail),
    ]
}

pub fn render(frame: &mut Frame, app_state: &AppState, area: Rect) {
    let count = app_state.activity.len();
    let pause_tag = if app_state.paused { " ⏸" } else { "" };
    let title = if app_state.all_stream {
        format!("Activity (all, {}){}", count, pause_tag)
    } else if let Some(agent) = app_state.active_agent() {
        format!("Activity ({}, {}){}", agent.identity, count, pause_tag)
    } else {
        format!("Activity ({}){}", count, pause_tag)
    };

    let items: Vec<ListItem> = app_state
        .activity
        .iter()
        .skip(app_state.activity_scroll_offset)
        .take(area.height.saturating_sub(2) as usize)
        .map(|act| ListItem::new(Line::from(format_activity_line(app_state, act))))
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
