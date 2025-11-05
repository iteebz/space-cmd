use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
};

use crate::app::{AppState, SidebarTab};
use crate::time::format_elapsed_time;

pub fn render_sidebar(frame: &mut Frame, app_state: &AppState, area: Rect) {
    let tab_titles = vec!["CHANNELS", "SPAWNS"];
    let tab_index = match app_state.active_tab {
        SidebarTab::Channels => 0,
        SidebarTab::Spawns => 1,
    };

    let tabs_widget = ratatui::widgets::Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::BOTTOM))
        .select(tab_index)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    let inner_layout = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(2),
            ratatui::layout::Constraint::Min(1),
        ])
        .split(area);

    frame.render_widget(tabs_widget, inner_layout[0]);

    match app_state.active_tab {
        SidebarTab::Channels => render_channels_list(frame, app_state, inner_layout[1]),
        SidebarTab::Spawns => render_spawns_list(frame, app_state, inner_layout[1]),
    }
}

fn render_channels_list(frame: &mut Frame, app_state: &AppState, area: Rect) {
    let items: Vec<ListItem> = app_state
        .channels
        .iter()
        .enumerate()
        .map(|(idx, ch)| {
            let is_focused = idx == app_state.active_channel_idx;
            let is_unread = app_state.is_channel_unread(&ch.channel_id);

            let indicator = if is_focused {
                ">"
            } else if is_unread {
                "●"
            } else {
                " "
            };

            let name = format!("{} {}", indicator, ch.name);
            ListItem::new(name)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    frame.render_widget(list, area);
}

fn render_spawns_list(frame: &mut Frame, app_state: &AppState, area: Rect) {
    let mut items: Vec<ListItem> = Vec::new();

    for (idx, spawn) in app_state.spawns.iter().enumerate() {
        let is_focused = idx == app_state.active_spawn_idx;
        let is_expanded = app_state.expanded_spawns.contains(&spawn.id);

        let indicator = match (is_focused, is_expanded) {
            (true, _) => ">",
            (false, true) => "▾",
            (false, false) => "▸",
        };

        let status_style = match spawn.status.as_str() {
            "running" => "R",
            "paused" => "P",
            "pending" => "W",
            _ => "?",
        };

        let elapsed = format_elapsed_time(&spawn.created_at);
        let spawn_short = spawn.id.get(0..7).unwrap_or("?");
        let name = format!(
            "{} {}{} ({})",
            indicator, status_style, spawn_short, elapsed
        );

        items.push(ListItem::new(name));

        if is_expanded {
            if let Some(session_id) = &spawn.session_id {
                let transcripts =
                    crate::db::get_transcripts(session_id, 8).unwrap_or_else(|_| vec![]);
                let mut transcript_lines: Vec<String> = transcripts
                    .iter()
                    .rev()
                    .map(|t| {
                        let ts = t.timestamp.get(11..19).unwrap_or("??:??:??");
                        format!("  {} | {}", ts, t.content)
                    })
                    .collect();

                let total_transcripts = transcripts.len();
                if total_transcripts > 8 {
                    transcript_lines.push(format!("  ...{} more", total_transcripts - 8));
                }

                for line in transcript_lines {
                    items.push(ListItem::new(line));
                }
            } else {
                items.push(ListItem::new("  (no session linked)"));
            }
        }
    }

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    frame.render_widget(list, area);
}
