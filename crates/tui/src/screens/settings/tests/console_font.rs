//! The Console font row and the Symbols row that offers the oXySol font.

use ratatui::crossterm::event::KeyCode;

use super::{index_of, move_down, open_settings, row};
use crate::app::{CardFontState, FontFacts};
use crate::screens::settings::options::{FIELDS, Field};
use crate::screens::settings::values::line;
use crate::settings::{Settings, TerminalKind};
use crate::test_support::sound_styles;

fn row_with(settings: &Settings, card_font: CardFontState, terminal: TerminalKind) -> String {
    let facts = FontFacts {
        terminal,
        card_font,
        cell_px: None,
    };
    line(
        Field::ConsoleFont,
        settings,
        true,
        &sound_styles(),
        (200, facts),
        None,
    )
}

fn on() -> Settings {
    Settings {
        console_font: true,
        ..Settings::default()
    }
}

#[test]
fn the_row_exists_in_the_display_group_after_the_symbols_row() {
    assert_eq!(index_of(Field::ConsoleFont), index_of(Field::Glyphs) + 1);
    assert!(FIELDS.contains(&Field::ConsoleFont));
}

#[test]
fn the_value_says_why_the_option_does_not_work() {
    let classic = TerminalKind::ClassicConsole;
    let off = Settings::default();
    assert!(row_with(&off, CardFontState::Off, classic).ends_with("Off"));
    assert!(row_with(&on(), CardFontState::Active, classic).ends_with("On"));
    assert!(row_with(&on(), CardFontState::NotRegistered, classic).ends_with("On (not installed)"));
    assert!(row_with(&on(), CardFontState::Refused, classic).ends_with("On (refused)"));
    assert!(
        row_with(
            &off,
            CardFontState::Unavailable,
            TerminalKind::WindowsTerminal
        )
        .ends_with("unavailable")
    );
}

#[test]
fn a_failure_is_not_shown_while_the_option_is_off() {
    let text = row_with(
        &Settings::default(),
        CardFontState::NotRegistered,
        TerminalKind::ClassicConsole,
    );
    assert!(text.ends_with("Off"), "{text}");
}

#[test]
fn a_terminal_without_the_console_font_ignores_the_row() {
    let mut h = open_settings();
    move_down(&mut h, index_of(Field::ConsoleFont));
    let before = h.app.settings().clone();
    h.press(KeyCode::Enter);
    h.press(KeyCode::Right);
    assert_eq!(h.app.settings(), &before);
    assert!(row(Field::ConsoleFont, &before, true).ends_with("unavailable"));
}

#[test]
fn the_description_of_the_row_follows_the_terminal_and_the_state() {
    let text = |terminal, card_font| {
        Field::ConsoleFont.describe(FontFacts {
            terminal,
            card_font,
            cell_px: None,
        })
    };
    let wt = text(TerminalKind::WindowsTerminal, CardFontState::Unavailable);
    assert!(wt.contains("oXySol Mono"), "{wt}");
    let other = text(TerminalKind::Linux, CardFontState::Unavailable);
    assert!(!other.contains("oXySol Mono"), "{other}");
    let missing = text(TerminalKind::ClassicConsole, CardFontState::NotRegistered);
    let refused = text(TerminalKind::ClassicConsole, CardFontState::Refused);
    let plain = text(TerminalKind::ClassicConsole, CardFontState::Off);
    let all = [&wt, &other, &missing, &refused, &plain];
    for (index, one) in all.iter().enumerate() {
        assert!(!one.starts_with("settings."), "{one}");
        assert!(all[index + 1..].iter().all(|two| two != one));
    }
}
