//! The Help window has one size on every tab: the same rectangle on Keys, Rules and Command line at
//! each size, a height that is a share of the terminal, and no logo cell above its border.

use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;

use crate::app::Modal;
use crate::i18n::{self, Locale};
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, TRUE_COLOR};
use crate::widgets::logo::logo;

const SIZES: [(u16, u16); 2] = [(100, 30), (200, 60)];
const LOGO_GLYPHS: [char; 3] = ['█', '▀', '▄'];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tab {
    Keys,
    Rules,
    CommandLine,
}

const TABS: [Tab; 3] = [Tab::Keys, Tab::Rules, Tab::CommandLine];

fn open(tab: Tab, (width, height): (u16, u16)) -> Harness {
    let mut h = Harness::with(MemoryStorage::new(), TRUE_COLOR, width, height);
    i18n::set_locale(Locale::EnUs);
    match tab {
        Tab::Keys => h.type_char('?'),
        Tab::CommandLine => {
            h.type_char('?');
            h.press(KeyCode::Left);
        }
        Tab::Rules => h.type_char('i'),
    }
    h.lines();
    h
}

fn frame_of(lines: &[String]) -> Rect {
    let top_left = lines
        .iter()
        .enumerate()
        .find_map(|(y, line)| line.chars().position(|ch| ch == '╔').map(|x| (x, y)));
    let bottom_right = lines.iter().enumerate().rev().find_map(|(y, line)| {
        line.chars()
            .enumerate()
            .filter(|(_, ch)| *ch == '╝')
            .last()
            .map(|(x, _)| (x, y))
    });
    let (Some((x, y)), Some((right, bottom))) = (top_left, bottom_right) else {
        panic!("no double border is drawn");
    };
    Rect::new(
        u16::try_from(x).unwrap_or(u16::MAX),
        u16::try_from(y).unwrap_or(u16::MAX),
        u16::try_from(right - x + 1).unwrap_or(u16::MAX),
        u16::try_from(bottom - y + 1).unwrap_or(u16::MAX),
    )
}

fn logo_top(lines: &[String]) -> Option<usize> {
    let first = logo(true)[0].trim();
    lines.iter().position(|line| line.contains(first))
}

#[test]
fn every_help_tab_draws_the_window_at_the_same_rectangle_at_both_sizes() {
    for size in SIZES {
        let frames: Vec<(Tab, Rect)> = TABS
            .into_iter()
            .map(|tab| (tab, frame_of(&open(tab, size).lines())))
            .collect();
        let first = frames[0].1;
        for (tab, frame) in &frames {
            assert_eq!(*frame, first, "{tab:?} at {size:?}");
        }
    }
    i18n::set_locale(Locale::EnUs);
}

#[test]
fn the_window_height_is_a_share_of_the_terminal_height() {
    assert_eq!(frame_of(&open(Tab::Keys, (100, 30)).lines()).height, 28);
    assert_eq!(frame_of(&open(Tab::Keys, (200, 60)).lines()).height, 36);
}

#[test]
fn switching_between_keys_and_command_line_keeps_the_rectangle() {
    for size in SIZES {
        let mut h = open(Tab::Keys, size);
        let before = frame_of(&h.lines());
        h.press(KeyCode::Left);
        assert!(matches!(h.app.modal(), Some(Modal::CommandLine(_))));
        assert_eq!(frame_of(&h.lines()), before, "{size:?}");
    }
}

#[test]
fn no_logo_cell_is_drawn_above_the_window_border() {
    for size in SIZES {
        for tab in TABS {
            let mut h = open(tab, size);
            let lines = h.lines();
            let frame = frame_of(&lines);
            for line in lines.iter().take(usize::from(frame.y)) {
                let above: String = line
                    .chars()
                    .skip(usize::from(frame.x))
                    .take(usize::from(frame.width))
                    .collect();
                assert!(
                    !above.chars().any(|ch| LOGO_GLYPHS.contains(&ch)),
                    "{tab:?} at {size:?}: logo above the border in {above:?}"
                );
            }
        }
    }
}

#[test]
fn the_logo_is_on_the_menu_so_the_check_above_the_border_means_something() {
    for size in SIZES {
        let mut h = Harness::with(MemoryStorage::new(), TRUE_COLOR, size.0, size.1);
        i18n::set_locale(Locale::EnUs);
        assert!(
            logo_top(&h.lines()).is_some(),
            "{size:?}: no logo on the menu"
        );
    }
}

#[test]
fn the_logo_stays_under_the_border_at_every_height_from_30_to_60() {
    for height in 30..=60 {
        let mut menu = Harness::with(MemoryStorage::new(), TRUE_COLOR, 100, height);
        let logo_row = logo_top(&menu.lines()).unwrap_or(0);
        let frame = frame_of(&open(Tab::Keys, (100, height)).lines());
        assert!(
            usize::from(frame.y) <= logo_row,
            "height {height}: window top {} is below the logo row {logo_row}",
            frame.y
        );
    }
}

#[test]
fn the_rules_text_shows_the_rows_of_the_taller_window_at_200_by_60() {
    let mut h = open(Tab::Rules, (200, 60));
    assert!(h.text().contains("1-28 / "), "{}", h.text());
}
