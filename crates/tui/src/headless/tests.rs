use std::time::Instant;

use oxysol_core::{DrawCount, Variant};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{
    KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::style::Modifier;

use super::{Headless, HeadlessConfig};
use crate::app::Zone;
use crate::clock::Clock;
use crate::input::Input;
use crate::screens::Effect;
use crate::settings::Settings;
use crate::test_support::{Harness, TEST_VERSION, TRUE_COLOR, sound_styles};
use crate::{ScreenKind, UiSnapshot};

fn key(code: KeyCode) -> Input {
    Input::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

fn char_key(c: char) -> Input {
    key(KeyCode::Char(c))
}

fn fixed_config(base: Instant) -> HeadlessConfig {
    HeadlessConfig {
        cols: 120,
        rows: 40,
        caps: TRUE_COLOR,
        settings: None,
        start_seed: None,
        seed_start: 42,
        sound_styles: sound_styles(),
        version: TEST_VERSION,
        zone: Zone::Utc,
        base_time: Some(base),
    }
}

fn assert_buffers_equal(left: &Buffer, right: &Buffer) {
    assert_eq!(left.area, right.area);
    for y in 0..left.area.height {
        for x in 0..left.area.width {
            let a = &left[(x, y)];
            let b = &right[(x, y)];
            assert_eq!(a.symbol(), b.symbol(), "symbol mismatch at ({x}, {y})");
            assert_eq!(a.fg, b.fg, "fg mismatch at ({x}, {y})");
            assert_eq!(a.bg, b.bg, "bg mismatch at ({x}, {y})");
            let mask = Modifier::BOLD | Modifier::UNDERLINED | Modifier::REVERSED | Modifier::DIM;
            assert_eq!(
                a.modifier & mask,
                b.modifier & mask,
                "modifier mismatch at ({x}, {y})"
            );
        }
    }
}

#[test]
fn two_headless_runs_with_same_inputs_give_identical_buffers_and_snapshots() {
    let base = Instant::now();
    let mut first = Headless::new(fixed_config(base)).unwrap();
    let mut second = Headless::new(fixed_config(base)).unwrap();

    let script = [
        char_key('?'),
        key(KeyCode::Esc),
        key(KeyCode::Enter),
        key(KeyCode::Enter),
        char_key(' '),
        key(KeyCode::Right),
    ];

    for input in script {
        first.step(input).unwrap();
        second.step(input).unwrap();
        assert_eq!(first.frame_id(), second.frame_id());
        assert_buffers_equal(first.buffer(), second.buffer());
        assert_eq!(first.text_frame(), second.text_frame());
        assert_eq!(first.snapshot(), second.snapshot());
    }

    first.tick(1_500).unwrap();
    second.tick(1_500).unwrap();
    assert_buffers_equal(first.buffer(), second.buffer());
    assert_eq!(first.snapshot(), second.snapshot());
    assert_eq!(
        toml::to_string(&first.snapshot()).unwrap(),
        toml::to_string(&second.snapshot()).unwrap()
    );
}

#[test]
fn headless_frame_equals_harness_frame_for_same_inputs() {
    let mut harness = Harness::new();
    let mut headless = Headless::new(fixed_config(harness.now)).unwrap();

    assert_eq!(headless.text_frame(), harness.text());

    let inputs = [
        key(KeyCode::Down),
        char_key('?'),
        key(KeyCode::Esc),
        key(KeyCode::Enter),
        key(KeyCode::Enter),
        key(KeyCode::Right),
        char_key('a'),
    ];

    for input in inputs {
        harness.send(input);
        headless.step(input).unwrap();
        assert_eq!(headless.text_frame(), harness.text());
    }

    harness.advance(3);
    headless.tick(3_000).unwrap();
    assert_eq!(headless.text_frame(), harness.text());

    harness.resize(100, 30);
    headless.resize(100, 30).unwrap();
    assert_eq!(headless.text_frame(), harness.text());
}

#[test]
fn snapshot_is_read_only_and_reports_screen_overlay_focus_hover_and_game_stats() {
    let base = Instant::now();
    let mut headless = Headless::new(HeadlessConfig {
        cols: 140,
        rows: 40,
        ..fixed_config(base)
    })
    .unwrap();

    let snap0: UiSnapshot = headless.snapshot();
    assert_eq!(snap0, headless.snapshot());
    assert_eq!(snap0.screen, ScreenKind::MainMenu);
    assert_eq!(snap0.active_overlay, None);
    assert_eq!(snap0.overlay_rect, None);
    assert_eq!(snap0.game_stats, None);
    assert_eq!(snap0.legal_actions, None);

    headless.step(key(KeyCode::BackTab)).unwrap();
    let snap_picker = headless.snapshot();
    assert!(snap_picker.tooltip_visible);
    assert_eq!(
        snap_picker.focus.as_ref().map(|f| f.target.as_str()),
        Some("language_selector")
    );

    headless
        .step(Input::Mouse(MouseEvent {
            kind: MouseEventKind::Moved,
            column: 12,
            row: 6,
            modifiers: KeyModifiers::NONE,
        }))
        .unwrap();
    assert_eq!(
        headless.snapshot().hover.map(|pos| (pos.x, pos.y)),
        Some((12, 6))
    );

    headless.step(char_key('?')).unwrap();
    let snap_help = headless.snapshot();
    assert!(snap_help.active_overlay.is_some());
    assert!(snap_help.overlay_rect.is_some());
    assert!(snap_help.dock_mode.is_some());
    assert!(!snap_help.tooltip_visible);

    headless.step(key(KeyCode::Esc)).unwrap();
    headless.app_mut().apply(Effect::Start {
        variant: Variant::Klondike(DrawCount::One),
        seed: Some(123),
        undo_allowed: true,
    });
    headless.tick(2_000).unwrap();

    let snap_board = headless.snapshot();
    assert_eq!(snap_board.screen, ScreenKind::Board);
    let stats = snap_board.game_stats.expect("board must have game_stats");
    assert_eq!(stats.variant, Variant::Klondike(DrawCount::One));
    assert_eq!(stats.seed, 123);
    assert_eq!(stats.elapsed_ms, 2_000);
    assert_eq!(stats.moves, 0);
    assert!(!stats.won);
    assert!(
        snap_board
            .legal_actions
            .as_ref()
            .is_some_and(|actions| !actions.is_empty())
    );
    assert!(snap_board.status_line.is_some());

    headless.step(char_key('u')).unwrap();
    let snap_refusal = headless.snapshot();
    assert!(snap_refusal.last_error.is_some());

    headless.step(char_key('c')).unwrap();
    assert_eq!(headless.clipboard_copies(), vec!["123".to_owned()]);
    let _ = MouseButton::Left;
}

#[test]
fn headless_dependencies_are_single_threaded_and_deterministic() {
    let base = Instant::now();
    let mut settings = Settings::for_caps(&TRUE_COLOR);
    settings.winnable_only = true;
    let mut headless = Headless::new(HeadlessConfig {
        settings: Some(settings),
        seed_start: 100,
        ..fixed_config(base)
    })
    .unwrap();

    headless.step(key(KeyCode::Enter)).unwrap();
    headless.step(key(KeyCode::Enter)).unwrap();
    let snap = headless.snapshot();
    assert_eq!(snap.screen, ScreenKind::Board);
    let stats = snap
        .game_stats
        .expect("winnable search resolves synchronously");
    assert!(stats.seed >= 100);
    assert_eq!(headless.clock().now(), base);
}
