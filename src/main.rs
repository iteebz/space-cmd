mod db;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};
use std::{io, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check schema compatibility on startup
    db::check_schema_version().map_err(|e| format!("Schema check failed: {}", e))?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Hardcoded channel for now - will add tabs later
    let channel = "general";

    // Main loop
    loop {
        // Fetch messages (poll every iteration)
        let messages = db::get_channel_messages(channel).unwrap_or_else(|_| vec![]);

        terminal.draw(|frame| {
            let area = frame.area();

            // Convert messages to list items
            let items: Vec<ListItem> = messages
                .iter()
                .map(|msg| {
                    let timestamp = msg.created_at.get(11..19).unwrap_or("??:??:??");

                    // Color-code: agents = cyan, human = green
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
                        Span::raw(format!(" > {}", msg.content)),
                    ]);

                    ListItem::new(line)
                })
                .collect();

            let list = List::new(items).block(
                Block::default()
                    .title(format!("📡 {}", channel))
                    .title_alignment(Alignment::Left)
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White)),
            );

            frame.render_widget(list, area);
        })?;

        // Handle input (poll every 500ms like Council)
        if event::poll(Duration::from_millis(500))?
            && let Event::Key(key) = event::read()?
            && key.code == KeyCode::Char('q')
        {
            break;
        }
    }

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}
