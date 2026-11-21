//! The guide on screens too narrow for the side panel: the window that `g` opens, the `Do:` part
//! that names the key, and the panel of a short wide screen.

use oxysol_core::{DrawCount, PyramidRules, SuitCount, Variant};
use ratatui::crossterm::event::{KeyCode, KeyModifiers, MouseEvent, MouseEventKind};

use super::*;
use crate::action::Action;
use crate::app::Modal;
use crate::caps::TerminalCaps;
use crate::glyphs::is_wgl4;
use crate::i18n::{Locale, set_locale, t, tf};
use crate::input::{Input, primary_key};
use crate::settings::Settings;
use crate::test_support::PLAIN;

const KLONDIKE: Variant = Variant::Klondike(DrawCount::Three);
const NARROW: [(u16, u16); 2] = [(100, 30), (115, 35)];

const GAMES: [Variant; 4] = [
    KLONDIKE,
    Variant::Spider(SuitCount::Two),
    Variant::FreeCell,
    Variant::Pyramid(PyramidRules::Relaxed),
];

fn shown(
    variant: Variant,
    (width, height): (u16, u16),
    locale: Locale,
    caps: TerminalCaps,
) -> Harness {
    let storage = MemoryStorage {
        settings: Some(Settings {
            locale,
            moves_panel: false,
            ..Settings::for_caps(&caps)
        }),
        ..MemoryStorage::default()
    };
    let mut h = Harness::with(storage, caps, width, height);
    h.app.apply(Effect::Start {
        variant,
        seed: Some(5),
        undo_allowed: true,
    });
    h.lines();
    h
}

fn english(size: (u16, u16)) -> Harness {
    shown(KLONDIKE, size, Locale::EnUs, TRUE_COLOR)
}

fn window_open(h: &Harness) -> bool {
    matches!(h.app.modal(), Some(Modal::Guide(_)))
}

fn wheel(h: &mut Harness, kind: MouseEventKind) {
    h.send(Input::Mouse(MouseEvent {
        kind,
        column: 50,
        row: 15,
        modifiers: KeyModifiers::NONE,
    }));
}

fn first_visible(h: &mut Harness) -> usize {
    let lines = h.lines();
    let Some(footer) = lines.iter().find(|line| line.contains("PgDn")) else {
        unreachable!("the window has a position")
    };
    let before = footer.split(" / ").next().unwrap_or_default();
    let range = before.rsplit(' ').next().unwrap_or_default();
    range
        .split('-')
        .next()
        .and_then(|from| from.parse().ok())
        .unwrap_or(0)
}

fn last_row(h: &mut Harness) -> String {
    h.lines().last().cloned().unwrap_or_default()
}

#[test]
fn g_opens_the_guide_as_a_window_with_every_section_on_narrow_screens() {
    for size in NARROW {
        let mut h = english(size);
        h.type_char('g');
        assert!(window_open(&h), "{size:?}");
        assert!(h.app.aids().guide_panel, "the setting stays on");
        let text = h.text();
        let needles = [
            "Guide",
            "Game",
            "Klondike, Draw 3",
            "Goal",
            "Build the four",
            "Keys",
            "• Arrows",
            "g/Esc: close",
        ];
        for needle in needles {
            assert!(text.contains(needle), "{size:?}: {needle}\n{text}");
        }
    }
}

#[test]
fn the_window_closes_with_g_esc_enter_and_the_close_button() {
    let mut h = english((100, 30));
    for press in [KeyCode::Char('g'), KeyCode::Esc, KeyCode::Enter] {
        h.type_char('g');
        assert!(window_open(&h));
        h.press(press);
        assert!(!window_open(&h), "{press:?}");
    }
    h.type_char('g');
    let Some((column, row)) = h.find("Close") else {
        unreachable!("the window has a Close button")
    };
    h.click(column, row);
    assert!(!window_open(&h));
}

#[test]
fn the_window_scrolls_by_key_and_wheel_to_its_last_and_back_to_its_first_line() {
    let mut h = english((100, 30));
    h.type_char('g');
    assert_eq!(first_visible(&mut h), 1);
    assert!(
        !h.text().contains("Legend"),
        "the legend is below the first page"
    );
    h.press(KeyCode::End);
    assert!(h.text().contains("Legend"), "{}", h.text());
    assert!(window_open(&h));
    h.press(KeyCode::Home);
    assert_eq!(first_visible(&mut h), 1);
    h.press(KeyCode::Down);
    assert_eq!(first_visible(&mut h), 2);
    h.press(KeyCode::Up);
    h.press(KeyCode::PageDown);
    assert!(first_visible(&mut h) > 2);
    h.press(KeyCode::Home);
    wheel(&mut h, MouseEventKind::ScrollDown);
    assert_eq!(first_visible(&mut h), 4);
    wheel(&mut h, MouseEventKind::ScrollUp);
    assert_eq!(first_visible(&mut h), 1);
}

#[test]
fn scrolling_stops_at_the_end_so_one_key_goes_back() {
    let mut h = english((100, 30));
    h.type_char('g');
    for _ in 0..80 {
        h.press(KeyCode::Down);
    }
    let end = first_visible(&mut h);
    h.press(KeyCode::Up);
    assert_eq!(first_visible(&mut h), end - 1);
}

#[test]
fn the_window_stops_the_clock_and_keys_do_not_reach_the_board() {
    let mut h = english((100, 30));
    h.type_char('g');
    h.advance(30);
    h.type_char('s');
    assert!(h.text().contains("Time 0:00"), "{}", h.text());
    h.type_char('g');
    assert!(!h.text().contains("Dealt from the stock."));
}

#[test]
fn from_130_columns_g_still_hides_and_shows_the_panel() {
    for size in [(130, 40), (130, 30)] {
        let mut h = english(size);
        h.type_char('g');
        assert!(!window_open(&h), "{size:?}");
        assert!(!h.app.aids().guide_panel);
        h.type_char('g');
        assert!(h.app.aids().guide_panel);
        assert!(h.text().contains("Guide (g: hide)"));
    }
}

#[test]
fn the_do_row_keeps_restart_and_options_at_every_width_of_the_guide_window() {
    for size in [(100, 30), (129, 40), (130, 30), (130, 40)] {
        let mut h = english(size);
        let possible = last_row(&mut h);
        assert!(
            possible.contains("Shift+R: restart"),
            "{size:?}: {possible}"
        );
        assert!(possible.contains("o: options"), "{size:?}: {possible}");
    }
}

#[test]
fn the_do_row_keeps_restart_and_options_at_100_columns_in_every_game_and_locale() {
    for locale in Locale::offered() {
        set_locale(locale);
        let restart = tf(
            "board.action.restart",
            &[("key", &primary_key(Action::Restart).unwrap_or_default())],
        );
        let options = tf(
            "board.action.options",
            &[("key", &primary_key(Action::Options).unwrap_or_default())],
        );
        for variant in GAMES {
            let mut h = shown(variant, (100, 30), locale, TRUE_COLOR);
            let browsing = last_row(&mut h);
            assert!(browsing.trim_end().chars().count() <= 100, "{browsing}");
            assert!(browsing.contains(&restart), "{variant:?}: {browsing}");
            assert!(browsing.contains(&options), "{variant:?}: {browsing}");
        }
    }
    set_locale(Locale::EnUs);
}

#[test]
fn the_do_row_keeps_restart_while_cards_are_carried() {
    let mut h = english((100, 30));
    let Some((from, _, _)) = first_transfer(&h) else {
        unreachable!("seed 5 has a transfer")
    };
    put_cursor(&mut h, from);
    h.press(KeyCode::Enter);
    let carrying = last_row(&mut h);
    assert!(carrying.starts_with("Do: "), "{carrying}");
    assert!(carrying.contains("Shift+R: restart"), "{carrying}");
}

fn every_narrow_screen() -> Vec<(Variant, (u16, u16), Locale, TerminalCaps)> {
    let mut all = Vec::new();
    for size in NARROW {
        for locale in Locale::offered() {
            for caps in [TRUE_COLOR, PLAIN] {
                all.extend(GAMES.map(|variant| (variant, size, locale, caps)));
            }
        }
    }
    all
}

#[test]
fn every_narrow_size_locale_and_glyph_mode_draws_the_window_inside_the_screen() {
    for (variant, size, locale, caps) in every_narrow_screen() {
        let mut h = shown(variant, size, locale, caps);
        h.type_char('g');
        let lines = h.lines();
        assert_eq!(lines.len(), usize::from(size.1));
        let plain_english = !caps.unicode && locale == Locale::EnUs;
        for ch in lines.concat().chars() {
            assert!(ch.is_ascii() || is_wgl4(ch), "{size:?} {variant:?}: {ch:?}");
            assert!(!plain_english || ch.is_ascii(), "{size:?}: {ch:?}");
        }
        set_locale(locale);
        let title = t("board.guide.window_title").into_owned();
        assert!(lines.concat().contains(&title), "{size:?} {variant:?}");
    }
    set_locale(Locale::EnUs);
}

#[test]
fn a_short_wide_screen_shows_the_panel_with_every_section_it_has_room_for() {
    let mut h = english((130, 30));
    let panel: Vec<String> = h
        .lines()
        .iter()
        .map(|line| line.chars().skip(100).collect())
        .collect();
    let text = panel.join("\n");
    for needle in ["Guide (g: hide)", "Game", "Goal", "Keys", "Legend", "focus"] {
        assert!(text.contains(needle), "{needle}\n{text}");
    }
    assert!(
        !text.contains("Mouse"),
        "the mouse section goes first:\n{text}"
    );
    let legend = panel.iter().position(|row| row.contains("Legend"));
    assert!(
        legend.is_some_and(|row| panel[row + 1].contains("focus")),
        "{text}"
    );
}

#[test]
fn a_tall_panel_keeps_every_section() {
    let mut h = english((130, 40));
    let text = h.text();
    for needle in ["Goal", "Keys", "Mouse", "Legend"] {
        assert!(text.contains(needle), "{needle}");
    }
}
