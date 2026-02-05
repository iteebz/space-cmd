use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

use crate::app::{AppState, RightPane};

mod activity;
mod input;
mod ledger;
mod sidebar;
mod status;
mod stream;

pub fn render_ui(frame: &mut Frame, app_state: &AppState) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let content_area = main_layout[0];
    let status_area = main_layout[1];
    let input_area = main_layout[2];

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(content_area);

    let sidebar_area = horizontal[0];
    let activity_area = horizontal[1];
    let right_area = horizontal[2];

    sidebar::render_sidebar(frame, app_state, sidebar_area);
    activity::render(frame, app_state, activity_area);
    match app_state.right_pane {
        RightPane::Stream => stream::render(frame, app_state, right_area),
        RightPane::Ledger => ledger::render(frame, app_state, right_area),
    }
    status::render(frame, app_state, status_area);
    input::render_input_bar(frame, app_state, input_area);
}
