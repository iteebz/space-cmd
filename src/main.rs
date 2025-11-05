use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use space_cmd::app::AppState;
use space_cmd::db;
use space_cmd::ui::render_ui;
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
                KeyCode::Char('j') => {
                    if app_state.active_tab == space_cmd::app::SidebarTab::Channels {
                        app_state.scroll_messages_down();
                    } else {
                        app_state.next_in_sidebar();
                    }
                }
                KeyCode::Char('k') => {
                    if app_state.active_tab == space_cmd::app::SidebarTab::Channels {
                        app_state.scroll_messages_up();
                    } else {
                        app_state.prev_in_sidebar();
                    }
                }
                KeyCode::Char(' ') => app_state.toggle_spawn_expansion(),
                KeyCode::Char(ch) => {
                    app_state.add_char(ch);
                    app_state.detect_and_trigger_autocomplete();
                }
                KeyCode::Backspace => {
                    app_state.backspace();
                    if app_state.autocomplete_mode.is_some() {
                        app_state.detect_and_trigger_autocomplete();
                    }
                }
                KeyCode::Enter => {
                    if app_state.autocomplete_mode.is_some() {
                        app_state.autocomplete_select();
                    } else {
                        let _ = app_state.submit_input();
                    }
                }
                KeyCode::Up => {
                    if app_state.autocomplete_mode.is_some() {
                        app_state.autocomplete_prev();
                    } else {
                        app_state.history_prev();
                    }
                }
                KeyCode::Down => {
                    if app_state.autocomplete_mode.is_some() {
                        app_state.autocomplete_next();
                    } else {
                        app_state.history_next();
                    }
                }
                KeyCode::Esc => {
                    if app_state.autocomplete_mode.is_some() {
                        app_state.cancel_autocomplete();
                    } else {
                        app_state.input_text.clear();
                        app_state.history_idx = None;
                    }
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}
