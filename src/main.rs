mod app;
mod db;
mod schema;

use app::{AppState, SidebarTab};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::{io, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    db::check_schema_version().map_err(|e| format!("Schema check failed: {}", e))?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app_state = AppState::new();

    app_state.channels = db::get_channels().unwrap_or_else(|_| vec![]);
    app_state.spawns = db::get_spawns().unwrap_or_else(|_| vec![]);

    loop {
        if let Some(channel) = app_state.current_channel() {
            app_state.messages =
                db::get_channel_messages(&channel.channel_id).unwrap_or_else(|_| vec![]);
            app_state.mark_channel_read();
        }

        terminal.draw(|frame| {
            render_ui(frame, &app_state);
        })?;

        if event::poll(Duration::from_millis(500))?
            && let Event::Key(key) = event::read()?
        {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('h') => app_state.switch_tab(),
                KeyCode::Char('l') => app_state.switch_tab(),
                KeyCode::Char('j') | KeyCode::Down => {
                    if app_state.active_tab == SidebarTab::Channels {
                        app_state.scroll_messages_down();
                    } else {
                        app_state.next_in_sidebar();
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if app_state.active_tab == SidebarTab::Channels {
                        app_state.scroll_messages_up();
                    } else {
                        app_state.prev_in_sidebar();
                    }
                }
                KeyCode::Char(' ') => app_state.toggle_spawn_expansion(),
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}

fn render_ui(frame: &mut ratatui::Frame, app_state: &AppState) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    let content_area = main_layout[0];
    let input_area = main_layout[1];

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(content_area);

    let sidebar_area = horizontal[0];
    let right_pane_area = horizontal[1];

    render_sidebar(frame, app_state, sidebar_area);
    render_right_pane(frame, app_state, right_pane_area);
    render_input_bar(frame, input_area);
}

fn render_sidebar(frame: &mut ratatui::Frame, app_state: &AppState, area: Rect) {
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

    let inner_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(1)])
        .split(area);

    frame.render_widget(tabs_widget, inner_layout[0]);

    match app_state.active_tab {
        SidebarTab::Channels => render_channels_list(frame, app_state, inner_layout[1]),
        SidebarTab::Spawns => render_spawns_list(frame, app_state, inner_layout[1]),
    }
}

fn render_channels_list(frame: &mut ratatui::Frame, app_state: &AppState, area: Rect) {
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

fn render_spawns_list(frame: &mut ratatui::Frame, app_state: &AppState, area: Rect) {
    let items: Vec<ListItem> = app_state
        .spawns
        .iter()
        .enumerate()
        .map(|(idx, spawn)| {
            let indicator = if idx == app_state.active_spawn_idx {
                ">"
            } else {
                " "
            };
            let status_short = spawn.status.get(0..1).unwrap_or("?").to_uppercase();
            let name = format!(
                "{} {}#{} ({})",
                indicator,
                status_short,
                spawn.id.get(0..7).unwrap_or("?"),
                spawn.status
            );
            ListItem::new(name)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    frame.render_widget(list, area);
}

fn render_right_pane(frame: &mut ratatui::Frame, app_state: &AppState, area: Rect) {
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

fn render_input_bar(frame: &mut ratatui::Frame, area: Rect) {
    let input = Paragraph::new("/bridge send general ")
        .block(Block::default().borders(Borders::TOP))
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(input, area);
}
