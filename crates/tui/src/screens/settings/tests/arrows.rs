use ratatui::crossterm::event::KeyCode;

use super::{SETTINGS_ITEM, changed, move_down, open_settings, value};
use crate::i18n::{Locale, set_locale};
use crate::screens::settings::options::{Field, Step};
use crate::screens::settings::values::line;
use crate::settings::Settings;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, PLAIN, sound_styles};

/// The cells of the two arrows of the focused row: the first `◄` and the last `►` on the screen.
fn arrow_cells(h: &mut Harness) -> Option<((u16, u16), (u16, u16))> {
    let lines = h.lines();
    let row = lines.iter().position(|line| line.contains('◄'))?;
    let chars: Vec<char> = lines.get(row)?.chars().collect();
    let left = chars.iter().position(|c| *c == '◄')?;
    let right = chars.iter().rposition(|c| *c == '►')?;
    Some((
        (u16::try_from(left).ok()?, u16::try_from(row).ok()?),
        (u16::try_from(right).ok()?, u16::try_from(row).ok()?),
    ))
}

#[test]
fn only_the_focused_row_shows_its_value_between_arrows() {
    set_locale(Locale::EnUs);
    let mut h = open_settings();
    let language = format!("◄ {} ►", value(Field::Language, h.app.settings()));
    assert!(h.text().contains(&language), "{}", h.text());
    assert_eq!(h.text().matches('◄').count(), 1);
    move_down(&mut h, 1);
    let theme = format!("◄ {} ►", value(Field::Theme, h.app.settings()));
    assert!(h.text().contains(&theme), "{}", h.text());
    assert_eq!(h.text().matches('◄').count(), 1);
}

#[test]
fn the_values_line_up_with_and_without_arrows() {
    let settings = Settings::default();
    let sounds = sound_styles();
    let plain = line(Field::Theme, &settings, true, &sounds, 200, None);
    let framed = line(
        Field::Theme,
        &settings,
        true,
        &sounds,
        200,
        Some(("◄", "►")),
    );
    let shown = value(Field::Theme, &settings);
    let column = |text: &str| text.find(&shown).map(|at| text[..at].chars().count());
    assert_eq!(column(&plain), column(&framed), "{plain:?} / {framed:?}");
    assert!(framed.ends_with(&format!("{shown} ►")), "{framed}");
    assert!(!plain.contains('◄'));
}

#[test]
fn a_click_on_the_left_arrow_steps_back_and_one_on_the_right_arrow_forward() {
    set_locale(Locale::EnUs);
    let mut h = open_settings();
    move_down(&mut h, 1);

    let before = h.app.settings().clone();
    let (left, _) = arrow_cells(&mut h).unwrap_or_default();
    h.click(left.0, left.1);
    assert_eq!(
        h.app.settings().theme,
        changed(Field::Theme, &before, Step::Previous).theme
    );

    let before = h.app.settings().clone();
    let (_, right) = arrow_cells(&mut h).unwrap_or_default();
    h.click(right.0, right.1);
    assert_eq!(
        h.app.settings().theme,
        changed(Field::Theme, &before, Step::Next).theme
    );
}

#[test]
fn ascii_arrows_are_angle_brackets() {
    let mut h = Harness::with(MemoryStorage::new(), PLAIN, 120, 40);
    move_down(&mut h, SETTINGS_ITEM);
    h.press(KeyCode::Enter);
    let text = h.text();
    assert!(text.is_ascii());
    assert!(text.contains("< "), "{text}");
    assert!(text.contains(" >"), "{text}");
}
