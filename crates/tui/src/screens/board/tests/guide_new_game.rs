//! The guide panel lists `n` as new game as the last row of `Keys`, the first row a short panel
//! drops: it shows on a tall panel, and on a short one the other sections keep their room.

use oxysol_core::{DrawCount, Variant};

use super::footer::board_game;
use crate::i18n::{Locale, set_locale, t};

const KLONDIKE: Variant = Variant::Klondike(DrawCount::One);

fn listed(lines: &[String], label: &str) -> bool {
    lines
        .iter()
        .any(|line| line.contains(label) && line.contains(" n "))
}

#[test]
fn the_keys_section_lists_n_as_new_game_last_on_a_tall_panel_in_every_locale() {
    for locale in Locale::offered() {
        let mut h = board_game(locale, KLONDIKE, 5, 200, 60);
        let lines = h.lines();
        let label = t("board.guide.new_game").into_owned();
        let help = t("board.guide.help").into_owned();
        let Some(new_game) = lines
            .iter()
            .position(|line| line.contains(&label) && line.contains(" n "))
        else {
            panic!("{locale:?}: no `n` row for {label:?}\n{}", lines.join("\n"));
        };
        let Some(all_keys) = lines.iter().position(|line| line.contains(&help)) else {
            panic!("{locale:?}: no row for {help:?}\n{}", lines.join("\n"));
        };
        assert_eq!(
            new_game,
            all_keys + 1,
            "{locale:?}: the `n` row follows the last key row\n{}",
            lines.join("\n")
        );
    }
    set_locale(Locale::EnUs);
}

#[test]
fn a_short_panel_drops_the_n_row_first_and_keeps_its_keys_heading() {
    for locale in Locale::offered() {
        let mut h = board_game(locale, KLONDIKE, 5, 130, 30);
        let lines = h.lines();
        let label = t("board.guide.new_game").into_owned();
        assert!(
            !listed(&lines, &label),
            "{locale:?}: the `n` row is the first one a short panel drops\n{}",
            lines.join("\n")
        );
        let keys = t("board.guide.keys_title").into_owned();
        assert!(
            lines.iter().any(|line| line.contains(&keys)),
            "{locale:?}: the Keys section is gone\n{}",
            lines.join("\n")
        );
    }
    set_locale(Locale::EnUs);
}
