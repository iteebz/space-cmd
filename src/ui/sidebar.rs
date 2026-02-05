use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
};

use crate::app::{AppState, SidebarTab};
use crate::time::format_elapsed_time;

pub fn render_sidebar(frame: &mut Frame, app_state: &AppState, area: Rect) {
    let tab_titles = vec!["AGENTS", "SPAWNS"];
    let tab_index = match app_state.active_tab {
        SidebarTab::Agents => 0,
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
        SidebarTab::Agents => render_agents_list(frame, app_state, inner_layout[1]),
        SidebarTab::Spawns => render_spawns_list(frame, app_state, inner_layout[1]),
    }
}

fn render_agents_list(frame: &mut Frame, app_state: &AppState, area: Rect) {
    let items: Vec<ListItem> = app_state
        .agents
        .iter()
        .enumerate()
        .map(|(idx, agent)| {
            let indicator = if idx == app_state.active_agent_idx {
                ">"
            } else {
                " "
            };

            let type_icon = match agent.agent_type.as_str() {
                "ai" => "~",
                "human" => "*",
                _ => "?",
            };

            let name = format!("{} {} {}", indicator, type_icon, agent.identity);
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
        let is_selected = app_state.selected_spawn_idx == Some(idx);
        let is_expanded = app_state.expanded_spawns.contains(&spawn.id);

        let indicator = match (is_focused, is_selected, is_expanded) {
            (_, true, _) => "*",
            (true, _, _) => ">",
            (_, _, true) => "v",
            _ => " ",
        };

        let status_icon = match spawn.status.as_str() {
            "active" => "●",
            "done" if spawn.error.is_some() => "x",
            "done" => ".",
            _ => "?",
        };

        let elapsed = format_elapsed_time(&spawn.created_at);
        let identity = app_state.resolve_identity(&spawn.agent_id);
        let name = format!("{} {} {} ({})", indicator, status_icon, identity, elapsed);

        items.push(ListItem::new(name));

        if is_expanded {
            if let Some(summary) = &spawn.summary {
                let truncated = if summary.len() > 40 {
                    format!("  {}...", &summary[..37])
                } else {
                    format!("  {}", summary)
                };
                items.push(ListItem::new(truncated));
            }
            if let Some(error) = &spawn.error {
                let truncated = if error.len() > 40 {
                    format!("  err: {}...", &error[..34])
                } else {
                    format!("  err: {}", error)
                };
                items.push(ListItem::new(truncated));
            }
        }
    }

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    frame.render_widget(list, area);
}
