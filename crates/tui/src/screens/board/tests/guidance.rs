use oxysol_core::{DrawCount, Move, PyramidRules, SuitCount, Variant};
use ratatui::crossterm::event::KeyCode;

use super::*;
use crate::announce::{card_name, pile_name, targets_text};
use crate::glyphs::is_wgl4;
use crate::i18n::{Locale, set_locale};
use crate::settings::Settings;

const KLONDIKE: Variant = Variant::Klondike(DrawCount::One);
const TITLE: &str = "Guide (g: hide)";
const PANEL_COLUMN: usize = 110;

fn row(h: &mut Harness, prefix: &str) -> String {
    h.lines()
        .into_iter()
        .map(|line| line.trim().to_owned())
        .find(|line| line.starts_with(prefix))
        .unwrap_or_default()
}

fn sized(variant: Variant, width: u16, height: u16) -> Harness {
    let mut h = Harness::with(MemoryStorage::new(), TRUE_COLOR, width, height);
    h.app.apply(Effect::Start {
        variant,
        seed: Some(5),
        undo_allowed: true,
    });
    h
}

fn top_card(h: &Harness, pile: PileRef) -> String {
    let card = session(h)
        .game()
        .piles()
        .into_iter()
        .find(|snapshot| snapshot.pile == pile)
        .and_then(|snapshot| snapshot.cards.last().map(|table| table.card));
    card.map(card_name).unwrap_or_default()
}

fn panel_rows(h: &mut Harness) -> Vec<String> {
    h.lines()
        .iter()
        .map(|line| line.chars().skip(PANEL_COLUMN).collect::<String>())
        .collect()
}

fn panel_row(h: &mut Harness, keys: &str) -> String {
    panel_rows(h)
        .into_iter()
        .map(|line| line.chars().skip(2).collect::<String>())
        .find(|line| line.chars().skip(2).collect::<String>().starts_with(keys))
        .unwrap_or_default()
}

fn first_move_played(h: &mut Harness) {
    let Some(mv) = hint_moves(KLONDIKE, 5).into_iter().next() else {
        unreachable!("seed 5 has a hint")
    };
    match mv {
        Move::Transfer { from, to, .. } => drag(h, from, to),
        _ => h.type_char('s'),
    }
}

#[test]
fn browsing_names_the_place_and_the_actions_of_this_state() {
    let mut h = start(KLONDIKE, 5);
    let Some((from, _, _)) = first_transfer(&h) else {
        unreachable!("seed 5 has a transfer")
    };
    put_cursor(&mut h, from);
    let card = top_card(&h, from);
    let now = row(&mut h, "Now:");
    assert_eq!(now, format!("Now: Browsing {}, {card}", pile_name(from)));
    let possible = row(&mut h, "Possible:");
    assert!(
        possible.starts_with("Possible: Enter: pick up"),
        "{possible}"
    );
    assert!(possible.contains("s: turn stock"), "{possible}");
    assert!(possible.contains("h: hint"), "{possible}");
    assert!(!possible.contains("u: undo"), "{possible}");
}

#[test]
fn a_run_deeper_in_a_column_is_named_with_its_size() {
    let mut h = start(KLONDIKE, 5);
    let pile = (0..7)
        .map(PileRef::Tableau)
        .find(|pile| pile_len(&h, *pile) >= 3);
    let Some(pile) = pile else {
        unreachable!("seed 5 has a column of three cards or more")
    };
    put_cursor(&mut h, pile);
    h.type_char('+');
    let (_, depth) = cursor(&h);
    if depth >= 2 {
        let now = row(&mut h, "Now:");
        assert!(now.contains(&format!("{depth} cards from")), "{now}");
        assert!(row(&mut h, "Possible:").contains("+/-: change run size"));
    }
}

#[test]
fn carrying_names_the_cards_the_source_and_the_targets() {
    let mut h = start(KLONDIKE, 5);
    let Some((from, _, count)) = first_transfer(&h) else {
        unreachable!("seed 5 has a transfer")
    };
    put_cursor(&mut h, from);
    h.press(KeyCode::Enter);
    let targets = board(&h).marks().targets.len();
    let card = top_card(&h, from);
    let now = row(&mut h, "Now:");
    if count == 1 {
        let expected = format!(
            "Now: Carrying {card} from {}, {}",
            pile_name(from),
            targets_text(targets)
        );
        assert_eq!(now, expected);
    } else {
        assert!(now.starts_with("Now: Carrying "), "{now}");
    }
    let possible = row(&mut h, "Possible:");
    assert!(
        possible.starts_with("Possible: Enter: drop | Esc: put back | Arrows: choose target"),
        "{possible}"
    );
}

#[test]
fn the_first_steps_hint_is_gone_and_the_keys_stay_in_the_action_line() {
    let mut h = start(KLONDIKE, 5);
    assert!(!h.text().contains("Enter: pick up / drop"));
    let possible = row(&mut h, "Possible:");
    assert!(possible.contains("?: keys") || possible.contains("i: how to play"));
}

#[test]
fn a_position_without_moves_offers_undo_restart_and_a_new_game() {
    let variant = Variant::Pyramid(PyramidRules::Classic);
    let stuck = (1..80_u64).find_map(|seed| {
        let moves = hint_moves(variant, seed);
        let h = resumed(variant, seed, &moves);
        let done = session(&h).game().legal_moves().is_empty() && !session(&h).game().is_won();
        done.then_some((seed, moves))
    });
    let Some((seed, moves)) = stuck else {
        unreachable!("some classic pyramid deal ends without moves")
    };
    let mut h = resumed(variant, seed, &moves);
    assert_eq!(row(&mut h, "Now:"), "Now: No legal moves");
    let possible = row(&mut h, "Possible:");
    assert!(
        possible.starts_with("Possible: u: undo | R: restart | n: new game"),
        "{possible}"
    );
}

#[test]
fn a_won_game_offers_a_new_game_and_the_menu() {
    let variant = Variant::Pyramid(PyramidRules::Relaxed);
    let moves = hint_moves(variant, 2);
    let mut h = resumed(variant, 2, &moves);
    assert!(session(&h).game().is_won());
    assert_eq!(row(&mut h, "Now:"), "Now: Game won");
    let possible = row(&mut h, "Possible:");
    assert!(
        possible.starts_with("Possible: n: new game | m: menu"),
        "{possible}"
    );
}

#[test]
fn the_action_line_follows_the_stock_of_each_game() {
    let mut h = start(Variant::FreeCell, 5);
    assert!(!row(&mut h, "Possible:").contains("stock"));
    let mut h = start(Variant::Spider(SuitCount::One), 5);
    put_cursor(&mut h, PileRef::Tableau(0));
    assert!(row(&mut h, "Possible:").contains("s: deal a row"));
    let mut h = start(KLONDIKE, 5);
    put_cursor(&mut h, PileRef::Stock);
    assert!(row(&mut h, "Possible:").starts_with("Possible: Enter: turn stock"));
}

#[test]
fn the_action_line_fits_the_smallest_terminal_in_every_game_and_locale() {
    for locale in Locale::ALL {
        for variant in Variant::ALL {
            let mut h = sized(variant, 100, 30);
            set_locale(locale);
            let lines = h.lines();
            set_locale(Locale::EnUs);
            let prefixes = match locale {
                Locale::EnUs => ("Now:", "Possible:"),
                Locale::PlPl => ("Teraz:", "Możliwe:"),
            };
            let found = |prefix: &str| lines.iter().any(|line| line.trim().starts_with(prefix));
            assert!(found(prefixes.0) && found(prefixes.1), "{variant:?}");
            for line in &lines {
                assert!(line.chars().count() <= 100, "{variant:?}: {line}");
            }
        }
    }
}

#[test]
fn the_action_line_has_polish_text() {
    let mut h = start(KLONDIKE, 5);
    set_locale(Locale::PlPl);
    let now = row(&mut h, "Teraz:");
    let possible = row(&mut h, "Możliwe:");
    set_locale(Locale::EnUs);
    assert!(now.starts_with("Teraz: Przeglądasz"), "{now}");
    assert!(
        possible.contains("weź") || possible.contains("dobierz"),
        "{possible}"
    );
}

#[test]
fn a_paused_board_shows_no_action_line() {
    let mut h = start(KLONDIKE, 5);
    h.type_char('p');
    assert_eq!(row(&mut h, "Now:"), "");
    assert_eq!(row(&mut h, "Possible:"), "");
}

#[test]
fn the_guide_is_shown_at_140_by_40_and_hidden_at_100_by_30() {
    let mut wide = sized(KLONDIKE, 140, 40);
    assert!(wide.text().contains(TITLE));
    assert!(wide.text().contains("Build the four foundations"));
    let mut small = sized(KLONDIKE, 100, 30);
    assert!(!small.text().contains(TITLE));
    assert!(!small.text().contains("Mouse"));
}

#[test]
fn the_board_keeps_its_width_and_the_guide_needs_130_columns() {
    let mut edge = sized(KLONDIKE, 130, 40);
    assert!(edge.text().contains(TITLE));
    let mut narrow = sized(KLONDIKE, 129, 40);
    assert!(!narrow.text().contains(TITLE));
}

#[test]
fn the_guide_is_off_when_the_setting_is_off() {
    let mut h = sized(KLONDIKE, 140, 40);
    h.app.apply(Effect::Apply(Settings {
        guide_panel: false,
        ..Settings::default()
    }));
    assert!(!h.text().contains(TITLE));
    h.app.apply(Effect::Apply(Settings::default()));
    assert!(h.text().contains(TITLE));
}

#[test]
fn the_g_key_and_a_click_on_the_title_toggle_the_guide_and_save_the_setting() {
    let mut h = sized(KLONDIKE, 140, 40);
    assert!(h.app.settings().guide_panel);
    h.type_char('g');
    assert!(!h.app.settings().guide_panel);
    assert!(!h.text().contains(TITLE));
    let saved = h.storage.borrow().settings.clone();
    assert!(saved.is_some_and(|settings| !settings.guide_panel));
    h.type_char('g');
    assert!(h.app.settings().guide_panel);
    let Some((column, line)) = h.find(TITLE) else {
        unreachable!("the title is drawn")
    };
    h.click(column + 2, line);
    assert!(!h.app.settings().guide_panel);
    assert!(!h.text().contains(TITLE));
}

#[test]
fn the_guide_lists_the_goal_the_keys_and_the_mouse_of_the_game() {
    let mut h = sized(Variant::Spider(SuitCount::Two), 140, 40);
    let panel = panel_rows(&mut h).join("\n");
    for needle in [
        "Build eight runs of one",
        "Arrows",
        "Enter",
        "pick up / drop",
        "deal a row",
        "undo / redo",
        "how to play",
        "all keys",
        "Double-click: quick move",
    ] {
        assert!(panel.contains(needle), "{needle}\n{panel}");
    }
    let mut free = sized(Variant::FreeCell, 140, 40);
    assert!(!panel_rows(&mut free).join("\n").contains("stock"));
    let mut pyramid = sized(Variant::Pyramid(PyramidRules::Classic), 140, 40);
    let text = panel_rows(&mut pyramid).join("\n");
    assert!(text.contains("add up to 13") && text.contains("remove pair"));
    assert!(!text.contains("run size"));
}

#[test]
fn the_guide_marks_the_keys_that_work_right_now() {
    let mut h = sized(KLONDIKE, 140, 40);
    assert!(panel_row(&mut h, "u/r").starts_with("  "));
    first_move_played(&mut h);
    assert!(panel_row(&mut h, "u/r").starts_with("► "));
    assert!(panel_row(&mut h, "Arrows").starts_with("► "));
    assert!(panel_row(&mut h, "Esc").starts_with("  "));
    put_cursor(&mut h, PileRef::Tableau(0));
    h.press(KeyCode::Enter);
    if board(&h).marks().selected.is_some() {
        assert!(panel_row(&mut h, "Esc").starts_with("► "));
    }
}

#[test]
fn the_guide_and_the_action_line_use_only_wgl4_symbols() {
    for variant in Variant::ALL {
        let mut h = sized(variant, 140, 40);
        for ch in h.text().chars().filter(|ch| *ch != '\n') {
            assert!(ch.is_ascii() || is_wgl4(ch), "{variant:?}: {ch:?}");
        }
    }
}

#[test]
fn a_guide_taller_than_the_board_is_cut_without_spilling_over_the_chrome() {
    let mut h = sized(Variant::Spider(SuitCount::Four), 130, 30);
    let lines = h.lines();
    assert_eq!(lines.len(), 30);
    assert!(lines[0].contains("Spider"));
    assert!(lines[29].contains("Esc Back"));
}
