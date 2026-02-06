use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::AppState;
use crate::source::Mode;
use crate::time::format_elapsed_time;

pub fn render(frame: &mut Frame, app_state: &AppState, area: Rect) {
    let d = &app_state.daemon;

    let (daemon_icon, daemon_color) = if d.running {
        ("●", Color::Green)
    } else {
        ("○", Color::Red)
    };

    let (swarm_label, swarm_color) = if d.enabled {
        ("ON", Color::Green)
    } else {
        ("OFF", Color::DarkGray)
    };

    let slots = format!("{}/{}", d.active_count, d.concurrency);

    let skip_text = d
        .last_skip
        .as_deref()
        .map(|ts| format!(" skip:{}", format_elapsed_time(ts)))
        .unwrap_or_default();

    let line = Line::from(vec![
        Span::styled(
            format!(" {} ", daemon_icon),
            Style::default().fg(daemon_color),
        ),
        Span::styled(
            format!("SWARM {} ", swarm_label),
            Style::default()
                .fg(swarm_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(slots, Style::default().fg(Color::Cyan)),
        Span::styled(skip_text, Style::default().fg(Color::DarkGray)),
        Span::styled(
            match app_state.source_mode {
                Mode::Api => " [API]",
                Mode::Db => " [DB]",
            },
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let widget = Paragraph::new(line).style(Style::default().fg(Color::White));
    frame.render_widget(widget, area);
}
