//! The winnability checks of the statistics screen: the header and the legend at 100 by 30 in every
//! offered locale, the keys and buttons that start a check, its progress, and what it stores.

use oxysol_core::{SolveBudget, Variant, Verdict, YukonRules, solve};
use ratatui::crossterm::event::KeyCode;

use super::support::{click_text, open_stats, three_games};
use crate::app::Screen;
use crate::glyphs::Glyphs;
use crate::i18n::{Locale, set_locale, t};
use crate::screens::stats::format::legend_line;
use crate::settings::{GlyphMode, Settings};
use crate::solver::Proof;
use crate::storage::memory::{HistoryRow, MemoryStorage};
use crate::storage::{GameId, Outcome, Storage};
use crate::test_support::Harness;

const WIDTH: u16 = 100;
const HEIGHT: u16 = 30;
const FREECELL: Variant = Variant::FreeCell;

fn in_locale<T>(locale: Locale, read: impl FnOnce() -> T) -> T {
    set_locale(locale);
    let value = read();
    set_locale(Locale::EnUs);
    value
}

fn known_winnable_seed() -> u64 {
    (100_000u64..)
        .find(|seed| {
            matches!(
                solve(FREECELL, *seed, SolveBudget::default_for(FREECELL)),
                Verdict::Winnable { .. }
            )
        })
        .unwrap_or(100_000)
}

fn row_with(lines: &[String], seed: u64) -> String {
    lines
        .iter()
        .find(|line| line.split_whitespace().any(|word| word == seed.to_string()))
        .cloned()
        .unwrap_or_default()
}

fn proof_of(h: &Harness, id: i64) -> Option<Proof> {
    h.storage
        .borrow()
        .history()
        .unwrap_or_default()
        .iter()
        .find(|record| record.id == GameId(id))
        .and_then(|record| record.proof)
}

fn status_of(h: &mut Harness) -> String {
    h.lines()[usize::from(HEIGHT) - 2].clone()
}

#[test]
fn the_winnable_header_and_the_legend_fit_100_by_30_in_every_offered_locale() {
    for locale in Locale::offered() {
        let (seed_header, short, legend) = in_locale(locale, || {
            (
                t("stats.history.seed").into_owned(),
                t("stats.history.winnable_short").into_owned(),
                legend_line(&Glyphs::new(GlyphMode::Unicode)),
            )
        });
        let mut storage = three_games();
        storage.settings = Some(Settings {
            locale,
            ..Settings::default()
        });
        let mut h = open_stats(storage, WIDTH, HEIGHT);
        let lines = h.lines();
        let text = lines.join("\n");
        assert!(
            lines
                .iter()
                .any(|line| line.contains(&seed_header) && line.contains(&short)),
            "{locale:?}: the header lacks {short:?}\n{text}"
        );
        assert!(
            lines
                .iter()
                .all(|line| line.trim_end().chars().count() <= usize::from(WIDTH)),
            "{locale:?}\n{text}"
        );
        let legend_row = usize::from(HEIGHT) - 4;
        assert!(
            lines[legend_row].contains(&legend),
            "{locale:?}: the legend is not under the table\n{text}"
        );
    }
}

#[test]
fn w_checks_the_selected_game_and_a_known_winnable_seed_turns_its_mark_to_a_check() {
    let seed = known_winnable_seed();
    let storage = MemoryStorage {
        history: vec![HistoryRow::finished(
            1,
            FREECELL,
            seed,
            Outcome::Abandoned,
            30_000,
        )],
        ..MemoryStorage::new()
    };
    let mut h = open_stats(storage, WIDTH, HEIGHT);
    assert!(row_with(&h.lines(), seed).contains('·'));
    h.solver.borrow_mut().verdicts.insert(seed, Proof::Winnable);
    h.type_char('w');
    let lines = h.lines();
    assert!(row_with(&lines, seed).contains('✓'), "{}", lines.join("\n"));
    assert_eq!(proof_of(&h, 1), Some(Proof::Winnable));
    let submitted = h.solver.borrow().submitted.clone();
    assert_eq!(submitted.len(), 1);
    assert_eq!((submitted[0].variant, submitted[0].seed), (FREECELL, seed));
}

#[test]
fn w_checks_a_yukon_game_and_stores_the_answer() {
    let yukon = Variant::Yukon(YukonRules::Alaska);
    let seed = 13;
    let storage = MemoryStorage {
        history: vec![HistoryRow::finished(
            1,
            yukon,
            seed,
            Outcome::Abandoned,
            30_000,
        )],
        ..MemoryStorage::new()
    };
    let mut h = open_stats(storage, WIDTH, HEIGHT);
    let proof = match solve(yukon, seed, SolveBudget::default_for(yukon)) {
        Verdict::Winnable { .. } => Proof::Winnable,
        Verdict::Unwinnable => Proof::Unwinnable,
        Verdict::Unknown => Proof::Unknown,
    };
    assert_eq!(proof, Proof::Winnable);
    h.solver.borrow_mut().verdicts.insert(seed, proof);
    h.type_char('w');
    assert_eq!(proof_of(&h, 1), Some(Proof::Winnable));
    let submitted = h.solver.borrow().submitted.clone();
    assert_eq!((submitted[0].variant, submitted[0].seed), (yukon, seed));
}

#[test]
fn w_on_an_unwinnable_game_turns_its_mark_to_a_cross_and_only_that_row_changes() {
    let mut h = open_stats(three_games(), WIDTH, HEIGHT);
    h.press(KeyCode::Down);
    h.solver.borrow_mut().verdicts.insert(22, Proof::Unwinnable);
    h.type_char('w');
    let lines = h.lines();
    assert!(row_with(&lines, 22).contains('✗'), "{}", lines.join("\n"));
    assert!(row_with(&lines, 33).contains('·'));
    assert!(row_with(&lines, 11).contains('·'));
    assert_eq!(proof_of(&h, 2), Some(Proof::Unwinnable));
    assert_eq!(proof_of(&h, 1), None);
}

#[test]
fn capital_w_checks_every_game_without_a_proof_and_keeps_the_others() {
    let mut storage = three_games();
    storage.history[1].proof = Some(Proof::Winnable);
    let mut h = open_stats(storage, WIDTH, HEIGHT);
    h.solver.borrow_mut().verdicts.insert(33, Proof::Unwinnable);
    h.type_char('W');
    let submitted: Vec<u64> = h.solver.borrow().submitted.iter().map(|q| q.seed).collect();
    assert_eq!(submitted, vec![33, 11]);
    let lines = h.lines();
    assert!(row_with(&lines, 33).contains('✗'));
    assert!(row_with(&lines, 11).contains('?'));
    assert!(row_with(&lines, 22).contains('✓'));
    assert_eq!(proof_of(&h, 1), Some(Proof::Unknown));
    assert_eq!(proof_of(&h, 3), Some(Proof::Unwinnable));
    assert_eq!(proof_of(&h, 2), Some(Proof::Winnable));
}

#[test]
fn the_progress_counts_the_answers_and_the_finished_check_says_so() {
    let mut h = open_stats(three_games(), WIDTH, HEIGHT);
    h.solver.borrow_mut().held = true;
    h.type_char('W');
    assert!(
        status_of(&mut h).contains("Checking winnable: 0 of 3"),
        "{}",
        h.text()
    );
    h.solver.borrow_mut().release(1);
    h.advance(0);
    assert!(status_of(&mut h).contains("Checking winnable: 1 of 3"));
    h.solver.borrow_mut().release(2);
    h.advance(0);
    assert!(status_of(&mut h).contains("Winnable check finished (3)"));
    assert_eq!(h.solver.borrow().pending(), 0);
}

#[test]
fn a_second_check_does_not_start_while_one_runs_and_an_empty_check_says_why() {
    let mut h = open_stats(three_games(), WIDTH, HEIGHT);
    h.solver.borrow_mut().held = true;
    h.type_char('W');
    h.type_char('W');
    assert_eq!(h.solver.borrow().submitted.len(), 3);
    h.solver.borrow_mut().release(3);
    h.advance(0);
    h.type_char('w');
    assert!(status_of(&mut h).contains("This game is already checked"));
    h.type_char('W');
    assert!(status_of(&mut h).contains("Every game is already checked"));
    assert_eq!(h.solver.borrow().submitted.len(), 3);
}

#[test]
fn the_buttons_start_the_same_checks_as_the_keys() {
    let mut h = open_stats(three_games(), WIDTH, HEIGHT);
    click_text(&mut h, "Check all ]");
    assert!(
        status_of(&mut h).contains("Winnable check finished (3)"),
        "{}",
        h.text()
    );
    assert_eq!(h.solver.borrow().submitted.len(), 3);
    let mut h = open_stats(three_games(), WIDTH, HEIGHT);
    click_text(&mut h, "Check ]");
    assert_eq!(h.solver.borrow().submitted.len(), 1);
    assert_eq!(proof_of(&h, 3), Some(Proof::Unknown));
}

#[test]
fn leaving_the_screen_stops_a_running_check_and_drops_its_queued_queries() {
    let mut h = open_stats(three_games(), WIDTH, HEIGHT);
    h.solver.borrow_mut().held = true;
    h.type_char('W');
    let cancels = h.solver.borrow().cancels;
    h.press(KeyCode::Esc);
    assert_eq!(h.app.screen(), Screen::MainMenu);
    assert_eq!(h.solver.borrow().cancels, cancels + 1);
    assert_eq!(h.solver.borrow().pending(), 0);
}

#[test]
fn the_check_labels_and_keys_are_shown_in_every_offered_locale() {
    for locale in Locale::offered() {
        let (check, check_all, hint) = in_locale(locale, || {
            (
                t("stats.button.check").into_owned(),
                t("stats.button.check_all").into_owned(),
                t("stats.check.hint").into_owned(),
            )
        });
        let mut storage = three_games();
        storage.settings = Some(Settings {
            locale,
            ..Settings::default()
        });
        let mut h = open_stats(storage, WIDTH, HEIGHT);
        let lines = h.lines();
        let buttons = &lines[usize::from(HEIGHT) - 3];
        assert!(
            buttons.contains(&format!("[ {check} ]")),
            "{locale:?}: {buttons}"
        );
        assert!(
            buttons.contains(&format!("[ {check_all} ]")),
            "{locale:?}: {buttons}"
        );
        assert!(
            lines
                .iter()
                .all(|line| line.trim_end().chars().count() <= usize::from(WIDTH))
        );
        assert!(
            lines[usize::from(HEIGHT) - 2].contains(&hint),
            "{locale:?}: {hint}"
        );
    }
}
