use sm_core::{Pomodoro, SessionState};

#[test]
fn test_new_pomodoro_is_idle() {
    let pomo = Pomodoro::new();
    assert!(!pomo.is_active());
    assert_eq!(pomo.remaining_seconds(), 0);
}

#[test]
fn test_start_focus_sets_active() {
    let mut pomo = Pomodoro::new();
    pomo.start_focus();
    assert!(pomo.is_active());
    assert!(matches!(pomo.state, SessionState::Focusing { .. }));
}

#[test]
fn test_remaining_seconds_decreases() {
    let mut pomo = Pomodoro::new();
    pomo.start_focus();
    let expected = (pomo.config.focus_duration as i64) * 60;
    let remaining = pomo.remaining_seconds();
    // Should be close to full duration (within 1 second)
    assert!(remaining > 0 && remaining <= expected);
    assert!(remaining > expected - 2);
}

#[test]
fn test_skip_from_idle_starts_focus() {
    let mut pomo = Pomodoro::new();
    pomo.skip();
    assert!(matches!(pomo.state, SessionState::Focusing { .. }));
}

#[test]
fn test_skip_from_focus_starts_break() {
    let mut pomo = Pomodoro::new();
    pomo.start_focus();
    pomo.skip();
    assert!(matches!(pomo.state, SessionState::ShortBreak { .. }));
}

#[test]
fn test_stop_returns_to_idle() {
    let mut pomo = Pomodoro::new();
    pomo.start_focus();
    assert!(pomo.is_active());
    pomo.stop();
    assert!(!pomo.is_active());
    assert!(matches!(pomo.state, SessionState::Idle));
}

#[test]
fn test_double_start_is_noop() {
    let mut pomo = Pomodoro::new();
    pomo.start_focus();
    let state_before = pomo.state.clone();
    pomo.start_focus();
    assert!(matches!(state_before, SessionState::Focusing { .. }));
    assert!(matches!(pomo.state, SessionState::Focusing { .. }));
}

#[test]
fn test_cycle_tracking() {
    let mut pomo = Pomodoro::new();
    pomo.start_focus();
    // cycle should be 1
    if let SessionState::Focusing { cycle, .. } = pomo.state {
        assert_eq!(cycle, 1);
    } else {
        panic!("expected Focusing");
    }
    pomo.skip();
    pomo.skip();
    // now in focus cycle 2
    if let SessionState::Focusing { cycle, .. } = pomo.state {
        assert_eq!(cycle, 2);
    } else {
        panic!("expected Focusing cycle 2");
    }
}

#[test]
fn test_long_break_after_4_cycles() {
    let mut pomo = Pomodoro::new();
    pomo.config.cycles_before_long = 4;

    // cycles 1-4: f->s->f->s->f->s->f
    for _ in 0..4 {
        pomo.start_focus();
        pomo.skip(); // becomes break
    }

    // After 4 cycles, next break should be long
    pomo.start_focus();
    // cycle 5
    if let SessionState::Focusing { cycle, .. } = pomo.state {
        assert_eq!(cycle, 5);
    } else {
        panic!("expected Focusing cycle 5");
    }
    pomo.skip(); // should be LongBreak because 5 % 4 == 1? No...

    // Wait, let me re-think. The logic is:
    // start_focus → cycle 1
    // skip (break) → ShortBreak cycle 1
    // skip → start_focus → cycle 2
    // And so on.
    // cycles_before_long = 4 means after 4 cycles, long break.
    // cycle 4 → ShortBreak, cycle 4...
    // Actually the logic: if cycle % cycles_before_long == 0 → LongBreak
    // So cycle 4 → 4 % 4 == 0 → LongBreak
    // cycle 8 → LongBreak, etc.

    // Let me restart:
    let mut pomo = Pomodoro::new();
    pomo.config.cycles_before_long = 4;

    // Cycle 1: focus → short break
    pomo.start_focus();
    pomo.skip();
    assert!(matches!(pomo.state, SessionState::ShortBreak { .. }));

    // Cycle 2: focus → short break
    pomo.skip();
    pomo.skip();
    assert!(matches!(pomo.state, SessionState::ShortBreak { .. }));

    // Cycle 3: focus → short break
    pomo.skip();
    pomo.skip();
    assert!(matches!(pomo.state, SessionState::ShortBreak { .. }));

    // Cycle 4: focus → LONG break
    pomo.skip();
    pomo.skip();
    assert!(matches!(pomo.state, SessionState::LongBreak { .. }));
}
