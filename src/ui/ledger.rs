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

fn primitive_color(primitive: &str) -> Color {
    match primitive {
        "decision" => Color::Magenta,
        "insight" => Color::Yellow,
        "task" => Color::Cyan,
        _ => Color::White,
    }
}

pub fn render(frame: &mut Frame, app_state: &AppState, area: Rect) {
    let count = app_state.ledger.len();
    let title = format!("Ledger ({})", count);

    let items: Vec<ListItem> = app_state
        .ledger
        .iter()
        .skip(app_state.ledger_scroll_offset)
        .take(area.height.saturating_sub(2) as usize)
        .map(|act| {
            let timestamp = act
                .created_at
                .get(TIME_SLICE_START..TIME_SLICE_END)
                .unwrap_or("??:??:??");

            let identity = app_state.resolve_identity(&act.agent_id);

            let action_color = match act.action.as_str() {
                "created" => Color::Green,
                "completed" => Color::Blue,
                "claimed" => Color::Yellow,
                "failed" => Color::Red,
                _ => Color::White,
            };

            let detail = match act.after.as_deref() {
                Some(after) if after.len() > 40 => format!(" {}...", &after[..37]),
                Some(after) => format!(" {}", after),
                None => String::new(),
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{} ", timestamp),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{:8} ", identity),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:8} ", act.primitive),
                    Style::default().fg(primitive_color(&act.primitive)),
                ),
                Span::styled(act.action.clone(), Style::default().fg(action_color)),
                Span::styled(detail, Style::default().fg(Color::DarkGray)),
            ]);

            ListItem::new(line)
        })
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
