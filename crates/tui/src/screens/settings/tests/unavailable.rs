//! Unavailable rows: no arrows, the dimmed style when they are not focused, and console font notes
//! that name only what the screen offers.

use ratatui::crossterm::event::KeyCode;

use super::{index_of, line_of, move_down, open_settings};
use crate::app::{CardFontState, FontFacts};
use crate::caps::ColorDepth;
use crate::i18n::{Locale, set_locale, t};
use crate::screens::settings::options::Field;
use crate::screens::settings::values::line;
use crate::settings::{Settings, TerminalKind};
use crate::test_support::sound_styles;
use crate::theme::{Theme, Token};

const ARROWS: Option<(&str, &str)> = Some(("◄", "►"));

fn unavailable_console_font() -> FontFacts {
    FontFacts {
        card_font: CardFontState::Unavailable,
        ..FontFacts::default()
    }
}

fn text_of(field: Field, audio_available: bool, facts: FontFacts) -> String {
    line(
        field,
        &Settings::default(),
        audio_available,
        &sound_styles(),
        (200, facts),
        ARROWS,
    )
}

#[test]
fn an_unavailable_row_takes_no_arrows_even_when_focused() {
    let text = text_of(Field::ConsoleFont, true, unavailable_console_font());
    let unavailable = t("settings.value.unavailable").into_owned();
    assert!(text.ends_with(&unavailable), "{text}");
    assert!(!text.contains('◄') && !text.contains('►'), "{text}");
}

#[test]
fn a_sound_row_without_a_device_takes_no_arrows_and_an_available_one_keeps_them() {
    let unavailable = text_of(Field::Music, false, FontFacts::default());
    assert!(
        !unavailable.contains('◄') && !unavailable.contains('►'),
        "{unavailable}"
    );
    let available = text_of(Field::Music, true, FontFacts::default());
    assert!(
        available.contains('◄') && available.contains('►'),
        "{available}"
    );
}

#[test]
fn an_available_row_keeps_its_arrows() {
    let text = text_of(Field::Theme, true, unavailable_console_font());
    assert!(text.contains('◄') && text.contains('►'), "{text}");
}

#[test]
fn an_unavailable_row_is_dimmed_and_has_no_arrows_on_screen() {
    let label = t(Field::ConsoleFont.label_key()).into_owned();
    let mut h = open_settings();
    move_down(&mut h, index_of(Field::ConsoleFont));
    let focused = line_of(&mut h, &label);
    assert!(!focused.contains('◄'), "{focused}");
    assert_eq!(
        focused.matches('►').count(),
        1,
        "only the focus marker: {focused}"
    );

    h.press(KeyCode::Down);
    let plain = line_of(&mut h, &label);
    assert!(!plain.contains('◄') && !plain.contains('►'), "{plain}");
    assert!(
        plain.contains(t("settings.value.unavailable").as_ref()),
        "{plain}"
    );

    let theme = Theme::new(h.app.settings().theme, ColorDepth::TrueColor);
    let dimmed = theme.style(Token::TextMuted, Token::Background).fg;
    let (column, row) = h.find(&label).unwrap_or_default();
    let cell = &h.terminal.backend().buffer()[(column, row)];
    assert_eq!(Some(cell.fg), dimmed);
}

#[test]
fn a_focused_unavailable_row_keeps_the_focus_style() {
    let label = t(Field::ConsoleFont.label_key()).into_owned();
    let mut h = open_settings();
    move_down(&mut h, index_of(Field::ConsoleFont));
    let theme = Theme::new(h.app.settings().theme, ColorDepth::TrueColor);
    let (column, row) = h.find(&label).unwrap_or_default();
    let cell = &h.terminal.backend().buffer()[(column, row)];
    assert_eq!(Some(cell.fg), theme.focus().fg);
}

#[test]
fn the_console_font_notes_name_only_what_the_screen_offers_in_every_locale() {
    for locale in Locale::offered() {
        set_locale(locale);
        let symbols_row = t(Field::Glyphs.label_key()).into_owned();
        let facts = |terminal| FontFacts {
            terminal,
            ..unavailable_console_font()
        };
        let windows_terminal = Field::ConsoleFont.describe(facts(TerminalKind::WindowsTerminal));
        let other = Field::ConsoleFont.describe(facts(TerminalKind::Linux));
        assert!(windows_terminal.contains("oXySol Mono"), "{}", locale.id());
        assert!(other.contains(&symbols_row), "{}: {other}", locale.id());
        assert!(!other.contains("oXySol Mono"), "{}: {other}", locale.id());
    }
    set_locale(Locale::EnUs);
}
