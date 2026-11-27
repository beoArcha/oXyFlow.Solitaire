//! "New game from seed" through the whole app, by keyboard and by mouse: the game first, then the
//! seed screen of that game (typed, or a curated level), and every locale at the minimum sizes.
//! Which variants have curated seeds depends on the shipped table, so a test that expects the empty
//! message picks a variant the table does not have.

use ratatui::crossterm::event::KeyCode;

use crate::app::Screen;
use crate::i18n::{Locale, set_locale, t, tf};
use crate::screens::variant_label_key;
use crate::settings::Settings;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, PLAIN};
use oxysol_core::curated::{self, Difficulty};
use oxysol_core::{GameOptions, StockPasses, Variant, YukonRules};

/// Golf with the wrap rule: options of the variant, so the levels are not offered.
const GOLF_WRAP: GameOptions = GameOptions {
    golf_wrap: true,
    ..GameOptions::DEFAULT
};

/// Klondike with one stock pass: options of another game, so the Golf levels stay offered.
const KLONDIKE_ONE_PASS: GameOptions = GameOptions {
    klondike_passes: StockPasses::One,
    ..GameOptions::DEFAULT
};

fn storage_with_options(locale: Locale, options: GameOptions) -> MemoryStorage {
    MemoryStorage {
        settings: Some(Settings {
            locale,
            game_options: options,
            ..Settings::for_caps(&PLAIN)
        }),
        ..MemoryStorage::new()
    }
}

fn locale_storage(locale: Locale) -> MemoryStorage {
    storage_with_options(locale, GameOptions::DEFAULT)
}

/// The seed options of `variant` under `options`, at 100 × 30 without colour.
fn seed_options_under(variant: Variant, options: GameOptions) -> Harness {
    let mut h = Harness::with(storage_with_options(Locale::EnUs, options), PLAIN, 100, 30);
    open_game_list(&mut h);
    h.focus_game_variant(variant);
    h.press(KeyCode::Enter);
    h
}

fn game_title(variant: Variant) -> String {
    let game = t(variant_label_key(variant)).into_owned();
    tf("menu.seed.title_game", &[("game", game.as_str())])
}

/// A variant the shipped table has no curated seeds for; `None` once the table has them all.
fn variant_without_curated_deals() -> Option<Variant> {
    Variant::ALL
        .into_iter()
        .find(|variant| curated::seeds(*variant, Difficulty::VeryEasy).is_empty())
}

fn open_game_list(h: &mut Harness) {
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
}

#[test]
fn new_game_from_seed_asks_for_the_game_first() {
    let mut h = Harness::new();
    open_game_list(&mut h);
    assert_eq!(h.app.screen(), Screen::MainMenu);
    let text = h.text();
    assert!(
        text.contains(&t("menu.seed.choose_game").into_owned()),
        "{text}"
    );
    assert!(
        text.contains(&t("menu.footer.seed_options").into_owned()),
        "{text}"
    );
    assert!(
        !text.contains(&t("menu.seed.type_seed").into_owned()),
        "{text}"
    );
}

#[test]
fn enter_on_a_game_opens_the_seed_options_of_that_game() {
    let mut h = Harness::new();
    open_game_list(&mut h);
    h.press(KeyCode::Enter);
    let text = h.text();
    assert!(text.contains(&game_title(Variant::ALL[0])), "{text}");
    assert!(
        text.contains(&t("menu.seed.type_seed").into_owned()),
        "{text}"
    );
    assert!(text.contains(&t("menu.seed.levels").into_owned()), "{text}");
    assert!(text.contains(&t("menu.footer.open").into_owned()), "{text}");
    assert_eq!(h.app.screen(), Screen::MainMenu);
}

#[test]
fn every_variant_is_reached_by_keyboard_and_deals_a_typed_seed() {
    for variant in Variant::ALL {
        let mut h = Harness::new();
        open_game_list(&mut h);
        h.focus_game_variant(variant);
        h.press(KeyCode::Enter);
        assert!(h.text().contains(&game_title(variant)), "{variant:?}");
        h.press(KeyCode::Enter);
        for c in "909".chars() {
            h.type_char(c);
        }
        h.press(KeyCode::Enter);
        assert!(
            h.app
                .board()
                .session()
                .is_some_and(|s| s.variant() == variant && s.seed() == 909),
            "{variant:?}"
        );
    }
}

#[test]
fn the_mouse_chooses_a_game_then_types_a_seed_for_it() {
    let variant = Variant::Yukon(YukonRules::Alaska);
    let mut h = Harness::with(
        MemoryStorage::new(),
        crate::test_support::TRUE_COLOR,
        120,
        40,
    );
    open_game_list(&mut h);
    let (column, row) = h.find(&t(variant_label_key(variant))).unwrap_or_default();
    h.click(column, row);
    assert!(h.text().contains(&game_title(variant)));
    let (column, row) = h.find(&t("menu.seed.type_seed")).unwrap_or_default();
    h.click(column, row);
    for c in "31".chars() {
        h.type_char(c);
    }
    let (column, row) = h.find("Continue").unwrap_or_default();
    h.click(column, row);
    assert_eq!(h.app.screen(), Screen::Board);
    assert!(
        h.app
            .board()
            .session()
            .is_some_and(|s| s.variant() == variant && s.seed() == 31)
    );
}

#[test]
fn the_mouse_reaches_the_levels_and_the_footer_back_returns_to_the_options() {
    let mut h = Harness::with(
        MemoryStorage::new(),
        crate::test_support::TRUE_COLOR,
        120,
        40,
    );
    open_game_list(&mut h);
    h.press(KeyCode::Enter);
    let (column, row) = h.find(&t("menu.seed.levels")).unwrap_or_default();
    h.click(column, row);
    assert!(
        h.text()
            .contains(&t("menu.seed.levels_prompt").into_owned())
    );
    let (column, row) = h.find(&t("help.action.back")).unwrap_or_default();
    h.click(column, row);
    let text = h.text();
    assert!(
        text.contains(&t("menu.seed.type_seed").into_owned()),
        "{text}"
    );
    assert!(
        !text.contains(&t("menu.seed.levels_prompt").into_owned()),
        "{text}"
    );
    assert_eq!(h.app.screen(), Screen::MainMenu);
    assert!(h.app.board().session().is_none());
}

#[test]
fn levels_of_a_variant_without_curated_seeds_say_so_and_start_nothing() {
    let Some(variant) = variant_without_curated_deals() else {
        return;
    };
    let mut h = Harness::new();
    open_game_list(&mut h);
    h.focus_game_variant(variant);
    h.press(KeyCode::Enter);
    h.press(KeyCode::Down);
    h.press(KeyCode::Down);
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
    assert!(h.text().contains(&t("menu.seed.levels_none").into_owned()));
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::MainMenu);
    assert!(h.app.board().session().is_none());
}

#[test]
fn every_seed_page_fits_in_every_locale_at_100_by_30_and_100_by_60() {
    for locale in Locale::offered() {
        for height in [30, 60] {
            let mut h = Harness::with(locale_storage(locale), PLAIN, 100, height);
            open_game_list(&mut h);
            let text = h.text();
            assert!(
                text.contains(&t("menu.seed.choose_game").into_owned()),
                "{locale:?} {height}\n{text}"
            );
            assert!(
                text.contains(&t("menu.footer.seed_options").into_owned()),
                "{locale:?} {height}\n{text}"
            );
            h.press(KeyCode::Enter);
            let text = h.text();
            assert!(
                text.contains(&game_title(Variant::ALL[0])),
                "{locale:?} {height}\n{text}"
            );
            assert!(
                text.contains(&t("menu.seed.type_seed").into_owned()),
                "{locale:?} {height}\n{text}"
            );
            assert!(
                text.contains(&t("menu.seed.levels").into_owned()),
                "{locale:?} {height}\n{text}"
            );
            assert!(
                text.contains(&t("menu.footer.open").into_owned()),
                "{locale:?} {height}\n{text}"
            );
            for _ in 0..3 {
                h.press(KeyCode::Down);
            }
            h.press(KeyCode::Enter);
            let text = h.text();
            assert!(
                text.contains(&t("menu.seed.levels_prompt").into_owned()),
                "{locale:?} {height}\n{text}"
            );
            assert!(
                text.contains(&t("menu.footer.play").into_owned()),
                "{locale:?} {height}\n{text}"
            );
            h.press(KeyCode::Esc);
            h.press(KeyCode::Enter);
            let text = h.text();
            assert!(
                text.contains(&t("menu.seed.keys").into_owned()),
                "{locale:?} {height}\n{text}"
            );
            set_locale(Locale::EnUs);
        }
    }
}

#[test]
fn options_changed_for_the_variant_dim_the_levels_and_say_why() {
    let mut h = seed_options_under(Variant::Golf, GOLF_WRAP);
    let text = h.text();
    assert!(text.contains(&t("menu.seed.levels").into_owned()), "{text}");
    assert!(
        text.contains(&t("menu.seed.levels_standard_only").into_owned()),
        "{text}"
    );
}

#[test]
fn default_options_and_options_of_another_game_show_no_reason() {
    let reason = t("menu.seed.levels_standard_only").into_owned();
    let mut h = seed_options_under(Variant::Golf, GameOptions::DEFAULT);
    assert!(!h.text().contains(&reason), "{}", h.text());
    let mut h = seed_options_under(Variant::Golf, KLONDIKE_ONE_PASS);
    assert!(!h.text().contains(&reason), "{}", h.text());
}

#[test]
fn dimmed_levels_open_neither_by_keyboard_nor_by_mouse() {
    let levels_prompt = t("menu.seed.levels_prompt").into_owned();
    let mut h = seed_options_under(Variant::Golf, GOLF_WRAP);
    for _ in 0..3 {
        h.press(KeyCode::Down);
    }
    h.press(KeyCode::Enter);
    assert!(!h.text().contains(&levels_prompt), "{}", h.text());
    let (column, row) = h.find(&t("menu.seed.levels")).unwrap_or_default();
    h.click(column, row);
    let text = h.text();
    assert!(!text.contains(&levels_prompt), "{text}");
    assert!(
        text.contains(&t("menu.seed.type_seed").into_owned()),
        "{text}"
    );
    assert_eq!(h.app.screen(), Screen::MainMenu);
    assert!(h.app.board().session().is_none());
}

#[test]
fn the_levels_of_another_game_still_open_under_changed_options() {
    let mut h = seed_options_under(Variant::Golf, KLONDIKE_ONE_PASS);
    for _ in 0..3 {
        h.press(KeyCode::Down);
    }
    h.press(KeyCode::Enter);
    assert!(
        h.text()
            .contains(&t("menu.seed.levels_prompt").into_owned())
    );
}

#[test]
fn the_reason_line_fits_in_every_locale_at_100_by_30() {
    for locale in Locale::offered() {
        let mut h = Harness::with(storage_with_options(locale, GOLF_WRAP), PLAIN, 100, 30);
        open_game_list(&mut h);
        h.focus_game_variant(Variant::Golf);
        h.press(KeyCode::Enter);
        let reason = t("menu.seed.levels_standard_only").into_owned();
        let text = h.text();
        assert!(text.contains(&reason), "{locale:?}\n{text}");
        set_locale(Locale::EnUs);
    }
}
