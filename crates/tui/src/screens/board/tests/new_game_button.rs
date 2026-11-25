//! The new game button in the `Game` box of the side panel: drawn at 130 columns and wider with the
//! panel shown, and a click asks for the new game like `n`. Below 130 columns there is no button,
//! and the moves window (`v`) still offers new game.

use oxysol_core::{DrawCount, Variant};
use ratatui::crossterm::event::KeyCode;

use super::footer::board_game;
use crate::action::Action;
use crate::app::Modal;
use crate::i18n::{Locale, set_locale, t, tf};
use crate::input::primary_key;

const KLONDIKE: Variant = Variant::Klondike(DrawCount::One);
const PANEL_SIZES: [(u16, u16); 3] = [(130, 30), (130, 40), (200, 60)];
const PANEL_START: u16 = 100;

fn button_text() -> String {
    let key = primary_key(Action::NewGame).unwrap_or_default();
    let label = t("board.guide.new_game");
    tf(
        "board.guide.new_game_button",
        &[("key", &key), ("label", &label)],
    )
}

fn asks_new_game(modal: Option<&Modal>) -> bool {
    matches!(
        modal,
        Some(Modal::Confirm { prompt, .. }) if *prompt == "board.confirm.new_game"
    )
}

#[test]
fn the_game_box_button_is_drawn_in_the_panel_at_every_size_and_locale() {
    for locale in Locale::offered() {
        for (width, height) in PANEL_SIZES {
            let mut h = board_game(locale, KLONDIKE, 5, width, height);
            let text = button_text();
            let Some((column, _)) = h.find(&text) else {
                panic!(
                    "{locale:?} {width}x{height}: {text:?} is not drawn\n{}",
                    h.text()
                );
            };
            assert!(
                column >= PANEL_START,
                "{locale:?} {width}x{height}: the button is on the board, not in the panel"
            );
        }
    }
    set_locale(Locale::EnUs);
}

#[test]
fn a_click_on_the_game_box_button_asks_for_a_new_game_like_n() {
    let mut by_key = board_game(Locale::EnUs, KLONDIKE, 5, 130, 30);
    by_key.type_char('n');
    assert!(asks_new_game(by_key.app.modal()));

    let mut by_click = board_game(Locale::EnUs, KLONDIKE, 5, 130, 30);
    let text = button_text();
    let Some((column, row)) = by_click.find(&text) else {
        panic!("{text:?} is not drawn\n{}", by_click.text());
    };
    by_click.click(column + 1, row);
    assert!(asks_new_game(by_click.app.modal()));
    by_click.press(KeyCode::Right);
    by_click.press(KeyCode::Enter);
    assert!(by_click.app.modal().is_none());
    set_locale(Locale::EnUs);
}

#[test]
fn below_130_columns_no_game_box_button_is_drawn_and_v_still_offers_new_game() {
    let mut h = board_game(Locale::EnUs, KLONDIKE, 5, 100, 30);
    assert!(h.find(&button_text()).is_none(), "{}", h.text());
    h.type_char('v');
    assert!(matches!(h.app.modal(), Some(Modal::Moves(_))));
    let label = t("board.moves.label.new_game").into_owned();
    assert!(h.text().contains(&label));
    set_locale(Locale::EnUs);
}

#[test]
fn the_help_window_lists_n_as_new_game() {
    let mut h = board_game(Locale::EnUs, KLONDIKE, 5, 130, 30);
    h.type_char('?');
    let label = t("help.action.new_game").into_owned();
    assert!(h.text().contains(&label), "{}", h.text());
    set_locale(Locale::EnUs);
}
