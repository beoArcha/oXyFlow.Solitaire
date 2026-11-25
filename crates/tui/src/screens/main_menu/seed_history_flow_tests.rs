//! The played seeds and the random replay through the whole app, by keyboard and by mouse, with and
//! without a history, and every history page at the minimum sizes in every locale.

use ratatui::crossterm::event::KeyCode;

use crate::app::Screen;
use crate::i18n::{Locale, set_locale, t, tf};
use crate::screens::Effect;
use crate::screens::variant_label_key;
use crate::storage::Outcome;
use crate::storage::memory::{HistoryRow, MemoryStorage};
use crate::test_support::{Harness, PLAIN};
use oxysol_core::Variant;

/// The game of the history: the first variant of the list, so the game list needs no moves.
const PLAYED: Variant = Variant::ALL[0];
/// Another variant, whose games must not be offered for `PLAYED`.
const OTHER: Variant = Variant::ALL[1];

/// Three games of `PLAYED`, oldest first (rows 1 and 3 started at 00:01 and 00:03 UTC), and one of
/// `OTHER`. Seeds 10 and 30 are played; 20 belongs to the other variant.
fn history() -> MemoryStorage {
    MemoryStorage {
        history: vec![
            HistoryRow::finished(1, PLAYED, 10, Outcome::Won, 60_000),
            HistoryRow::finished(2, OTHER, 20, Outcome::Won, 60_000),
            HistoryRow::finished(3, PLAYED, 30, Outcome::Abandoned, 60_000),
        ],
        ..MemoryStorage::new()
    }
}

fn locale_storage(locale: Locale, storage: MemoryStorage) -> MemoryStorage {
    MemoryStorage {
        settings: Some(crate::settings::Settings {
            locale,
            ..crate::settings::Settings::for_caps(&PLAIN)
        }),
        ..storage
    }
}

fn harness(storage: MemoryStorage) -> Harness {
    Harness::with(storage, PLAIN, 100, 30)
}

fn row(when: &str, outcome: Outcome, seed: u64) -> String {
    let result = match outcome {
        Outcome::Won => t("stats.outcome.won"),
        Outcome::Abandoned => t("stats.outcome.abandoned"),
    };
    tf(
        "menu.seed.played_row",
        &[
            ("when", when),
            ("result", result.as_ref()),
            ("seed", &seed.to_string()),
        ],
    )
}

fn open_seed_options(h: &mut Harness) {
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
    h.press(KeyCode::Enter);
}

fn board_seed(h: &Harness) -> Option<u64> {
    h.app
        .board()
        .session()
        .filter(|session| session.variant() == PLAYED)
        .map(|session| session.seed())
}

#[test]
fn the_played_seeds_are_listed_newest_first_with_result_and_time() {
    let mut h = harness(history());
    open_seed_options(&mut h);
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
    let text = h.text();
    let newer = row("1970-01-01 00:03", Outcome::Abandoned, 30);
    let older = row("1970-01-01 00:01", Outcome::Won, 10);
    let (Some(first), Some(second)) = (text.find(&newer), text.find(&older)) else {
        panic!("both played seeds should be listed:\n{text}");
    };
    assert!(first < second, "{text}");
    assert!(!text.contains("seed 20"), "{text}");
    assert!(text.contains(&t("menu.seed.played_prompt").into_owned()));
}

#[test]
fn enter_on_a_played_seed_starts_that_seed_as_typed() {
    let mut h = harness(history());
    open_seed_options(&mut h);
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Board);
    assert_eq!(board_seed(&h), Some(30));
}

#[test]
fn the_mouse_starts_a_played_seed() {
    let mut h = harness(history());
    open_seed_options(&mut h);
    let (column, row_index) = h.find(&t("menu.seed.played")).unwrap_or_default();
    h.click(column, row_index);
    let (column, row_index) = h
        .find(&row("1970-01-01 00:01", Outcome::Won, 10))
        .unwrap_or_default();
    h.click(column, row_index);
    assert_eq!(h.app.screen(), Screen::Board);
    assert_eq!(board_seed(&h), Some(10));
}

#[test]
fn the_random_replay_starts_one_played_seed_by_keyboard() {
    let mut h = harness(history());
    open_seed_options(&mut h);
    h.press(KeyCode::Down);
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
    assert_eq!(h.app.screen(), Screen::Board);
    assert!(matches!(board_seed(&h), Some(10 | 30)));
}

#[test]
fn the_random_replay_starts_one_played_seed_by_mouse() {
    let mut h = harness(history());
    open_seed_options(&mut h);
    let (column, row_index) = h.find(&t("menu.seed.random_replay")).unwrap_or_default();
    h.click(column, row_index);
    assert_eq!(h.app.screen(), Screen::Board);
    assert!(matches!(board_seed(&h), Some(10 | 30)));
}

#[test]
fn without_a_played_game_the_played_seeds_say_so_and_start_nothing() {
    let mut h = harness(MemoryStorage::new());
    open_seed_options(&mut h);
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
    let text = h.text();
    assert!(
        text.contains(&t("menu.seed.played_none").into_owned()),
        "{text}"
    );
    assert_eq!(h.app.screen(), Screen::MainMenu);
    assert!(h.app.board().session().is_none());
}

#[test]
fn without_a_played_game_the_random_replay_says_so_by_mouse_and_starts_nothing() {
    let mut h = harness(MemoryStorage::new());
    open_seed_options(&mut h);
    let (column, row_index) = h.find(&t("menu.seed.random_replay")).unwrap_or_default();
    h.click(column, row_index);
    assert!(h.text().contains(&t("menu.seed.played_none").into_owned()));
    assert_eq!(h.app.screen(), Screen::MainMenu);
    assert!(h.app.board().session().is_none());
}

#[test]
fn an_unreadable_history_says_so_and_starts_nothing() {
    let mut h = harness(history());
    h.storage.borrow_mut().failing = true;
    h.app.apply(Effect::Goto(Screen::MainMenu));
    open_seed_options(&mut h);
    h.press(KeyCode::Down);
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
    let text = h.text();
    assert!(
        text.contains(&t("menu.seed.history_unreadable").into_owned()),
        "{text}"
    );
    assert_eq!(h.app.screen(), Screen::MainMenu);
}

#[test]
fn the_notice_goes_once_the_next_key_is_pressed() {
    let mut h = harness(MemoryStorage::new());
    open_seed_options(&mut h);
    h.press(KeyCode::Down);
    h.press(KeyCode::Enter);
    h.press(KeyCode::Up);
    assert!(!h.text().contains(&t("menu.seed.played_none").into_owned()));
}

#[test]
fn the_played_seeds_fit_in_every_locale_at_100_by_30_and_100_by_60() {
    let title = variant_label_key(PLAYED);
    for locale in Locale::offered() {
        for height in [30, 60] {
            let storage = MemoryStorage {
                settings: Some(crate::settings::Settings {
                    locale,
                    ..crate::settings::Settings::for_caps(&PLAIN)
                }),
                ..history()
            };
            let mut h = Harness::with(storage, PLAIN, 100, height);
            open_seed_options(&mut h);
            let text = h.text();
            let game = t(title).into_owned();
            assert!(
                text.contains(&tf("menu.seed.title_game", &[("game", &game)])),
                "{locale:?} {height}\n{text}"
            );
            for label in [
                "menu.seed.type_seed",
                "menu.seed.played",
                "menu.seed.random_replay",
                "menu.seed.levels",
            ] {
                assert!(
                    text.contains(&t(label).into_owned()),
                    "{locale:?} {height} {label}\n{text}"
                );
            }
            h.press(KeyCode::Down);
            h.press(KeyCode::Enter);
            let text = h.text();
            assert!(
                text.contains(&t("menu.seed.played_prompt").into_owned()),
                "{locale:?} {height}\n{text}"
            );
            assert!(
                text.contains(&row("1970-01-01 00:03", Outcome::Abandoned, 30)),
                "{locale:?} {height}\n{text}"
            );
            let mut broken = Harness::with(locale_storage(locale, history()), PLAIN, 100, height);
            broken.storage.borrow_mut().failing = true;
            broken.app.apply(Effect::Goto(Screen::MainMenu));
            open_seed_options(&mut broken);
            broken.press(KeyCode::Down);
            broken.press(KeyCode::Down);
            broken.press(KeyCode::Enter);
            let text = broken.text();
            assert!(
                text.contains(&t("menu.seed.history_unreadable").into_owned()),
                "{locale:?} {height}
{text}"
            );
            set_locale(Locale::EnUs);
        }
    }
}
