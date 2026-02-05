use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use space_cmd::app::AppState;
use space_cmd::db;
use space_cmd::ui::render_ui;
use std::{io, time::Duration};

fn handle_scroll_down(app_state: &mut AppState) {
    if app_state.selected_spawn_idx.is_some() {
        app_state.scroll_spawn_activity_down();
    } else {
        app_state.scroll_activity_down();
    }
}

fn handle_scroll_up(app_state: &mut AppState) {
    if app_state.selected_spawn_idx.is_some() {
        app_state.scroll_spawn_activity_up();
    } else {
        app_state.scroll_activity_up();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    db::check_schema_version().map_err(|e| format!("Schema check failed: {}", e))?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app_state = AppState::new();

    app_state.agents = db::get_agents().unwrap_or_default();
    app_state.spawns = db::get_spawns().unwrap_or_default();
    app_state.agent_identities = db::get_agent_identities().unwrap_or_default();
    app_state.activity = if let Some(agent) = app_state.active_agent() {
        db::get_agent_activity(&agent.id, 500).unwrap_or_default()
    } else {
        vec![]
    };

    loop {
        if !app_state.paused {
            app_state.agents = db::get_agents().unwrap_or_default();
            app_state.spawns = db::get_spawns().unwrap_or_default();
            app_state.agent_identities = db::get_agent_identities().unwrap_or_default();

            app_state.activity = if app_state.all_stream {
                db::get_activity(500).unwrap_or_default()
            } else if let Some(agent) = app_state.active_agent() {
                db::get_agent_activity(&agent.id, 500).unwrap_or_default()
            } else {
                vec![]
            };

            if let Some(spawn) = app_state.selected_spawn() {
                app_state.spawn_activity =
                    db::get_spawn_activity(&spawn.id, 200).unwrap_or_default();
            }

            let active_count = app_state
                .spawns
                .iter()
                .filter(|s| s.status == "active")
                .count();
            app_state.daemon = db::get_daemon_status(active_count);
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
                KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    app_state.next_spawn_global();
                }
                KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    app_state.prev_spawn_global();
                }
                KeyCode::Char('j') => {
                    app_state.next_in_sidebar();
                    if !app_state.all_stream {
                        app_state.activity_scroll_offset = 0;
                    }
                }
                KeyCode::Char('k') => {
                    app_state.prev_in_sidebar();
                    if !app_state.all_stream {
                        app_state.activity_scroll_offset = 0;
                    }
                }
                KeyCode::Char('J') => handle_scroll_down(&mut app_state),
                KeyCode::Char('K') => handle_scroll_up(&mut app_state),
                KeyCode::Char(' ') => app_state.toggle_pause(),
                KeyCode::Char('a') => app_state.toggle_all_stream(),
                KeyCode::Char('e') => app_state.toggle_spawn_expansion(),
                KeyCode::Char(ch) if key.modifiers.contains(KeyModifiers::ALT) => {
                    app_state.focus_agent_by_initial(ch);
                }
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
