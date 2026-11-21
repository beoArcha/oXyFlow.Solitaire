use oxysol_core::Variant;
use ratatui::crossterm::event::{KeyCode, MouseEventKind};

use super::support::{STATISTICS_ITEM, many_games, open_stats, row, three_games, wheel};
use crate::app::Screen;
use crate::glyphs::is_wgl4;
use crate::i18n::{Locale, set_locale, t, tf};
use crate::settings::Settings;
use crate::solver::Proof;
use crate::storage::Outcome::{Abandoned, Won};
use crate::storage::memory::{HistoryRow, MemoryStorage};
use crate::test_support::{Harness, PLAIN, TRUE_COLOR};

#[test]
fn the_list_scrolls_by_key_page_and_wheel_and_a_click_selects_a_row() {
    let mut h = open_stats(many_games(60), 100, 30);
    assert!(h.text().contains("Game 1 of 60"));
    assert!(
        !h.text().contains("1970-01-01 00:01"),
        "the oldest game is off screen"
    );
    h.press(KeyCode::End);
    let text = h.text();
    assert!(text.contains("Game 60 of 60"), "{text}");
    assert!(
        text.contains("1970-01-01 00:01"),
        "the oldest game is visible"
    );
    h.press(KeyCode::Home);
    assert!(h.text().contains("Game 1 of 60"));
    h.press(KeyCode::PageDown);
    assert!(!h.text().contains("Game 1 of 60"));
    h.press(KeyCode::PageUp);
    assert!(h.text().contains("Game 1 of 60"));
    wheel(&mut h, MouseEventKind::ScrollDown);
    wheel(&mut h, MouseEventKind::ScrollDown);
    assert!(h.text().contains("Game 3 of 60"));
    wheel(&mut h, MouseEventKind::ScrollUp);
    assert!(h.text().contains("Game 2 of 60"));
    let (column, header) = h.find("Date").unwrap_or_default();
    h.click(column + 2, header + 3);
    assert!(h.text().contains("Game 3 of 60"));
    h.type_char('c');
    assert_eq!(h.clipboard.borrow().copies, ["638"]);
}

#[test]
fn the_screen_fits_100_by_30_in_both_locales_with_only_wgl4_glyphs() {
    for locale in Locale::offered() {
        let settings = Settings {
            locale,
            ..Settings::default()
        };
        let mut storage = many_games(40);
        storage.history.push(row(
            41,
            Variant::Pyramid(oxysol_core::PyramidRules::Relaxed),
            Abandoned,
            3_725_000,
        ));
        for variant in Variant::ALL {
            storage.history.push(row(50, variant, Won, 61_000));
        }
        storage.settings = Some(settings);
        for caps in [TRUE_COLOR, PLAIN] {
            let mut h = Harness::with(storage_clone(&storage), caps, 100, 30);
            for _ in 0..STATISTICS_ITEM {
                h.press(KeyCode::Down);
            }
            h.press(KeyCode::Enter);
            assert_eq!(h.app.screen(), Screen::Stats);
            h.type_char('c');
            let lines = h.lines();
            assert_eq!(lines.len(), 30);
            for line in &lines {
                for ch in line.chars() {
                    assert!(ch.is_ascii() || is_wgl4(ch), "{ch:?} in {line:?}");
                }
            }
            let text = lines.join("\n");
            assert!(text.contains("50"), "{text}");
            let widest = lines
                .iter()
                .map(|line| line.trim_end().chars().count())
                .max();
            assert!(widest.is_some_and(|width| width <= 100));
            assert!(
                !text.contains("stats."),
                "a key is shown instead of text\n{text}"
            );
        }
    }
}

fn storage_clone(storage: &MemoryStorage) -> MemoryStorage {
    MemoryStorage {
        history: storage.history.clone(),
        settings: storage.settings.clone(),
        ..MemoryStorage::new()
    }
}

fn in_locale<T>(locale: Locale, read: impl FnOnce() -> T) -> T {
    set_locale(locale);
    let value = read();
    set_locale(Locale::EnUs);
    value
}

fn mode_label(locale: Locale, mode: &str) -> String {
    set_locale(locale);
    let label = format!("({})", t(&format!("stats.mode.{mode}")));
    set_locale(Locale::EnUs);
    label
}

#[test]
fn the_summary_names_the_undo_mode_once_two_modes_exist_and_still_fits() {
    for locale in Locale::offered() {
        let mut storage = three_games();
        storage.history.push(HistoryRow {
            undo_allowed: false,
            ..row(
                4,
                Variant::Pyramid(oxysol_core::PyramidRules::Relaxed),
                Won,
                9_000,
            )
        });
        storage.settings = Some(Settings {
            locale,
            ..Settings::default()
        });
        let mut h = open_stats(storage, 100, 30);
        let lines = h.lines();
        let text = lines.join("\n");
        let (undo, no_undo) = (mode_label(locale, "undo"), mode_label(locale, "no_undo"));
        assert!(text.contains(&undo) && text.contains(&no_undo), "{text}");
        assert!(
            lines
                .iter()
                .all(|line| line.trim_end().chars().count() <= 100)
        );
    }
}

#[test]
fn every_variant_in_both_undo_modes_is_summarized_at_100_by_30() {
    for locale in Locale::offered() {
        let mut history = Vec::new();
        for (index, variant) in Variant::ALL.into_iter().enumerate() {
            let id = i64::try_from(index).unwrap_or(0) * 2 + 1;
            history.push(row(id, variant, Won, 61_000));
            history.push(HistoryRow {
                undo_allowed: false,
                ..row(id + 1, variant, Won, 62_000)
            });
        }
        let storage = MemoryStorage {
            history,
            settings: Some(Settings {
                locale,
                ..Settings::default()
            }),
            ..MemoryStorage::new()
        };
        let total = (2 * Variant::ALL.len()).to_string();
        let expected = in_locale(locale, || {
            (
                t("stats.summary.longest").into_owned(),
                tf("stats.position", &[("current", "1"), ("total", &total)]),
                t("stats.button.back").into_owned(),
                tf("stats.position", &[("current", &total), ("total", &total)]),
            )
        });
        let (streak, first, back, last) = expected;
        let mut h = open_stats(storage, 100, 30);
        let lines = h.lines();
        let no_undo = mode_label(locale, "no_undo");
        let summarized = lines.iter().filter(|line| line.contains(&no_undo)).count();
        assert_eq!(summarized, Variant::ALL.len(), "{}", lines.join("\n"));
        let widest = lines
            .iter()
            .map(|line| line.trim_end().chars().count())
            .max();
        assert!(widest.is_some_and(|width| width <= 100));
        assert!(!lines.iter().any(|line| line.contains(&streak)));
        let text = lines.join(
            "
",
        );
        assert!(text.contains(&first), "{text}");
        assert!(text.contains(&back), "{text}");
        set_locale(locale);
        h.press(KeyCode::End);
        assert!(
            h.text().contains(&last),
            "{locale:?} {last}
{}",
            h.text()
        );
    }
    set_locale(Locale::EnUs);
}

#[test]
fn the_history_marks_winnable_deals_in_both_languages_at_100_by_30() {
    for (locale, yes, no, unknown, legend_word) in [
        (Locale::EnUs, "✓", "✗", "?", "winnable"),
        (Locale::PlPl, "✓", "✗", "?", "do wygrania"),
    ] {
        let mut storage = three_games();
        storage.history[0].proof = Some(Proof::Winnable);
        storage.history[1].proof = Some(Proof::Unwinnable);
        storage.history[2].proof = Some(Proof::Unknown);
        storage.settings = Some(Settings {
            locale,
            ..Settings::default()
        });
        let mut h = open_stats(storage, 100, 30);
        let lines = h.lines();
        assert!(
            lines.iter().any(|line| line.contains('?')),
            "missing short header"
        );
        for (seed, mark) in [("11", yes), ("22", no), ("33", unknown)] {
            let line = lines
                .iter()
                .find(|line| line.split_whitespace().any(|word| word == seed))
                .unwrap_or_else(|| panic!("no row for seed {seed}: {lines:#?}"));
            assert!(line.contains(mark), "{line}");
        }
        assert!(
            lines.iter().any(|line| line.contains(legend_word)),
            "missing legend"
        );
        assert!(
            lines
                .iter()
                .all(|line| line.trim_end().chars().count() <= 100)
        );
    }
}

#[test]
fn the_history_shows_winnable_text_column_at_130_by_40() {
    for (locale, header, yes, no, unknown) in [
        (Locale::EnUs, "Winnable", "✓ yes", "✗ no", "? unknown"),
        (Locale::PlPl, "Wygrana?", "✓ tak", "✗ nie", "? nieznane"),
    ] {
        let mut storage = three_games();
        storage.history[0].proof = Some(Proof::Winnable);
        storage.history[1].proof = Some(Proof::Unwinnable);
        storage.history[2].proof = Some(Proof::Unknown);
        storage.settings = Some(Settings {
            locale,
            ..Settings::default()
        });
        let mut h = open_stats(storage, 130, 40);
        let lines = h.lines();
        assert!(lines.iter().any(|line| line.contains(header)), "{header}");
        for (seed, mark) in [("11", yes), ("22", no), ("33", unknown)] {
            let line = lines
                .iter()
                .find(|line| line.split_whitespace().any(|word| word == seed))
                .unwrap_or_else(|| panic!("no row for seed {seed}: {lines:#?}"));
            assert!(line.contains(mark), "{line}");
        }
        assert!(
            lines
                .iter()
                .all(|line| line.trim_end().chars().count() <= 130)
        );
    }
}
