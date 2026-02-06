use space_cmd::app::{AppState, SidebarTab};
use space_cmd::schema::{Activity, Agent, Spawn};

#[test]
fn tab_switch_toggles_agents_spawns() {
    let mut state = AppState::new();
    assert_eq!(state.active_tab, SidebarTab::Spawns);
    state.switch_tab();
    assert_eq!(state.active_tab, SidebarTab::Agents);
    state.switch_tab();
    assert_eq!(state.active_tab, SidebarTab::Spawns);
}

#[test]
fn sidebar_navigation_wraps_around_agents() {
    let mut state = AppState::new();
    state.active_tab = SidebarTab::Agents;
    state.agents = vec![
        Agent {
            id: "a1".to_string(),
            identity: "alpha".to_string(),
            agent_type: "ai".to_string(),
            model: None,
            constitution: None,
            avatar_path: None,
            color: None,
            created_at: "2026-02-05T10:00:00Z".to_string(),
            archived_at: None,
        },
        Agent {
            id: "a2".to_string(),
            identity: "beta".to_string(),
            agent_type: "ai".to_string(),
            model: None,
            constitution: None,
            avatar_path: None,
            color: None,
            created_at: "2026-02-05T10:01:00Z".to_string(),
            archived_at: None,
        },
    ];

    assert_eq!(state.active_agent_idx, 0);
    state.next_in_sidebar();
    assert_eq!(state.active_agent_idx, 1);
    state.next_in_sidebar();
    assert_eq!(state.active_agent_idx, 0);
}

#[test]
fn pause_toggle_flips_state() {
    let mut state = AppState::new();
    assert!(!state.paused);
    state.toggle_pause();
    assert!(state.paused);
    state.toggle_pause();
    assert!(!state.paused);
}

#[test]
fn all_stream_toggle_resets_activity_scroll() {
    let mut state = AppState::new();
    state.activity_scroll_offset = 10;
    state.toggle_all_stream();
    assert!(state.all_stream);
    assert_eq!(state.activity_scroll_offset, 0);
}

#[test]
fn spawn_expansion_toggle() {
    let mut state = AppState::new();
    state.spawns = vec![Spawn {
        id: "spawn1".to_string(),
        agent_id: "a1".to_string(),
        project_id: None,
        caller_spawn_id: None,
        source: Some("manual".to_string()),
        status: "active".to_string(),
        error: None,
        pid: Some(1234),
        session_id: Some("sess123".to_string()),
        summary: None,
        trace_hash: None,
        created_at: "2026-02-05T10:00:00Z".to_string(),
        last_active_at: None,
    }];

    assert!(!state.expanded_spawns.contains("spawn1"));
    state.toggle_spawn_expansion();
    assert!(state.expanded_spawns.contains("spawn1"));
}

#[test]
fn activity_scroll_respects_bounds() {
    let mut state = AppState::new();
    state.activity = vec![
        Activity {
            id: 1,
            agent_id: "a1".to_string(),
            spawn_id: None,
            primitive: "spawn".to_string(),
            primitive_id: "s1".to_string(),
            action: "created".to_string(),
            field: None,
            after: None,
            created_at: "2026-02-05T10:00:00Z".to_string(),
        },
        Activity {
            id: 2,
            agent_id: "a1".to_string(),
            spawn_id: None,
            primitive: "spawn".to_string(),
            primitive_id: "s1".to_string(),
            action: "started".to_string(),
            field: None,
            after: None,
            created_at: "2026-02-05T10:00:01Z".to_string(),
        },
    ];

    assert_eq!(state.activity_scroll_offset, 0);
    state.scroll_activity_down();
    assert_eq!(state.activity_scroll_offset, 1);
    state.scroll_activity_down();
    assert_eq!(state.activity_scroll_offset, 1);
    state.scroll_activity_up();
    assert_eq!(state.activity_scroll_offset, 0);
    state.scroll_activity_up();
    assert_eq!(state.activity_scroll_offset, 0);
}
