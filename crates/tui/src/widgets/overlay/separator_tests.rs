//! Every Help tab draws one separator row between its scrolling text and its footer, in Unicode and
//! in ASCII mode, at both sizes: the row is a full rule, the text sits above it and the footer
//! below it.

use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;

use super::help;
use crate::caps::TerminalCaps;
use crate::i18n::{self, Locale};
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, PLAIN, TRUE_COLOR};

const SIZES: [(u16, u16); 2] = [(100, 30), (200, 60)];
const MODES: [(&str, TerminalCaps, char); 2] =
    [("unicode", TRUE_COLOR, '─'), ("ascii", PLAIN, '-')];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tab {
    Keys,
    Rules,
    CommandLine,
}

const TABS: [Tab; 3] = [Tab::Keys, Tab::Rules, Tab::CommandLine];

fn open(tab: Tab, caps: TerminalCaps, (width, height): (u16, u16)) -> Harness {
    let mut h = Harness::with(MemoryStorage::new(), caps, width, height);
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

fn inner_cells(line: &str, frame: Rect) -> Vec<char> {
    line.chars()
        .skip(usize::from(frame.x) + 1)
        .take(usize::from(frame.width).saturating_sub(2))
        .collect()
}

fn rule_rows(lines: &[String], frame: Rect, rule: char) -> Vec<usize> {
    (usize::from(frame.y) + 1..usize::from(frame.bottom()).saturating_sub(1))
        .filter(|&y| {
            lines.get(y).is_some_and(|line| {
                let cells = inner_cells(line, frame);
                !cells.is_empty() && cells.iter().all(|&ch| ch == rule)
            })
        })
        .collect()
}

#[test]
fn every_help_tab_has_one_separator_row_above_the_footer() {
    for (mode, caps, rule) in MODES {
        for size in SIZES {
            for tab in TABS {
                let mut h = open(tab, caps, size);
                let lines = h.lines();
                let frame = help::geometry(Rect::new(0, 0, size.0, size.1)).frame;
                let rows = rule_rows(&lines, frame, rule);
                let context = format!("{tab:?} {mode} {size:?}");
                let [row] = rows[..] else {
                    panic!("{context}: expected one separator row, found {rows:?}");
                };
                assert!(
                    row > usize::from(frame.y) + 2,
                    "{context}: the rule sits on the top tab row"
                );
                let footer = lines.get(row + 1).map(|line| inner_cells(line, frame));
                assert!(
                    footer.is_some_and(|cells| cells.iter().any(|ch| !ch.is_whitespace())),
                    "{context}: no footer below the separator"
                );
            }
        }
    }
}

#[test]
fn the_separator_is_at_the_same_row_on_every_tab() {
    for (_, caps, rule) in MODES {
        for size in SIZES {
            let frame = help::geometry(Rect::new(0, 0, size.0, size.1)).frame;
            let rows: Vec<Vec<usize>> = TABS
                .into_iter()
                .map(|tab| rule_rows(&open(tab, caps, size).lines(), frame, rule))
                .collect();
            assert!(
                rows.iter().all(|found| found == &rows[0]),
                "{size:?}: {rows:?}"
            );
        }
    }
}
