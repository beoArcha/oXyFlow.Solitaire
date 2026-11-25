//! The quick options dialog as drawn (O-90): the focused row uses the selection style of the other
//! lists, the focused button the focused-button style, the Aura row carries the Settings name in
//! every complete locale, and the dialog keeps its text at 100 by 30 and 100 by 60.

use oxysol_core::{DrawCount, Variant};
use ratatui::buffer::Cell;
use ratatui::crossterm::event::KeyCode;

use super::*;
use crate::action::Action;
use crate::app::Modal;
use crate::i18n::{Locale, set_locale, t, tf};
use crate::input::primary_key;
use crate::settings::{Aid, Settings};
use crate::layout::STANDARD_MIN_WIDTH;
use crate::theme::{Theme, ThemeId, Token};

const KLONDIKE: Variant = Variant::Klondike(DrawCount::One);
const WIDE: (u16, u16) = (140, 40);
const SAVE: usize = Aid::ALL.len();

fn game_with(settings: Settings, size: (u16, u16)) -> Harness {
    let storage = MemoryStorage {
        settings: Some(settings),
        ..MemoryStorage::default()
    };
    let mut h = Harness::with(storage, TRUE_COLOR, size.0, size.1);
    h.app.apply(Effect::Start {
        variant: KLONDIKE,
        seed: Some(5),
        undo_allowed: true,
    });
    h.lines();
    h
}

fn open(h: &mut Harness) {
    h.type_char('o');
    assert!(matches!(h.app.modal(), Some(Modal::Quick(_))), "{}", h.text());
}

fn theme_of(h: &Harness) -> Theme {
    Theme::new(h.app.settings().theme, TRUE_COLOR.color)
}

fn cell_at(h: &mut Harness, needle: &str) -> Cell {
    let (column, row) = h
        .find(needle)
        .unwrap_or_else(|| panic!("{needle:?} is not drawn:\n{}", h.text()));
    h.terminal.backend().buffer()[(column, row)].clone()
}

fn aura_row(h: &mut Harness) -> Cell {
    cell_at(h, &format!("{} .", t(Aid::Aura.label_key())))
}

fn target_row(h: &mut Harness) -> Cell {
    cell_at(h, &format!("{} .", t(Aid::TargetMarks.label_key())))
}

#[test]
fn the_focused_row_is_drawn_in_the_selection_style_of_the_other_lists() {
    let mut h = game_with(Settings::default(), WIDE);
    open(&mut h);
    let theme = theme_of(&h);
    let focused = aura_row(&mut h);
    assert_eq!(focused.bg, theme.color(Token::FocusBg));
    assert_eq!(focused.fg, theme.color(Token::FocusFg));
    let other = target_row(&mut h);
    assert_eq!(other.bg, theme.color(Token::Surface));

    h.press(KeyCode::Down);
    assert_eq!(target_row(&mut h).bg, theme.color(Token::FocusBg));
    assert_eq!(aura_row(&mut h).bg, theme.color(Token::Surface));
}

#[test]
fn the_focused_row_is_drawn_with_the_settings_selection_colour_in_every_theme() {
    for theme_id in ThemeId::ALL {
        let settings = Settings {
            theme: theme_id,
            ..Settings::default()
        };
        let mut h = game_with(settings, WIDE);
        open(&mut h);
        let theme = theme_of(&h);
        assert_eq!(aura_row(&mut h).bg, theme.color(Token::FocusBg), "{theme_id:?}");
    }
}

#[test]
fn the_focused_button_is_framed_and_drawn_in_the_focused_button_style() {
    let mut h = game_with(Settings::default(), (100, 30));
    open(&mut h);
    for _ in 0..SAVE {
        h.press(KeyCode::Tab);
    }
    let theme = theme_of(&h);
    let save = t("board.options.save").into_owned();
    let close = tf("help.rules.close", &[("key", "Esc")]);
    let text = h.text();
    let framed = [format!("► {save} ◄"), format!("> {save} <")];
    assert!(framed.iter().any(|mark| text.contains(mark)), "{text}");
    assert_eq!(cell_at(&mut h, &save).fg, theme.color(Token::FocusBg));
    assert_ne!(cell_at(&mut h, &close).fg, theme.color(Token::FocusBg));
}

#[test]
fn every_complete_locale_keeps_the_dialog_text_at_100_by_30_and_100_by_60() {
    for locale in Locale::offered() {
        for size in [(100, 30), (100, 60)] {
            let settings = Settings {
                locale,
                ..Settings::default()
            };
            let mut h = game_with(settings, size);
            open(&mut h);
            let text = h.text();
            let context = format!("{locale:?} {size:?}");
            for aid in Aid::ALL {
                let label = t(aid.label_key()).into_owned();
                assert!(text.contains(&label), "{context}: {label:?}\n{text}");
            }
            let save = t("board.options.save").into_owned();
            let close = tf("help.rules.close", &[("key", "Esc")]);
            assert!(text.contains(&save), "{context}: {save:?}\n{text}");
            assert!(text.contains(&close), "{context}: {close:?}\n{text}");
            let description = t(Aid::Aura.description_key()).into_owned();
            for word in description.split_whitespace() {
                assert!(text.contains(word), "{context}: {word:?}\n{text}");
            }
            let footer = tf(
                "board.options.footer",
                &[
                    ("up", &primary_key(Action::Up).unwrap_or_default()),
                    ("down", &primary_key(Action::Down).unwrap_or_default()),
                    ("left", &primary_key(Action::Left).unwrap_or_default()),
                    ("right", &primary_key(Action::Right).unwrap_or_default()),
                    ("select", &primary_key(Action::Select).unwrap_or_default()),
                ],
            );
            assert!(text.contains(&footer), "{context}: {footer:?}\n{text}");
            let needs = tf(
                "settings.value.needs_columns",
                &[
                    ("value", &t("settings.value.on")),
                    ("needed", &STANDARD_MIN_WIDTH.to_string()),
                    ("now", &size.0.to_string()),
                ],
            );
            assert!(text.contains(&needs), "{context}: {needs:?}\n{text}");
        }
    }
    set_locale(Locale::EnUs);
}

#[test]
fn the_aura_row_has_the_settings_name_in_every_complete_locale() {
    for locale in Locale::offered() {
        set_locale(locale);
        let name = t("settings.row.aura").into_owned();
        let description = t("settings.desc.aura").into_owned();
        assert!(!description.contains("Glow"), "{locale:?}: {description}");
        let Some(rest) = description.strip_prefix(&name) else {
            panic!("{locale:?}: the description {description:?} does not start with {name:?}");
        };
        assert!(
            rest.starts_with(':') || rest.starts_with(" :"),
            "{locale:?}: {description:?}"
        );
    }
    set_locale(Locale::EnUs);
}
