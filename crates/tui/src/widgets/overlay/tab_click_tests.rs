//! A left click on a top tab of the help window switches to that tab, from every other tab, opened
//! from the main menu and from the board. The window is centred at every size, including sizes
//! whose height is odd, where the Rules window is one row shorter than the Keys window; a docked
//! window shows one tab between arrows, and a click on either arrow moves to the neighbour.

use ratatui::crossterm::event::KeyCode;

use super::tabs::HelpTab;
use crate::app::Modal;
use crate::i18n::t;
use crate::settings::{DockSide, Settings};
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, TRUE_COLOR};

const CENTRED_SIZES: [(u16, u16); 5] = [(100, 30), (100, 31), (120, 41), (130, 40), (130, 41)];
const DOCKABLE: (u16, u16) = (130, 40);

fn harness(size: (u16, u16), side: DockSide) -> Harness {
    let mut storage = MemoryStorage::new();
    storage.settings = Some(Settings {
        help_dock: side,
        ..Settings::default()
    });
    Harness::with(storage, TRUE_COLOR, size.0, size.1)
}

fn on_board(h: &mut Harness) {
    h.press(KeyCode::Enter);
    h.press(KeyCode::Enter);
    h.lines();
}

fn open(h: &mut Harness, tab: HelpTab) {
    match tab {
        HelpTab::Keys => h.type_char('?'),
        HelpTab::Rules => h.type_char('i'),
        HelpTab::CommandLine => {
            h.type_char('?');
            h.press(KeyCode::BackTab);
        }
    }
    assert_eq!(current(h), Some(tab), "{tab:?} did not open");
}

fn current(h: &Harness) -> Option<HelpTab> {
    match h.app.modal() {
        Some(Modal::Help(_)) => Some(HelpTab::Keys),
        Some(Modal::HowToPlay(_)) => Some(HelpTab::Rules),
        Some(Modal::CommandLine(_)) => Some(HelpTab::CommandLine),
        _ => None,
    }
}

fn cell_of(line: &str, needle: &str) -> Option<u16> {
    let byte = line.find(needle)?;
    u16::try_from(line[..byte].chars().count()).ok()
}

fn click_name(h: &mut Harness, from: HelpTab, to: HelpTab) {
    let active = format!("[{}]", t(from.label_key()));
    let name = t(to.label_key());
    let lines = h.lines();
    let row = lines
        .iter()
        .position(|line| line.contains(&active))
        .unwrap_or_else(|| panic!("{active} is not drawn"));
    let column = cell_of(&lines[row], &name)
        .unwrap_or_else(|| panic!("{name} is not drawn on the tab row of {from:?}"));
    h.click(column, u16::try_from(row).unwrap_or(u16::MAX));
}

fn click_docked_arrow(h: &mut Harness, from: HelpTab, next: bool) {
    let name = t(from.label_key());
    let needle = format!("◄ {name} ►");
    let lines = h.lines();
    let row = lines
        .iter()
        .position(|line| line.contains(&needle))
        .unwrap_or_else(|| panic!("{needle} is not drawn"));
    let start = cell_of(&lines[row], &needle).unwrap_or_default();
    let column = if next {
        start + u16::try_from(name.chars().count() + 3).unwrap_or(u16::MAX)
    } else {
        start
    };
    h.click(column, u16::try_from(row).unwrap_or(u16::MAX));
}

fn every_tab_reaches_every_other(size: (u16, u16), side: DockSide, from_board: bool) {
    for from in HelpTab::ALL {
        for to in HelpTab::ALL.into_iter().filter(|tab| *tab != from) {
            let mut h = harness(size, side);
            if from_board {
                on_board(&mut h);
            }
            open(&mut h, from);
            click_name(&mut h, from, to);
            assert_eq!(
                current(&h),
                Some(to),
                "click {to:?} from {from:?}, board={from_board}, dock={side:?}, size={size:?}"
            );
        }
    }
}

#[test]
fn clicking_each_top_tab_switches_from_the_menu_with_the_window_centred() {
    for size in CENTRED_SIZES {
        every_tab_reaches_every_other(size, DockSide::Centre, false);
    }
}

#[test]
fn clicking_each_top_tab_switches_from_the_board_with_the_window_centred() {
    for size in CENTRED_SIZES {
        every_tab_reaches_every_other(size, DockSide::Centre, true);
    }
}

#[test]
fn clicking_an_arrow_of_the_docked_tab_row_moves_through_all_three_tabs() {
    for side in [DockSide::Left, DockSide::Right] {
        for from_board in [false, true] {
            let mut h = harness(DOCKABLE, side);
            if from_board {
                on_board(&mut h);
            }
            open(&mut h, HelpTab::Keys);
            for (next, expected) in [
                (true, HelpTab::Rules),
                (true, HelpTab::CommandLine),
                (false, HelpTab::Rules),
                (false, HelpTab::Keys),
            ] {
                let from = current(&h).unwrap_or(HelpTab::Keys);
                click_docked_arrow(&mut h, from, next);
                assert_eq!(
                    current(&h),
                    Some(expected),
                    "{side:?} board={from_board} next={next}"
                );
            }
        }
    }
}
