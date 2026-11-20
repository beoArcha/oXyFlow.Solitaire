use oxysol_core::Variant;
use ratatui::crossterm::event::KeyCode;

use super::catalog::Family;
use crate::app::Screen;
use crate::i18n::t;
use crate::screens::focus::marked_text;
use crate::screens::variant_label_key;
use crate::test_support::Harness;

fn label_of(variant: Variant) -> String {
    t(variant_label_key(variant)).into_owned()
}

fn select() -> Harness {
    let mut h = Harness::new();
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::GameSelect);
    h
}

fn type_text(h: &mut Harness, text: &str) {
    for c in text.chars() {
        h.type_char(c);
    }
}

fn shown(h: &mut Harness) -> Vec<Variant> {
    let text = h.text();
    Variant::ALL
        .into_iter()
        .filter(|variant| text.contains(&label_of(*variant)))
        .collect()
}

fn focused_label(h: &mut Harness) -> Option<String> {
    marked_text(&h.lines(), "►").and_then(|text| {
        text.splitn(3, ' ')
            .nth(2)
            .map(|label| label.trim_start().to_owned())
    })
}

fn first_word(variant: Variant) -> String {
    label_of(variant)
        .split([' ', ','])
        .next()
        .unwrap_or_default()
        .to_lowercase()
}

#[test]
fn the_slash_key_opens_the_search_field() {
    let mut h = select();
    assert!(
        h.text()
            .contains(&format!("[ / {} ]", t("menu.search.label")))
    );
    h.type_char('/');
    assert!(h.text().contains(&t("menu.search.clear").into_owned()));
    assert!(h.app.settings().favourites.is_empty());
}

#[test]
fn typing_filters_the_list_by_game_name_and_variant() {
    let mut h = select();
    h.type_char('/');
    let word = first_word(Variant::ALL[0]);
    type_text(&mut h, &word);
    let expected: Vec<Variant> = Variant::ALL
        .into_iter()
        .filter(|variant| label_of(*variant).to_lowercase().contains(&word))
        .collect();
    assert_eq!(shown(&mut h), expected);
    assert!(!expected.is_empty());
    let narrow = label_of(Variant::ALL[0]).to_lowercase();
    type_text(&mut h, &narrow[word.len()..]);
    assert_eq!(shown(&mut h), vec![Variant::ALL[0]]);
}

#[test]
fn letters_that_are_shortcuts_are_typed_while_the_search_is_open() {
    let mut h = select();
    h.type_char('/');
    type_text(&mut h, "fu");
    assert!(h.app.settings().favourites.is_empty());
    assert!(h.app.settings().undo_allowed);
    assert!(h.text().contains(": fu_"));
}

#[test]
fn the_search_ignores_case_and_headings_without_matches_disappear() {
    let mut h = select();
    h.type_char('/');
    let word = first_word(Variant::ALL[0]).to_uppercase();
    type_text(&mut h, &word);
    assert!(!shown(&mut h).is_empty());
    let text = h.text();
    let family_titles: Vec<String> = Family::ALL
        .into_iter()
        .map(|family| t(family.title_key()).into_owned())
        .collect();
    let drawn = family_titles
        .iter()
        .filter(|title| text.contains(title.as_str()))
        .count();
    assert_eq!(drawn, 1);
}

#[test]
fn a_search_without_a_match_says_so_and_enter_starts_nothing() {
    let mut h = select();
    h.type_char('/');
    type_text(&mut h, "zzzz");
    assert!(shown(&mut h).is_empty());
    assert!(h.text().contains("No game matches: zzzz"));
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::GameSelect);
}

#[test]
fn backspace_widens_the_filter_again() {
    let mut h = select();
    h.type_char('/');
    type_text(&mut h, "zz");
    assert!(shown(&mut h).is_empty());
    h.press(KeyCode::Backspace);
    h.press(KeyCode::Backspace);
    assert_eq!(shown(&mut h), Variant::ALL.to_vec());
}

#[test]
fn the_first_escape_clears_the_search_and_the_second_leaves() {
    let mut h = select();
    h.type_char('/');
    type_text(&mut h, "zzzz");
    h.press(KeyCode::Esc);
    assert_eq!(h.app.screen(), Screen::GameSelect);
    assert_eq!(shown(&mut h), Variant::ALL.to_vec());
    assert!(
        h.text()
            .contains(&format!("[ / {} ]", t("menu.search.label")))
    );
    h.press(KeyCode::Esc);
    assert_eq!(h.app.screen(), Screen::MainMenu);
}

#[test]
fn an_empty_open_search_also_closes_with_the_first_escape() {
    let mut h = select();
    h.type_char('/');
    h.press(KeyCode::Esc);
    assert_eq!(h.app.screen(), Screen::GameSelect);
    h.press(KeyCode::Esc);
    assert_eq!(h.app.screen(), Screen::MainMenu);
}

#[test]
fn enter_starts_the_first_match() {
    let mut h = select();
    let target = Variant::ALL[Variant::ALL.len() - 1];
    h.type_char('/');
    type_text(&mut h, &label_of(target));
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Board);
    assert_eq!(h.app.board().session().map(|s| s.variant()), Some(target));
}

#[test]
fn the_arrow_keys_move_inside_the_filtered_list() {
    let mut h = select();
    h.type_char('/');
    let word = first_word(Variant::ALL[0]);
    type_text(&mut h, &word);
    let matches: Vec<Variant> = Variant::ALL
        .into_iter()
        .filter(|variant| label_of(*variant).to_lowercase().contains(&word))
        .collect();
    h.press(KeyCode::Down);
    let second = matches.get(1).copied().unwrap_or(matches[0]);
    assert_eq!(focused_label(&mut h), Some(label_of(second)));
}

#[test]
fn a_click_on_the_field_opens_the_search_and_a_click_on_a_match_starts_it() {
    let mut h = select();
    let field = format!("[ / {} ]", t("menu.search.label"));
    let (column, row) = h.find(&field).unwrap_or_default();
    h.click(column + 2, row);
    assert!(h.text().contains(&t("menu.search.clear").into_owned()));
    let target = Variant::ALL[Variant::ALL.len() - 1];
    type_text(&mut h, &label_of(target));
    let (column, row) = h
        .find(&format!("·   {}", label_of(target)))
        .unwrap_or_default();
    h.click(column + 4, row);
    assert_eq!(h.app.screen(), Screen::Board);
    assert_eq!(h.app.board().session().map(|s| s.variant()), Some(target));
}

#[test]
fn a_favourite_set_by_click_inside_a_search_keeps_the_filter_and_the_focus() {
    let mut h = select();
    h.type_char('/');
    let target = Variant::ALL[Variant::ALL.len() - 1];
    type_text(&mut h, &label_of(target));
    let (column, row) = h
        .find(&format!("·   {}", label_of(target)))
        .unwrap_or_default();
    h.click(column + 2, row);
    assert_eq!(h.app.screen(), Screen::GameSelect);
    assert_eq!(h.app.settings().favourites, vec![target]);
    assert_eq!(shown(&mut h), vec![target]);
    assert_eq!(focused_label(&mut h), Some(label_of(target)));
}

#[test]
fn escape_keeps_the_focused_game_so_that_f_can_mark_it() {
    let mut h = select();
    h.type_char('/');
    let target = Variant::ALL[Variant::ALL.len() - 1];
    type_text(&mut h, &label_of(target));
    h.press(KeyCode::Esc);
    assert_eq!(focused_label(&mut h), Some(label_of(target)));
    h.type_char('f');
    assert_eq!(h.app.settings().favourites, vec![target]);
}

#[test]
fn the_shortcuts_work_again_when_the_focus_leaves_the_field() {
    let mut h = select();
    h.type_char('/');
    h.press(KeyCode::Tab);
    h.type_char('u');
    assert!(!h.app.settings().undo_allowed);
}

#[test]
fn the_search_works_in_the_games_of_a_seed() {
    let mut h = Harness::starting_at_seed(7);
    h.type_char('/');
    type_text(&mut h, "zzzz");
    assert!(shown(&mut h).is_empty());
    h.press(KeyCode::Esc);
    assert_eq!(h.app.screen(), Screen::MainMenu);
    assert_eq!(shown(&mut h), Variant::ALL.to_vec());
    h.press(KeyCode::Esc);
    assert!(!h.text().contains(&t("menu.search.label").into_owned()));
}
