use space_cmd::app::{AppState, SidebarTab};
use space_cmd::schema::{Channel, Message, Spawn};

#[test]
fn focus_channel_marks_read() {
    let mut state = AppState::new();
    state.channels = vec![Channel {
        channel_id: "ch1".to_string(),
        name: "general".to_string(),
        topic: None,
        created_at: "2025-11-05T10:00:00Z".to_string(),
        pinned_at: None,
    }];
    state.messages = vec![Message {
        message_id: "m1".to_string(),
        channel_id: "ch1".to_string(),
        agent_id: "hailot".to_string(),
        content: "hello".to_string(),
        created_at: "2025-11-05T10:00:00Z".to_string(),
    }];

    assert!(state.is_channel_unread("ch1"));
    state.mark_channel_read();
    assert!(!state.is_channel_unread("ch1"));
}

#[test]
fn sidebar_navigation_wraps_around() {
    let mut state = AppState::new();
    state.channels = vec![
        Channel {
            channel_id: "ch1".to_string(),
            name: "general".to_string(),
            topic: None,
            created_at: "2025-11-05T10:00:00Z".to_string(),
            pinned_at: None,
        },
        Channel {
            channel_id: "ch2".to_string(),
            name: "tasks".to_string(),
            topic: None,
            created_at: "2025-11-05T10:01:00Z".to_string(),
            pinned_at: None,
        },
    ];

    assert_eq!(state.active_channel_idx, 0);
    state.next_in_sidebar();
    assert_eq!(state.active_channel_idx, 1);
    state.next_in_sidebar();
    assert_eq!(state.active_channel_idx, 0);
}

#[test]
fn tab_switch_toggles_channels_spawns() {
    let mut state = AppState::new();
    assert_eq!(state.active_tab, SidebarTab::Channels);
    state.switch_tab();
    assert_eq!(state.active_tab, SidebarTab::Spawns);
    state.switch_tab();
    assert_eq!(state.active_tab, SidebarTab::Channels);
}

#[test]
fn spawn_expansion_toggle() {
    let mut state = AppState::new();
    state.spawns = vec![Spawn {
        id: "spawn1".to_string(),
        agent_id: "hailot".to_string(),
        session_id: Some("sess123".to_string()),
        channel_id: None,
        constitution_hash: None,
        is_task: false,
        status: "running".to_string(),
        pid: Some(1234),
        created_at: "2025-11-05T10:00:00Z".to_string(),
        ended_at: None,
    }];
    state.active_tab = SidebarTab::Spawns;

    assert!(!state.expanded_spawns.contains("spawn1"));
    state.toggle_spawn_expansion();
    assert!(state.expanded_spawns.contains("spawn1"));
}

#[test]
fn message_scroll_respects_bounds() {
    let mut state = AppState::new();
    state.messages = vec![
        Message {
            message_id: "m1".to_string(),
            channel_id: "ch1".to_string(),
            agent_id: "hailot".to_string(),
            content: "msg1".to_string(),
            created_at: "2025-11-05T10:00:00Z".to_string(),
        },
        Message {
            message_id: "m2".to_string(),
            channel_id: "ch1".to_string(),
            agent_id: "hailot".to_string(),
            content: "msg2".to_string(),
            created_at: "2025-11-05T10:01:00Z".to_string(),
        },
        Message {
            message_id: "m3".to_string(),
            channel_id: "ch1".to_string(),
            agent_id: "hailot".to_string(),
            content: "msg3".to_string(),
            created_at: "2025-11-05T10:02:00Z".to_string(),
        },
    ];

    assert_eq!(state.message_scroll_offset, 0);
    state.scroll_messages_down();
    assert_eq!(state.message_scroll_offset, 1);
    state.scroll_messages_down();
    assert_eq!(state.message_scroll_offset, 2);
    state.scroll_messages_down();
    assert_eq!(state.message_scroll_offset, 2);

    state.scroll_messages_up();
    assert_eq!(state.message_scroll_offset, 1);
    state.scroll_messages_up();
    assert_eq!(state.message_scroll_offset, 0);
    state.scroll_messages_up();
    assert_eq!(state.message_scroll_offset, 0);
}
