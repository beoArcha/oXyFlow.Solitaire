use super::seed::parse_seed;
use crate::app::{Modal, Screen};
use crate::i18n::Locale;
use crate::settings::Settings;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, PLAIN, TRUE_COLOR};
use oxysol_core::{Session, Variant};
use ratatui::crossterm::event::KeyCode;

fn with_save() -> MemoryStorage {
    MemoryStorage {
        game: Some(crate::storage::SavedGame {
            id: crate::storage::GameId(3),
            record: Session::new(Variant::FreeCell, 5).record(),
            elapsed_ms: 1_000,
            undo_allowed: true,
        }),
        ..MemoryStorage::new()
    }
}

fn type_text(h: &mut Harness, text: &str) {
    for c in text.chars() {
        h.type_char(c);
    }
}

/// Opens "New game from seed": the game list comes first.
fn open_game_list(h: &mut Harness) {
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
}

/// The typing page of the first game: the game list, the first game, then "Type a seed".
fn open_typing(h: &mut Harness) {
    open_game_list(h);
    h.press(KeyCode::Enter);
    h.press(KeyCode::Enter);
}

#[test]
fn every_item_is_reached_by_keyboard() {
    let mut h = Harness::new();
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::GameSelect);
    h.press(KeyCode::Esc);
    open_game_list(&mut h);
    assert!(h.text().contains("New game from seed"));
    h.press(KeyCode::Esc);
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Stats);
    h.press(KeyCode::Esc);
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Settings);
    h.press(KeyCode::Esc);
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
    assert!(matches!(h.app.modal(), Some(Modal::HowToPlay(_))));
    h.press(KeyCode::Esc);
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
    assert!(matches!(h.app.modal(), Some(Modal::Help(_))));
    h.press(KeyCode::Esc);
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
    assert!(h.app.should_quit());
}

#[test]
fn every_item_is_reached_by_mouse() {
    for label in [
        "New game from seed",
        "Statistics",
        "Settings",
        "How to play",
        "Help",
        "Quit",
    ] {
        let mut h = Harness::new();
        let (column, row) = h.find(label).unwrap_or_default();
        h.click(column, row);
        match label {
            "Statistics" => assert_eq!(h.app.screen(), Screen::Stats),
            "Settings" => assert_eq!(h.app.screen(), Screen::Settings),
            "How to play" => assert!(matches!(h.app.modal(), Some(Modal::HowToPlay(_)))),
            "Help" => assert!(matches!(h.app.modal(), Some(Modal::Help(_)))),
            "Quit" => assert!(h.app.should_quit()),
            _ => assert!(h.text().contains("choose a game")),
        }
    }
    let mut h = Harness::new();
    let (column, row) = h.find("New game").unwrap_or_default();
    h.click(column, row);
    assert_eq!(h.app.screen(), Screen::GameSelect);
}

#[test]
fn resume_is_hidden_without_a_save_and_first_with_one() {
    let mut without = Harness::new();
    assert!(!without.text().contains("Resume"));
    let mut with = Harness::with(with_save(), TRUE_COLOR, 120, 40);
    assert!(with.text().contains("Resume"));
    with.press(KeyCode::Enter);
    assert_eq!(with.app.screen(), Screen::Board);
    assert!(with.app.board().session().is_some_and(|s| s.seed() == 5));
}

#[test]
fn resume_moves_the_other_items_down_by_one() {
    let mut h = Harness::with(with_save(), TRUE_COLOR, 120, 40);
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::GameSelect);
}

#[test]
fn a_typed_seed_reaches_the_session_for_every_variant() {
    for (index, variant) in Variant::ALL.into_iter().enumerate() {
        let mut h = Harness::new();
        open_game_list(&mut h);
        for _ in 0..index {
            h.press(KeyCode::Down);
        }
        h.press(KeyCode::Enter);
        assert!(h.text().contains("Type a seed"));
        h.press(KeyCode::Enter);
        type_text(&mut h, "12345");
        h.press(KeyCode::Enter);
        assert_eq!(h.app.screen(), Screen::Board);
        assert!(
            h.app
                .board()
                .session()
                .is_some_and(|s| s.variant() == variant && s.seed() == 12_345),
            "{variant:?}"
        );
        let history = h.storage.borrow().history.clone();
        assert_eq!((history[0].variant, history[0].seed), (variant, 12_345));
    }
}

#[test]
fn the_largest_seed_is_accepted() {
    let mut h = Harness::new();
    open_typing(&mut h);
    type_text(&mut h, "18446744073709551615");
    h.press(KeyCode::Enter);
    assert!(
        h.app
            .board()
            .session()
            .is_some_and(|s| s.seed() == u64::MAX)
    );
}

#[test]
fn a_non_numeric_seed_is_refused_with_a_message() {
    let mut h = Harness::new();
    open_typing(&mut h);
    type_text(&mut h, "4x2");
    h.press(KeyCode::Enter);
    assert!(h.text().contains("A seed is a whole number"));
    assert!(h.text().contains("New game from seed"));
    assert!(h.app.board().session().is_none());
    assert!(h.storage.borrow().history.is_empty());
}

#[test]
fn an_empty_or_out_of_range_seed_is_refused_with_a_message() {
    let mut h = Harness::new();
    open_typing(&mut h);
    h.press(KeyCode::Enter);
    assert!(h.text().contains("Type a seed first."));
    type_text(&mut h, "18446744073709551616");
    h.press(KeyCode::Enter);
    assert!(h.text().contains("This seed is too large"));
    assert!(h.app.board().session().is_none());
}

#[test]
fn backspace_deletes_a_character_and_clears_the_message() {
    let mut h = Harness::new();
    open_typing(&mut h);
    type_text(&mut h, "7a");
    h.press(KeyCode::Enter);
    assert!(h.text().contains("A seed is a whole number"));
    h.press(KeyCode::Backspace);
    assert!(!h.text().contains("A seed is a whole number"));
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Board);
    assert!(h.app.board().session().is_some_and(|s| s.seed() == 7));
}

#[test]
fn escape_goes_back_one_page_at_a_time() {
    let mut h = Harness::new();
    open_typing(&mut h);
    type_text(&mut h, "99");
    h.press(KeyCode::Esc);
    assert!(h.text().contains("Difficulty levels"));
    assert!(!h.text().contains("99"));
    h.press(KeyCode::Esc);
    assert!(h.text().contains("choose a game"));
    h.press(KeyCode::Esc);
    assert!(h.text().contains("New game"));
    assert!(!h.text().contains("choose a game"));
    assert_eq!(h.app.screen(), Screen::MainMenu);
}

#[test]
fn seed_buttons_work_by_mouse() {
    let mut h = Harness::new();
    open_typing(&mut h);
    type_text(&mut h, "77");
    let (column, row) = h.find("Continue").unwrap_or_default();
    h.click(column, row);
    assert_eq!(h.app.screen(), Screen::Board);
    assert!(h.app.board().session().is_some_and(|s| s.seed() == 77));
    let mut back = Harness::new();
    open_typing(&mut back);
    let (column, row) = back.find("Back").unwrap_or_default();
    back.click(column, row);
    assert!(back.text().contains("Difficulty levels"));
    assert!(!back.text().contains("Continue"));
}

#[test]
fn an_invalid_seed_is_refused_by_the_continue_button() {
    let mut h = Harness::new();
    open_typing(&mut h);
    let (column, row) = h.find("Continue").unwrap_or_default();
    h.click(column, row);
    assert!(h.text().contains("Type a seed first."));
    assert!(h.text().contains("Continue"));
}

#[test]
fn seed_entry_renders_in_polish_and_in_ascii() {
    let storage = MemoryStorage {
        settings: Some(Settings {
            locale: Locale::PlPl,
            ..Settings::for_caps(&PLAIN)
        }),
        ..MemoryStorage::new()
    };
    let mut h = Harness::with(storage, PLAIN, 120, 40);
    open_typing(&mut h);
    let text = h.text();
    let template = crate::i18n::t("menu.seed.title_game").into_owned();
    let prefix = template.split('{').next().unwrap_or_default().trim_end();
    assert!(text.contains(prefix), "{text}");
    assert!(
        text.chars()
            .all(|ch| ch.is_ascii() || crate::glyphs::is_wgl4(ch))
    );
}

#[test]
fn the_polish_seed_item_fits_at_100_by_30() {
    let storage = MemoryStorage {
        settings: Some(Settings {
            locale: Locale::PlPl,
            ..Settings::for_caps(&PLAIN)
        }),
        ..MemoryStorage::new()
    };
    let mut h = Harness::with(storage, PLAIN, 100, 30);
    let label = crate::i18n::t("menu.new_game_from_seed").into_owned();
    let text = h.text();
    assert!(text.contains(&label), "{label}\n{text}");
    crate::i18n::set_locale(Locale::EnUs);
}

#[test]
fn the_seed_parser_accepts_digits_and_names_each_refusal() {
    use super::seed::SeedError;
    assert_eq!(parse_seed("0"), Ok(0));
    assert_eq!(parse_seed("00042"), Ok(42));
    assert_eq!(parse_seed(""), Err(SeedError::Empty));
    assert_eq!(parse_seed("-1"), Err(SeedError::NotNumeric));
    assert_eq!(parse_seed("1 2"), Err(SeedError::NotNumeric));
    assert_eq!(
        parse_seed("18446744073709551616"),
        Err(SeedError::OutOfRange)
    );
}

#[test]
fn the_menu_shows_its_items_again_after_a_game_from_a_seed() {
    let mut h = Harness::new();
    open_typing(&mut h);
    type_text(&mut h, "7");
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Board);
    h.press(KeyCode::Esc);
    assert_eq!(h.app.screen(), Screen::MainMenu);
    let text = h.text();
    assert!(text.contains("New game from seed"), "{text}");
    assert!(!text.contains("Type a seed"), "{text}");
}

#[test]
fn the_menu_shows_its_tagline_and_version_in_both_locales() {
    for locale in Locale::offered() {
        let storage = MemoryStorage {
            settings: Some(Settings {
                locale,
                ..Settings::default()
            }),
            ..MemoryStorage::new()
        };
        let mut h = Harness::with(storage, TRUE_COLOR, 100, 30);
        let tagline = crate::i18n::t("common.app.tagline").into_owned();
        let version = format!("v{}", crate::test_support::TEST_VERSION);
        let text = h.text();
        assert!(text.contains(&tagline), "{text}");
        assert!(text.contains(&version), "{text}");
        crate::i18n::set_locale(Locale::EnUs);
    }
}
