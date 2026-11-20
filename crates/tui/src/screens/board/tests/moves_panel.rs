//! The possible moves panel: its entries against the legal moves on fixed deals, inactive entries,
//! clicks, the toggle key, the Tab order, the window of a Compact screen and the three screen sizes.

use oxysol_core::{DrawCount, Move, PileRef, PyramidRules, SuitCount, Variant};
use ratatui::crossterm::event::KeyCode;

use super::*;
use crate::action::Action;
use crate::app::Modal;
use crate::caps::TerminalCaps;
use crate::i18n::{Locale, set_locale, t};
use crate::screens::Effect;
use crate::settings::Settings;
use crate::test_support::PLAIN;
use crate::widgets::moves::MoveEntry;

const KLONDIKE: Variant = Variant::Klondike(DrawCount::Three);
const SEED: u64 = 5;
const STANDARD: (u16, u16) = (130, 40);
const COMPACT: (u16, u16) = (100, 30);

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
            ..Settings::for_caps(&caps)
        }),
        ..MemoryStorage::default()
    };
    let mut h = Harness::with(storage, caps, width, height);
    h.app.apply(Effect::Start {
        variant,
        seed: Some(SEED),
        undo_allowed: true,
    });
    h.lines();
    h
}

fn english(variant: Variant, size: (u16, u16)) -> Harness {
    shown(variant, size, Locale::EnUs, TRUE_COLOR)
}

fn entries(h: &Harness) -> Vec<MoveEntry> {
    board(h).move_entries()
}

fn entry(h: &Harness, action: Action) -> MoveEntry {
    match entries(h).into_iter().find(|entry| entry.action == action) {
        Some(entry) => entry,
        None => unreachable!("every game lists {action:?}"),
    }
}

fn active(h: &Harness, action: Action) -> bool {
    entry(h, action).active
}

fn click_text(h: &mut Harness, text: &str) {
    let Some((column, row)) = h.find(text) else {
        panic!("{text:?} is not drawn");
    };
    h.click(column + 1, row);
}

fn window_open(h: &Harness) -> bool {
    matches!(h.app.modal(), Some(Modal::Moves(_)))
}

fn assert_entries_match_the_engine(h: &Harness) {
    let game = session(h).game();
    let legal = game.legal_moves();
    assert_eq!(active(h, Action::Stock), legal.contains(&Move::Stock));
    assert_eq!(active(h, Action::Hint), game.hint().is_some());
    assert_eq!(active(h, Action::Undo), session(h).can_undo());
    assert_eq!(active(h, Action::Redo), session(h).can_redo());
    assert!(active(h, Action::Restart));
    assert!(active(h, Action::NewGame));
    assert!(!active(h, Action::Back));
    let cursor = board(h).cursor.pile;
    let pyramid = matches!(session(h).variant(), Variant::Pyramid(_));
    if active(h, Action::Select) && cursor != PileRef::Stock && !pyramid {
        assert!(legal.iter().any(|mv| mv.source() == Some(cursor)));
    }
}

#[test]
fn the_entries_match_the_legal_moves_on_fixed_deals_through_a_whole_game() {
    for variant in GAMES {
        let mut h = english(variant, STANDARD);
        assert_entries_match_the_engine(&h);
        for mv in hint_moves(variant, SEED).into_iter().take(40) {
            assert_eq!(h.app.board_mut().play(mv), Effect::None);
            assert_entries_match_the_engine(&h);
        }
    }
}

#[test]
fn undo_and_redo_follow_the_history() {
    let mut h = english(KLONDIKE, STANDARD);
    assert!(!active(&h, Action::Undo) && !active(&h, Action::Redo));
    h.type_char('s');
    assert!(active(&h, Action::Undo) && !active(&h, Action::Redo));
    h.type_char('u');
    assert!(!active(&h, Action::Undo) && active(&h, Action::Redo));
}

#[test]
fn a_game_without_undo_lists_both_as_inactive_with_the_reason() {
    let mut h = start_no_undo(KLONDIKE, SEED);
    h.resize(STANDARD.0, STANDARD.1);
    h.type_char('s');
    for action in [Action::Undo, Action::Redo] {
        let item = entry(&h, action);
        assert!(!item.active);
        assert_eq!(item.reason.as_deref(), Some("Undo is off"));
    }
}

#[test]
fn carrying_cards_offers_drop_and_put_back_and_blocks_the_rest() {
    let mut h = english(KLONDIKE, STANDARD);
    let from = legal(&h)
        .into_iter()
        .find_map(|mv| mv.source().filter(|pile| *pile != PileRef::Stock));
    put_cursor(&mut h, from.unwrap_or(PileRef::Tableau(0)));
    h.press(KeyCode::Enter);
    assert!(active(&h, Action::Back));
    assert_eq!(entry(&h, Action::Select).label, "Drop");
    assert!(!active(&h, Action::Stock));
    assert_eq!(
        entry(&h, Action::Stock).reason.as_deref(),
        Some("Carrying cards")
    );
}

#[test]
fn inactive_entries_are_marked_with_a_sign_and_a_reason_without_color() {
    let mut h = english(KLONDIKE, (200, 60));
    let text = h.text();
    assert!(text.contains("× Undo"), "{text}");
    assert!(text.contains("Nothing to undo"), "{text}");
    assert!(text.contains("• Hint"), "{text}");
    let mut plain = shown(KLONDIKE, (200, 60), Locale::EnUs, PLAIN);
    let text = plain.text();
    assert!(text.contains("x Undo"), "{text}");
    assert!(text.contains("o Hint"), "{text}");
}

#[test]
fn a_click_on_an_inactive_entry_does_nothing() {
    let mut h = english(KLONDIKE, STANDARD);
    let before = (moves_made(&h), cursor(&h));
    click_text(&mut h, "× Undo");
    h.lines();
    assert_eq!((moves_made(&h), cursor(&h)), before);
    assert!(h.app.modal().is_none());
}

#[test]
fn a_click_on_an_active_entry_runs_the_move() {
    let mut h = english(KLONDIKE, STANDARD);
    let stock_before = pile_len(&h, PileRef::Stock);
    click_text(&mut h, "• Draw card");
    assert_eq!(moves_made(&h), 1);
    assert!(pile_len(&h, PileRef::Stock) < stock_before);
    click_text(&mut h, "• Undo");
    assert_eq!(moves_made(&h), 0);
}

#[test]
fn enter_on_an_inactive_entry_says_so_and_enter_on_an_active_one_runs_it() {
    let mut h = english(KLONDIKE, STANDARD);
    let last = board(&h).piles().len();
    for _ in 0..last {
        h.press(KeyCode::Tab);
        if board(&h).panel_focus.is_some() {
            break;
        }
    }
    assert_eq!(board(&h).panel_focus, Some(0));
    assert!(active(&h, Action::Stock));
    h.press(KeyCode::Enter);
    assert_eq!(moves_made(&h), 1);
    for _ in 0..4 {
        h.press(KeyCode::Down);
    }
    assert_eq!(board(&h).panel_focus, Some(4));
    assert!(!active(&h, Action::Back) || entries(&h)[4].action == Action::Hint);
    h.press(KeyCode::Down);
    h.press(KeyCode::Down);
    assert_eq!(entries(&h)[6].action, Action::Redo);
    h.press(KeyCode::Enter);
    assert_eq!(status(&h), Some("board.status.not_possible"));
    assert_eq!(moves_made(&h), 1);
}

#[test]
fn tab_goes_from_the_last_pile_through_the_panel_and_wraps_to_the_first_pile() {
    let mut h = english(KLONDIKE, STANDARD);
    let piles = board(&h).piles().len();
    let count = entries(&h).len();
    put_cursor(&mut h, PileRef::Tableau(6));
    h.press(KeyCode::Tab);
    assert_eq!(board(&h).panel_focus, Some(0));
    for index in 1..count {
        h.press(KeyCode::Tab);
        assert_eq!(board(&h).panel_focus, Some(index));
    }
    h.press(KeyCode::Tab);
    assert_eq!(board(&h).panel_focus, None);
    assert_eq!(cursor(&h).0, PileRef::Stock);
    assert!(piles > 0);
    h.send(crate::input::Input::Key(
        ratatui::crossterm::event::KeyEvent::new(
            KeyCode::BackTab,
            ratatui::crossterm::event::KeyModifiers::SHIFT,
        ),
    ));
    assert_eq!(board(&h).panel_focus, Some(count - 1));
}

#[test]
fn the_focused_entry_shows_the_focus_marker() {
    let mut h = english(KLONDIKE, STANDARD);
    put_cursor(&mut h, PileRef::Tableau(6));
    h.press(KeyCode::Tab);
    let text = h.text();
    assert!(text.contains("►• Draw card"), "{text}");
}

#[test]
fn without_the_panel_tab_stays_on_the_piles() {
    let mut h = english(KLONDIKE, COMPACT);
    put_cursor(&mut h, PileRef::Tableau(6));
    h.press(KeyCode::Tab);
    assert_eq!(board(&h).panel_focus, None);
    assert_eq!(cursor(&h).0, PileRef::Stock);
}

#[test]
fn the_toggle_key_hides_and_shows_the_panel_for_this_game_only() {
    let mut h = english(KLONDIKE, STANDARD);
    assert!(h.text().contains("Possible moves (v: hide)"));
    h.type_char('v');
    assert!(!h.app.aids().moves_panel);
    assert!(!h.text().contains("Possible moves"));
    assert!(h.app.aids().guide_panel);
    h.type_char('v');
    assert!(h.app.aids().moves_panel);
    assert!(h.text().contains("Possible moves (v: hide)"));
}

#[test]
fn a_click_on_the_panel_title_hides_it() {
    let mut h = english(KLONDIKE, STANDARD);
    click_text(&mut h, "Possible moves");
    assert!(!h.app.aids().moves_panel);
}

#[test]
fn the_panel_works_with_the_guide_panel_off() {
    let mut h = english(KLONDIKE, STANDARD);
    h.type_char('g');
    assert!(!h.app.aids().guide_panel);
    let text = h.text();
    assert!(text.contains("Possible moves"), "{text}");
    assert!(!text.contains("Guide ("), "{text}");
    click_text(&mut h, "• Draw card");
    assert_eq!(moves_made(&h), 1);
}

#[test]
fn the_panel_renders_at_the_three_sizes_and_both_locales() {
    for (size, locale, title, undo) in [
        (STANDARD, Locale::EnUs, "Possible moves (v: hide)", "Undo"),
        ((200, 60), Locale::EnUs, "Possible moves (v: hide)", "Undo"),
        (STANDARD, Locale::PlPl, "Możliwe ruchy (v: ukryj)", "Cofnij"),
        (
            (200, 60),
            Locale::PlPl,
            "Możliwe ruchy (v: ukryj)",
            "Cofnij",
        ),
    ] {
        let mut h = shown(KLONDIKE, size, locale, TRUE_COLOR);
        let text = h.text();
        assert!(text.contains(title), "{size:?} {locale:?}\n{text}");
        assert!(text.contains(undo), "{size:?} {locale:?}\n{text}");
    }
    set_locale(Locale::EnUs);
}

#[test]
fn a_tall_screen_shows_the_reasons_and_a_short_one_leaves_them_out() {
    let mut tall = english(KLONDIKE, (200, 60));
    assert!(tall.text().contains("Nothing to undo"));
    let mut short = english(KLONDIKE, STANDARD);
    assert!(!short.text().contains("Nothing to undo"));
    let mut polish = shown(KLONDIKE, (200, 60), Locale::PlPl, TRUE_COLOR);
    assert!(polish.text().contains("Nic do cofnięcia"));
    set_locale(Locale::EnUs);
}

#[test]
fn a_compact_screen_opens_the_window_with_the_key_and_lists_every_entry() {
    for locale in Locale::offered() {
        let mut h = shown(KLONDIKE, COMPACT, locale, TRUE_COLOR);
        assert!(!h.text().contains("•"), "no side panel at 100 columns");
        h.type_char('v');
        assert!(window_open(&h));
        let text = h.text();
        set_locale(locale);
        let title = t("board.moves.window_title").into_owned();
        assert!(text.contains(&title), "{text}");
        assert!(text.contains("• ") && text.contains("× "), "{text}");
        h.press(KeyCode::Esc);
        assert!(h.app.modal().is_none());
    }
    set_locale(Locale::EnUs);
}

#[test]
fn the_window_runs_an_active_entry_by_enter_and_by_click_and_ignores_an_inactive_one() {
    let mut h = english(KLONDIKE, COMPACT);
    h.type_char('v');
    h.press(KeyCode::Enter);
    assert!(!window_open(&h));
    assert_eq!(moves_made(&h), 1);

    h.type_char('v');
    click_text(&mut h, "× Redo");
    assert!(window_open(&h));
    assert_eq!(moves_made(&h), 1);
    click_text(&mut h, "• Undo");
    assert!(!window_open(&h));
    assert_eq!(moves_made(&h), 0);
}

#[test]
fn the_window_footer_names_the_close_key_and_its_button_closes_it() {
    let mut h = english(KLONDIKE, COMPACT);
    h.type_char('v');
    assert!(h.text().contains("v/Esc: close"));
    click_text(&mut h, "v/Esc: close");
    assert!(h.app.modal().is_none());
}

#[test]
fn the_compact_do_row_keeps_working_and_the_key_is_in_help() {
    let mut h = english(KLONDIKE, COMPACT);
    h.type_char('?');
    assert!(h.text().contains("Possible moves panel"));
}

#[test]
fn spider_and_freecell_name_their_own_stock_entry() {
    let spider = english(Variant::Spider(SuitCount::Two), STANDARD);
    assert_eq!(entry(&spider, Action::Stock).label, "Deal a row");
    let mut klondike = english(KLONDIKE, STANDARD);
    while pile_len(&klondike, PileRef::Stock) > 0 {
        klondike.type_char('s');
    }
    assert_eq!(entry(&klondike, Action::Stock).label, "Turn waste over");
}
