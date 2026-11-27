use crate::i18n::{self, Locale, t};
use crate::widgets::overlay::help::{Row, sections};

/// The keys the game list reads itself (`screens/game_select.rs`: the page keys, `/`, `w` and the
/// fold keys), named as the Game menu rows name them. Enter and Space fold a heading.
const KEYS_THE_MENU_HANDLES: [&str; 10] = [
    "PgUp", "PgDn", "Home", "End", "/", "w", "Left", "Right", "Enter", "Space",
];

fn game_menu_rows() -> Vec<Row> {
    sections()
        .into_iter()
        .find(|section| section.title == t("help.group.game_menu"))
        .map(|section| section.rows)
        .unwrap_or_default()
}

#[test]
fn every_key_the_game_menu_handles_itself_has_a_row_in_the_game_menu_section() {
    i18n::set_locale(Locale::EnUs);
    let rows = game_menu_rows();
    assert!(!rows.is_empty(), "the Game menu section exists");
    for key in KEYS_THE_MENU_HANDLES {
        let listed = rows
            .iter()
            .any(|row| row.keys.split(" / ").any(|part| part == key));
        assert!(listed, "no Game menu row names {key}");
    }
    i18n::set_locale(Locale::EnUs);
}
