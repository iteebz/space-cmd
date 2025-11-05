use space_cmd::app::AppState;

#[test]
fn characters_accumulate_in_input() {
    let mut state = AppState::new();
    state.add_char('h');
    state.add_char('e');
    state.add_char('l');
    state.add_char('l');
    state.add_char('o');
    assert_eq!(state.input_text, "hello");
}

#[test]
fn backspace_removes_character() {
    let mut state = AppState::new();
    state.input_text = "hello".to_string();
    state.backspace();
    state.backspace();
    assert_eq!(state.input_text, "hel");
}

#[test]
fn submit_stores_in_history() {
    let mut state = AppState::new();
    state.input_text = "/bridge send general @hailot".to_string();

    let cmd = state.submit_input();
    assert_eq!(cmd, Some("/bridge send general @hailot".to_string()));
    assert!(state.input_text.is_empty());
    assert_eq!(state.input_history[0], "/bridge send general @hailot");
}

#[test]
fn history_prev_navigates_older() {
    let mut state = AppState::new();
    state.input_history = vec!["cmd3".to_string(), "cmd2".to_string(), "cmd1".to_string()];

    state.history_prev();
    assert_eq!(state.input_text, "cmd3");

    state.history_prev();
    assert_eq!(state.input_text, "cmd2");

    state.history_prev();
    assert_eq!(state.input_text, "cmd1");

    state.history_prev();
    assert_eq!(state.input_text, "cmd1");
}

#[test]
fn history_next_navigates_newer() {
    let mut state = AppState::new();
    state.input_history = vec!["cmd3".to_string(), "cmd2".to_string(), "cmd1".to_string()];
    state.history_idx = Some(2);
    state.input_text = "cmd1".to_string();

    state.history_next();
    assert_eq!(state.input_text, "cmd2");

    state.history_next();
    assert_eq!(state.input_text, "cmd3");

    state.history_next();
    assert!(state.input_text.is_empty());
    assert_eq!(state.history_idx, None);
}

#[test]
fn typing_exits_history_mode() {
    let mut state = AppState::new();
    state.input_history = vec!["old cmd".to_string()];
    state.history_prev();
    assert_eq!(state.history_idx, Some(0));

    state.add_char('x');
    assert_eq!(state.history_idx, None);
}

#[test]
fn empty_submit_returns_none() {
    let mut state = AppState::new();
    let result = state.submit_input();
    assert_eq!(result, None);
    assert!(state.input_history.is_empty());
}
