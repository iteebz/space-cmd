use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::{AppState, AutocompleteMode};

pub fn render_input_bar(frame: &mut Frame, app_state: &AppState, area: Rect) {
    let prompt = "/bridge send general ";
    let text = format!("{}{}", prompt, app_state.input_text);

    let input = Paragraph::new(text)
        .block(Block::default().borders(Borders::TOP))
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(input, area);

    if let Some(mode) = app_state.autocomplete_mode
        && area.height > 1
    {
        let dropdown_area = Rect {
            x: area.x,
            y: area.y + 1,
            width: area.width,
            height: (area.height - 1).min(10),
        };

        let items: Vec<ListItem> = app_state
            .autocomplete_list
            .iter()
            .enumerate()
            .take(10)
            .map(|(idx, item)| {
                let prefix = if idx == app_state.autocomplete_idx {
                    "➜ "
                } else {
                    "  "
                };

                let style = if idx == app_state.autocomplete_idx {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let label = match mode {
                    AutocompleteMode::Agent => format!("⚡ {}", item),
                    AutocompleteMode::File => format!("📄 {}", item),
                };

                ListItem::new(Span::styled(format!("{}{}", prefix, label), style))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM))
            .style(Style::default().fg(Color::White));

        frame.render_widget(list, dropdown_area);
    }
}
