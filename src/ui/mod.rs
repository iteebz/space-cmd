use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

use crate::app::AppState;

mod input;
mod pane;
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
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(content_area);

    let sidebar_area = horizontal[0];
    let right_pane_area = horizontal[1];

    sidebar::render_sidebar(frame, app_state, sidebar_area);
    pane::render_right_pane(frame, app_state, right_pane_area);
    input::render_input_bar(frame, app_state, input_area);
}
