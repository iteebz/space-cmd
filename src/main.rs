use clap::{Parser, Subcommand};
use crossterm::{
    event::{Event, EventStream, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures::StreamExt;
use ratatui::{Terminal, backend::CrosstermBackend};
use space_cmd::app::{AppState, RightPane};
use space_cmd::health;
use space_cmd::source::Source;
use space_cmd::ui::render_ui;
use std::{io, time::Duration};

#[derive(Parser)]
#[command(name = "space-cmd")]
#[command(about = "Command center for space agents", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Check repository health
    Health {
        /// Show detailed output
        #[arg(short, long)]
        verbose: bool,

        /// Automatically create a task if health is degraded
        #[arg(long)]
        auto_task: bool,
    },

    /// Create a task
    Task {
        /// Task content
        content: String,
    },
}

fn handle_scroll_down(app_state: &mut AppState) {
    match app_state.right_pane {
        RightPane::Stream => app_state.scroll_stream_down(),
        RightPane::Ledger => app_state.scroll_ledger_down(),
    }
}

fn handle_scroll_up(app_state: &mut AppState) {
    match app_state.right_pane {
        RightPane::Stream => app_state.scroll_stream_up(),
        RightPane::Ledger => app_state.scroll_ledger_up(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Health { verbose, auto_task }) => {
            let result = health::calculate_health().await;
            println!("Health Score: {}/100", result.score);
            println!(
                "API: {} (ok: {}, latency: {:?}ms)",
                result.api_base_url, result.api_ok, result.api_latency_ms
            );
            println!(
                "Freshness: ledger={:?}s spawns={:?}s",
                result.ledger_freshness_s, result.spawns_freshness_s
            );
            if let Some(repo) = &result.repo {
                if repo.is_git_repo {
                    println!(
                        "Repo: branch={} clean={} last_commit={:?}s ago",
                        repo.branch.as_deref().unwrap_or("?"),
                        repo.is_clean,
                        repo.last_commit_age_s
                    );
                } else {
                    println!("Repo: not a git repository");
                }
            }

            if verbose || result.score < 100 {
                for detail in &result.details {
                    println!("- {}", detail);
                }
            }

            if auto_task && result.score < 100 {
                let human_agent = space_cmd::api::get_human_agent().await.unwrap_or(None);

                if let Some(agent) = human_agent {
                    let task_content =
                        format!("fix space-cmd health: {}", result.details.join(", "));
                    match space_cmd::api::create_task(&task_content, &agent.id).await {
                        Ok(_) => println!("Task created to fix health issues."),
                        Err(e) => eprintln!("Failed to create task: {}", e),
                    }
                } else {
                    eprintln!("No human agent found to create task.");
                }
            }

            if result.score < 100 {
                std::process::exit(1);
            }
            Ok(())
        }
        Some(Commands::Task { content }) => {
            let human_agent = space_cmd::api::get_human_agent().await?;
            if let Some(agent) = human_agent {
                match space_cmd::api::create_task(&content, &agent.id).await {
                    Ok(_) => println!("Task created."),
                    Err(e) => eprintln!("Failed to create task: {}", e),
                }
            } else {
                eprintln!("No human agent found (required for identity).");
                std::process::exit(1);
            }
            Ok(())
        }
        None => run_tui().await,
    }
}

async fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
    let src = Source::connect();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app_state = AppState::new();

    // Initial fetch
    let (agents, spawns, identities) = tokio::join!(
        src.get_agents(),
        src.get_spawns(),
        src.get_agent_identities()
    );
    app_state.agents = agents;
    app_state.spawns = spawns;
    app_state.agent_identities = identities;

    app_state.activity = if let Some(agent) = app_state.active_agent() {
        src.get_agent_activity(&agent.id, 500).await
    } else {
        vec![]
    };
    app_state.ledger = src.get_ledger_activity(500).await;
    app_state.stream = src.get_tail(200).await;

    let mut reader = EventStream::new();
    let mut interval = tokio::time::interval(Duration::from_millis(500));

    loop {
        let mut should_fetch = false;
        let mut event_received = None;

        tokio::select! {
            _ = interval.tick() => {
                should_fetch = true;
            }
            Some(Ok(event)) = reader.next() => {
                event_received = Some(event);
            }
        }

        if let Some(Event::Key(key)) = event_received {
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
                KeyCode::Char('d') => app_state.toggle_right_pane(),
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
                        let _ = app_state.submit_input(); // This calls create_task which needs to be async or handled
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

        if should_fetch && !app_state.paused {
            // Parallel fetch using join!
            let (agents, spawns, identities) = tokio::join!(
                src.get_agents(),
                src.get_spawns(),
                src.get_agent_identities()
            );

            app_state.agents = agents;
            app_state.spawns = spawns;
            app_state.agent_identities = identities;

            // Dependent fetches
            app_state.activity = if app_state.all_stream {
                src.get_activity(500).await
            } else if let Some(agent) = app_state.active_agent() {
                src.get_agent_activity(&agent.id, 500).await
            } else {
                vec![]
            };

            if let Some(spawn) = app_state.selected_spawn() {
                app_state.spawn_activity = src.get_spawn_activity(&spawn.id, 200).await;
            }

            match app_state.right_pane {
                RightPane::Stream => {
                    app_state.stream = if app_state.all_stream {
                        src.get_tail(200).await
                    } else if let Some(agent) = app_state.active_agent() {
                        src.get_agent_tail(&agent.identity, 200).await
                    } else {
                        src.get_tail(200).await
                    };
                }
                RightPane::Ledger => {
                    app_state.ledger = src.get_ledger_activity(500).await;
                }
            }

            let active_count = app_state
                .spawns
                .iter()
                .filter(|s| s.status == "active")
                .count();
            app_state.daemon = src.get_daemon_status(active_count).await;
        }

        terminal.draw(|frame| {
            render_ui(frame, &app_state);
        })?;
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}
