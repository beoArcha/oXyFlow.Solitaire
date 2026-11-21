//! The main menu footer keeps an empty column between its text and a docked help window.

use ratatui::layout::Rect;

use crate::i18n::tf;
use crate::settings::DockSide;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, TEST_VERSION, TRUE_COLOR};
use crate::widgets::overlay::dock;

const SCREEN: Rect = Rect::new(0, 0, 130, 40);

fn docked_menu(side: DockSide) -> Harness {
    let mut h = Harness::with(MemoryStorage::new(), TRUE_COLOR, 130, 40);
    h.type_char('?');
    let presses = match side {
        DockSide::Centre => 0,
        DockSide::Right => 1,
        DockSide::Left => 2,
    };
    for _ in 0..presses {
        h.type_char('d');
    }
    assert_eq!(h.app.settings().help_dock, side);
    h
}

fn assert_empty_column_beside_the_panel(side: DockSide) {
    let mut h = docked_menu(side);
    let version = tf("menu.version", &[("version", TEST_VERSION)]);
    let (column, row) = h
        .find(&version)
        .expect("the version label is on the main menu");
    let (_, main) = dock::split(SCREEN, side).expect("the help window docks at 130 by 40");
    let edge = if side == DockSide::Left {
        main.x
    } else {
        main.right() - 1
    };
    let line: Vec<char> = h.lines()[usize::from(row)].chars().collect();
    assert_eq!(line[usize::from(edge)], ' ', "{side:?}: {line:?}");
    let label_end = column + u16::try_from(version.chars().count()).unwrap_or(u16::MAX);
    assert!(
        !(column..label_end).contains(&edge),
        "{side:?}: the version label reaches the panel"
    );
}

#[test]
fn the_version_label_keeps_an_empty_column_from_the_docked_help_on_the_right() {
    assert_empty_column_beside_the_panel(DockSide::Right);
}

#[test]
fn the_footer_keeps_an_empty_column_from_the_docked_help_on_the_left() {
    assert_empty_column_beside_the_panel(DockSide::Left);
}
