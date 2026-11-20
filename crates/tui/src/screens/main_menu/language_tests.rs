//! The language selector in the corner of the main menu: switching, the list, the tooltip, the Tab
//! order, persistence, the fallback without flags, and the renders at the two reference sizes.

use ratatui::crossterm::event::{KeyCode, KeyModifiers, MouseEvent, MouseEventKind};

use crate::app::Screen;
use crate::caps::TerminalCaps;
use crate::i18n::Locale;
use crate::input::Input;
use crate::settings::{ColorMode, Settings};
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, PLAIN, TRUE_COLOR};

const FLAG_ROW: &str = "◄ ▀▀▀▀ ► ▼";

fn harness(locale: Locale, caps: TerminalCaps, width: u16, height: u16) -> Harness {
    let storage = MemoryStorage {
        settings: Some(Settings {
            locale,
            ..Settings::for_caps(&caps)
        }),
        ..MemoryStorage::new()
    };
    Harness::with(storage, caps, width, height)
}

fn english() -> Harness {
    harness(Locale::EnUs, TRUE_COLOR, 120, 40)
}

fn focus_selector(h: &mut Harness) {
    h.press(KeyCode::BackTab);
    assert!(
        Locale::offered()
            .iter()
            .any(|loc| h.text().contains(loc.own_name()))
    );
}

fn hover(h: &mut Harness, column: u16, row: u16) {
    h.send(Input::Mouse(MouseEvent {
        kind: MouseEventKind::Moved,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }));
}

fn locale(h: &Harness) -> Locale {
    h.app.settings().locale
}

fn stored(h: &Harness) -> Option<Locale> {
    h.storage
        .borrow()
        .settings
        .as_ref()
        .map(|settings| settings.locale)
}

fn left_arrow(h: &mut Harness) -> (u16, u16) {
    h.find("◄").unwrap_or_default()
}

fn right_arrow(h: &mut Harness) -> (u16, u16) {
    let (column, row) = left_arrow(h);
    (column + 7, row)
}

fn down_arrow(h: &mut Harness) -> (u16, u16) {
    let (column, row) = left_arrow(h);
    (column + 9, row)
}

fn open_by_mouse(h: &mut Harness) {
    let (column, row) = down_arrow(h);
    h.click(column, row);
}

#[test]
fn the_corner_shows_the_flag_between_arrows_with_the_code_below() {
    let mut h = english();
    let lines = h.lines();
    assert!(lines[1].trim_start().starts_with(FLAG_ROW), "{}", lines[1]);
    assert!(lines[2].contains("▀▀▀▀ EN"), "{}", lines[2]);
    assert!(lines[1].starts_with(" ◄"));
}

#[test]
fn the_corner_shows_the_polish_flag_and_code_in_polish() {
    let mut h = harness(Locale::PlPl, TRUE_COLOR, 120, 40);
    let lines = h.lines();
    assert!(lines[1].trim_start().starts_with(FLAG_ROW), "{}", lines[1]);
    assert!(lines[2].contains("▀▀▀▀ PL"), "{}", lines[2]);
    assert!(h.text().contains("Nowa gra"));
}

#[test]
fn the_flag_pixels_are_the_flag_colors() {
    use ratatui::style::Color;
    let mut h = english();
    h.lines();
    let buffer = h.terminal.backend().buffer();
    let canton = &buffer[(4, 1)];
    assert_eq!(canton.symbol(), "▀");
    assert_eq!(canton.fg, Color::Rgb(60, 59, 110));
    assert_eq!(canton.bg, Color::Rgb(60, 59, 110));
    let stripe = &buffer[(6, 1)];
    assert_eq!(stripe.fg, Color::Rgb(200, 16, 46));
    assert_eq!(stripe.bg, Color::Rgb(255, 255, 255));
}

#[test]
fn the_menu_fits_with_the_selector_at_both_reference_sizes_in_both_locales() {
    for (width, height) in [(100, 30), (130, 40)] {
        for locale in Locale::offered() {
            let mut h = harness(locale, TRUE_COLOR, width, height);
            let lines = h.lines();
            assert!(lines[1].trim_start().starts_with(FLAG_ROW));
            let text = lines.join("\n");
            let quit = crate::i18n::t("common.menu.quit").into_owned();
            assert!(text.contains(&quit), "{width}x{height} {locale:?}");
            assert!(
                lines[..4]
                    .iter()
                    .all(|line| line.chars().skip(13).all(|c| c == ' ')),
                "the corner shares its rows with nothing at {width}x{height}"
            );
        }
    }
}

#[test]
fn the_open_list_fits_at_both_reference_sizes_in_both_locales() {
    for (width, height) in [(100, 30), (130, 40)] {
        for locale in Locale::offered() {
            let mut h = harness(locale, TRUE_COLOR, width, height);
            focus_selector(&mut h);
            h.press(KeyCode::Enter);
            let text = h.text();
            assert!(text.contains("English (US)"), "{width}x{height} {locale:?}");
            assert!(text.contains("Polski"));
            assert!(text.contains("EN") && text.contains("PL"));
            assert!(text.contains('●'));
        }
    }
}

#[test]
fn left_and_right_switch_the_language_at_once_and_save_it() {
    let mut h = english();
    focus_selector(&mut h);
    h.press(KeyCode::Right);
    assert_eq!(locale(&h), Locale::PlPl);
    assert_eq!(stored(&h), Some(Locale::PlPl));
    assert!(h.text().contains("Nowa gra"));
    h.press(KeyCode::Left);
    assert_eq!(locale(&h), Locale::EnUs);
    assert_eq!(stored(&h), Some(Locale::EnUs));
    h.press(KeyCode::Left);
    assert_eq!(Some(locale(&h)), Locale::offered().last().copied());
}

#[test]
fn the_selector_shares_the_language_row_of_the_settings() {
    let mut h = english();
    focus_selector(&mut h);
    h.press(KeyCode::Right);
    h.press(KeyCode::Tab);
    for _ in 0..3 {
        h.press(KeyCode::Tab);
    }
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Settings);
    assert!(h.text().contains("Polski"));
}

#[test]
fn the_list_opens_with_enter_and_space_and_chooses_with_enter() {
    for key in [KeyCode::Enter, KeyCode::Char(' ')] {
        let mut h = english();
        focus_selector(&mut h);
        h.press(key);
        assert!(h.text().contains("Polski"));
        assert_eq!(locale(&h), Locale::EnUs);
        h.press(KeyCode::Down);
        h.press(KeyCode::Enter);
        assert_eq!(locale(&h), Locale::PlPl);
        assert_eq!(stored(&h), Some(Locale::PlPl));
        assert!(!h.text().contains("English (US)"));
    }
}

#[test]
fn down_from_the_selector_returns_to_the_menu() {
    let mut h = english();
    focus_selector(&mut h);
    h.press(KeyCode::Down);
    assert!(h.text().contains("► New game"));
}

#[test]
fn up_in_the_list_wraps_and_escape_closes_it_without_a_change() {
    let mut h = english();
    focus_selector(&mut h);
    h.press(KeyCode::Enter);
    h.press(KeyCode::Up);
    h.press(KeyCode::Esc);
    assert_eq!(locale(&h), Locale::EnUs);
    assert!(!h.text().contains("Polski"));
    assert_eq!(h.app.screen(), Screen::MainMenu);
    h.press(KeyCode::Enter);
    h.press(KeyCode::Up);
    h.press(KeyCode::Enter);
    assert_eq!(Some(locale(&h)), Locale::offered().last().copied());
}

#[test]
fn the_open_list_takes_every_key() {
    let mut h = english();
    focus_selector(&mut h);
    h.press(KeyCode::Enter);
    h.press(KeyCode::Tab);
    h.type_char('q');
    assert!(!h.app.should_quit());
    assert_eq!(h.app.screen(), Screen::MainMenu);
    assert!(h.text().contains("Polski"));
}

#[test]
fn the_arrows_switch_by_mouse() {
    let mut h = english();
    let (column, row) = right_arrow(&mut h);
    h.click(column, row);
    assert_eq!(locale(&h), Locale::PlPl);
    assert_eq!(stored(&h), Some(Locale::PlPl));
    let (column, row) = left_arrow(&mut h);
    h.click(column, row);
    assert_eq!(locale(&h), Locale::EnUs);
    assert_eq!(stored(&h), Some(Locale::EnUs));
}

#[test]
fn the_down_arrow_opens_the_list_and_a_click_on_a_language_chooses_it() {
    let mut h = english();
    open_by_mouse(&mut h);
    let (column, row) = h.find("Polski").unwrap_or_default();
    h.click(column, row);
    assert_eq!(locale(&h), Locale::PlPl);
    assert_eq!(stored(&h), Some(Locale::PlPl));
    assert!(!h.text().contains("English (US)"));
}

#[test]
fn a_click_outside_closes_the_list_and_does_nothing_else() {
    let mut h = english();
    open_by_mouse(&mut h);
    let (column, row) = h.find("Quit").unwrap_or_default();
    h.click(column, row);
    assert!(!h.app.should_quit());
    assert_eq!(locale(&h), Locale::EnUs);
    assert!(!h.text().contains("Polski"));
    h.click(column, row);
    assert!(h.app.should_quit());
}

#[test]
fn hovering_a_row_of_the_open_list_highlights_it() {
    let mut h = english();
    open_by_mouse(&mut h);
    let (column, row) = h.find("Polski").unwrap_or_default();
    hover(&mut h, column, row);
    h.press(KeyCode::Enter);
    assert_eq!(locale(&h), Locale::PlPl);
}

#[test]
fn the_wheel_moves_the_highlight_of_the_open_list() {
    let mut h = english();
    open_by_mouse(&mut h);
    h.send(Input::Mouse(MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: 40,
        row: 20,
        modifiers: KeyModifiers::NONE,
    }));
    h.press(KeyCode::Enter);
    assert_eq!(locale(&h), Locale::PlPl);
}

#[test]
fn the_tooltip_names_the_language_in_itself_on_hover() {
    let mut h = english();
    assert!(!h.text().contains("English (US)"));
    let (column, row) = left_arrow(&mut h);
    hover(&mut h, column + 2, row);
    assert!(h.text().contains("English (US)"));
    hover(&mut h, 60, 20);
    assert!(!h.text().contains("English (US)"));
    let mut polish = harness(Locale::PlPl, TRUE_COLOR, 120, 40);
    let (column, row) = left_arrow(&mut polish);
    hover(&mut polish, column + 2, row + 1);
    assert!(polish.text().contains("Polski"));
    assert!(!polish.text().contains("English (US)"));
}

#[test]
fn the_tooltip_shows_on_keyboard_focus_too() {
    let mut h = english();
    assert!(!h.text().contains("English (US)"));
    h.press(KeyCode::BackTab);
    assert!(h.text().contains("English (US)"));
    h.press(KeyCode::Tab);
    assert!(!h.text().contains("English (US)"));
}

#[test]
fn the_selector_is_first_in_the_tab_order_before_the_items() {
    let mut h = english();
    h.press(KeyCode::BackTab);
    let lines = h.lines();
    assert!(lines[1].contains("◄ ▀▀▀▀ ► ▼"), "{}", lines[1]);
    assert!(!lines[1].contains("►◄"));
    assert!(lines[4..].iter().all(|line| !line.contains('►')));
    h.press(KeyCode::Tab);
    let text = h.text();
    assert!(text.contains("► New game"));
    assert_eq!(h.lines()[1].matches('►').count(), 1);
}

#[test]
fn the_tab_ring_wraps_between_the_last_item_the_selector_and_the_first_item() {
    let mut h = english();
    h.press(KeyCode::BackTab);
    h.press(KeyCode::BackTab);
    assert!(h.text().contains("► Quit"));
    h.press(KeyCode::Tab);
    assert!(h.text().contains("English (US)"));
    h.press(KeyCode::Tab);
    assert!(h.text().contains("► New game"));
}

#[test]
fn enter_on_the_items_still_activates_them_after_the_selector_lost_the_focus() {
    let mut h = english();
    focus_selector(&mut h);
    h.press(KeyCode::Tab);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::GameSelect);
}

#[test]
fn a_click_on_an_item_returns_the_focus_to_the_list() {
    let mut h = english();
    h.press(KeyCode::BackTab);
    let (column, row) = h.find("Help").unwrap_or_default();
    h.click(column, row);
    assert!(h.app.modal().is_some());
}

#[test]
fn a_click_on_the_flag_gives_the_selector_the_focus() {
    let mut h = english();
    let (column, row) = left_arrow(&mut h);
    h.click(column + 3, row);
    assert!(h.text().contains("English (US)"));
    h.press(KeyCode::Right);
    assert_eq!(locale(&h), Locale::PlPl);
}

#[test]
fn the_code_replaces_the_flag_in_ascii_mode() {
    let mut h = harness(Locale::EnUs, PLAIN, 100, 30);
    let lines = h.lines();
    assert!(
        lines[1].trim_start().starts_with("< EN > v"),
        "{}",
        lines[1]
    );
    assert!(!h.text().contains('▀'));
    h.press(KeyCode::BackTab);
    assert!(h.lines()[1].starts_with(" < EN > v"));
    h.press(KeyCode::Right);
    assert!(h.lines()[1].contains("< PL > v"));
    assert_eq!(locale(&h), Locale::PlPl);
}

#[test]
fn the_list_shows_codes_instead_of_flags_in_ascii_mode() {
    let mut h = harness(Locale::EnUs, PLAIN, 100, 30);
    h.press(KeyCode::BackTab);
    h.press(KeyCode::Enter);
    let text = h.text();
    assert!(!text.contains('▀'));
    assert!(text.contains("EN English (US)"), "{text}");
    assert!(text.contains("PL Polski"));
    assert!(text.contains('*'));
    let (column, row) = h.find("PL Polski").unwrap_or_default();
    h.click(column + 4, row);
    assert_eq!(locale(&h), Locale::PlPl);
}

#[test]
fn sixteen_colors_show_the_code_instead_of_the_flag_even_with_unicode() {
    let storage = MemoryStorage {
        settings: Some(Settings {
            color_mode: ColorMode::Ansi16,
            ..Settings::for_caps(&TRUE_COLOR)
        }),
        ..MemoryStorage::new()
    };
    let mut h = Harness::with(storage, TRUE_COLOR, 100, 30);
    let lines = h.lines();
    assert!(
        lines[1].trim_start().starts_with("◄ EN ► ▼"),
        "{}",
        lines[1]
    );
    assert!(lines[..3].iter().all(|line| !line.contains('▀')));
}

#[test]
fn the_tooltip_and_arrows_work_without_flags_too() {
    let mut h = harness(Locale::EnUs, PLAIN, 100, 30);
    hover(&mut h, 5, 1);
    assert!(h.text().contains("English (US)"));
    hover(&mut h, 60, 20);
    h.click(6, 1);
    assert_eq!(locale(&h), Locale::PlPl);
}

#[test]
fn with_the_list_open_no_extra_left_arrow_is_drawn() {
    for locale in Locale::offered() {
        let mut h = harness(locale, TRUE_COLOR, 100, 30);
        focus_selector(&mut h);
        h.press(KeyCode::Enter);
        assert_eq!(h.lines()[1].matches('◄').count(), 1, "{locale:?}");
        assert!(!h.lines()[1].contains("►◄"), "{locale:?}");
        assert!(!h.lines()[1].contains("◄◄"), "{locale:?}");
    }
}

#[test]
fn the_footer_shows_the_tab_entry_at_100x30_in_every_offered_locale() {
    for locale in Locale::offered() {
        let mut h = harness(locale, TRUE_COLOR, 100, 30);
        let footer_line = h.lines().last().cloned().unwrap_or_default();
        let expected_key = crate::i18n::t("menu.footer.language");
        let expected = format!("Tab {expected_key}");
        assert!(
            footer_line.contains(&expected),
            "footer in {locale:?} should contain {expected:?}, got: {footer_line:?}"
        );
    }
}
