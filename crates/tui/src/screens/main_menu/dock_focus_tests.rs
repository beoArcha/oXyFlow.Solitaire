//! The selection of the main menu while a docked help window holds the focus or does not.

use ratatui::crossterm::event::KeyCode;
use ratatui::style::Color;

use crate::caps::ColorDepth;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, TRUE_COLOR};
use crate::theme::{Theme, ThemeId, Token};

const MARKER: &str = "►";
const FIRST_ITEM: &str = "New game";

#[derive(Debug, PartialEq, Eq)]
struct Shown {
    fg: Color,
    bg: Color,
}

fn docked_help_menu() -> Harness {
    let mut h = Harness::with(MemoryStorage::new(), TRUE_COLOR, 130, 40);
    h.type_char('?');
    h.type_char('d');
    assert!(h.text().contains("Dock: right"));
    h
}

fn first_item_marker(h: &mut Harness) -> Option<(u16, u16)> {
    h.find(&format!("{MARKER} {FIRST_ITEM}"))
}

fn first_item_shown(h: &mut Harness) -> Shown {
    let (column, row) =
        first_item_marker(h).expect("the first item keeps its marker in every state");
    let buffer = h.terminal.backend().buffer();
    let cell = &buffer[(column + 2, row)];
    Shown {
        fg: cell.fg,
        bg: cell.bg,
    }
}

fn theme() -> Theme {
    Theme::new(ThemeId::Dark, ColorDepth::TrueColor)
}

#[test]
fn the_first_item_is_muted_while_the_docked_help_has_the_focus() {
    let mut h = docked_help_menu();
    let theme = theme();
    let menu_focused = Shown {
        fg: theme.color(Token::FocusFg),
        bg: theme.color(Token::FocusBg),
    };
    let menu_muted = Shown {
        fg: theme.color(Token::TextMuted),
        bg: theme.color(Token::Background),
    };

    assert_eq!(first_item_shown(&mut h), menu_focused);

    h.press(KeyCode::F(6));
    assert_eq!(first_item_shown(&mut h), menu_muted);
    assert!(first_item_marker(&mut h).is_some());

    h.press(KeyCode::F(6));
    assert_eq!(first_item_shown(&mut h), menu_focused);
    assert!(first_item_marker(&mut h).is_some());
}

#[test]
fn the_muted_selection_differs_from_the_focused_one_in_every_theme() {
    for id in ThemeId::ALL {
        let theme = Theme::new(id, ColorDepth::TrueColor);
        let focused = (theme.color(Token::FocusFg), theme.color(Token::FocusBg));
        let muted = (
            theme.color(Token::TextMuted),
            theme.color(Token::Background),
        );
        assert_ne!(focused, muted, "{id:?}");
    }
}
