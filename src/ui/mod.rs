use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

use crate::app::AppState;

mod channel;
mod input;
mod session;
mod sidebar;

pub fn render_ui(frame: &mut Frame, app_state: &AppState) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    let content_area = main_layout[0];
    let input_area = main_layout[1];

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(content_area);

    let sidebar_area = horizontal[0];
    let channel_area = horizontal[1];
    let session_area = horizontal[2];

    sidebar::render_sidebar(frame, app_state, sidebar_area);
    channel::render(frame, app_state, channel_area);
    session::render(frame, app_state, session_area);
    input::render_input_bar(frame, app_state, input_area);
}
