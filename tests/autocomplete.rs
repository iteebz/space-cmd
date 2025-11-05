use space_cmd::app::{AppState, AutocompleteMode};

#[test]
fn agent_trigger_at_word_start() {
    let mut state = AppState::new();
    state.input_text = "@hail".to_string();
    state.detect_and_trigger_autocomplete();

    assert_eq!(state.autocomplete_mode, Some(AutocompleteMode::Agent));
    assert_eq!(state.autocomplete_query, "hail");
}

#[test]
fn agent_trigger_after_space() {
    let mut state = AppState::new();
    state.input_text = "cmd @sen".to_string();
    state.detect_and_trigger_autocomplete();

    assert_eq!(state.autocomplete_mode, Some(AutocompleteMode::Agent));
    assert_eq!(state.autocomplete_query, "sen");
}

#[test]
fn file_trigger_requires_character_after_slash() {
    let mut state = AppState::new();
    state.input_text = "/".to_string();
    state.detect_and_trigger_autocomplete();

    assert_eq!(state.autocomplete_mode, None);
}

#[test]
fn file_trigger_with_character() {
    let mut state = AppState::new();
    state.input_text = "/src".to_string();
    state.detect_and_trigger_autocomplete();

    assert_eq!(state.autocomplete_mode, Some(AutocompleteMode::File));
    assert_eq!(state.autocomplete_query, "src");
}

#[test]
fn filter_matches_substring() {
    let mut state = AppState::new();
    state.autocomplete_list = vec![
        "hailot".to_string(),
        "sentinel".to_string(),
        "zealot".to_string(),
    ];
    state.autocomplete_query = "hail".to_string();

    state.filter_autocomplete();

    assert_eq!(state.autocomplete_list, vec!["hailot".to_string()]);
}

#[test]
fn filter_is_case_insensitive() {
    let mut state = AppState::new();
    state.autocomplete_list = vec!["Hailot".to_string(), "Sentinel".to_string()];
    state.autocomplete_query = "hail".to_string();

    state.filter_autocomplete();

    assert_eq!(state.autocomplete_list, vec!["Hailot".to_string()]);
}

#[test]
fn navigation_cycles_next() {
    let mut state = AppState::new();
    state.autocomplete_list = vec!["hailot".to_string(), "sentinel".to_string()];

    state.autocomplete_next();
    assert_eq!(state.autocomplete_idx, 1);

    state.autocomplete_next();
    assert_eq!(state.autocomplete_idx, 0);
}

#[test]
fn navigation_cycles_prev() {
    let mut state = AppState::new();
    state.autocomplete_list = vec!["hailot".to_string(), "sentinel".to_string()];
    state.autocomplete_idx = 0;

    state.autocomplete_prev();
    assert_eq!(state.autocomplete_idx, 1);

    state.autocomplete_prev();
    assert_eq!(state.autocomplete_idx, 0);
}

#[test]
fn select_agent_inserts_and_clears() {
    let mut state = AppState::new();
    state.input_text = "cmd @hai".to_string();
    state.autocomplete_mode = Some(AutocompleteMode::Agent);
    state.autocomplete_list = vec!["hailot".to_string()];
    state.autocomplete_idx = 0;

    state.autocomplete_select();

    assert_eq!(state.input_text, "cmd @hailot ");
    assert_eq!(state.autocomplete_mode, None);
    assert!(state.autocomplete_list.is_empty());
}

#[test]
fn select_file_inserts_and_clears() {
    let mut state = AppState::new();
    state.input_text = "edit /sr".to_string();
    state.autocomplete_mode = Some(AutocompleteMode::File);
    state.autocomplete_list = vec!["src".to_string()];
    state.autocomplete_idx = 0;

    state.autocomplete_select();

    assert_eq!(state.input_text, "edit /src ");
    assert_eq!(state.autocomplete_mode, None);
}

#[test]
fn cancel_clears_all_state() {
    let mut state = AppState::new();
    state.autocomplete_mode = Some(AutocompleteMode::Agent);
    state.autocomplete_list = vec!["hailot".to_string()];
    state.autocomplete_idx = 1;
    state.autocomplete_query = "hai".to_string();

    state.cancel_autocomplete();

    assert_eq!(state.autocomplete_mode, None);
    assert!(state.autocomplete_list.is_empty());
    assert_eq!(state.autocomplete_idx, 0);
    assert!(state.autocomplete_query.is_empty());
}
