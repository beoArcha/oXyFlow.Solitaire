//! The result column of the history: each result has its own colour token that passes the contrast
//! target in every theme, the word still names the result, and an unchecked cell says what the
//! legend says.

use ratatui::style::Color;

use super::KLONDIKE;
use super::support::{open_stats, three_games};
use crate::caps::ColorDepth;
use crate::glyphs::Glyphs;
use crate::i18n::{Locale, set_locale, t};
use crate::screens::stats::format::{legend_line, result_token};
use crate::settings::{GlyphMode, Settings};
use crate::storage::Outcome;
use crate::storage::memory::HistoryRow;
use crate::theme::contrast::contrast_ratio;
use crate::theme::{Theme, ThemeId, Token};

const WIDTH: u16 = 130;
const HEIGHT: u16 = 40;
const RESULTS: [Option<Outcome>; 3] = [Some(Outcome::Won), Some(Outcome::Abandoned), None];

fn rgb_of(color: Option<Color>) -> [u8; 3] {
    match color {
        Some(Color::Rgb(r, g, b)) => [r, g, b],
        other => panic!("expected a true-colour value, found {other:?}"),
    }
}

fn in_locale<T>(locale: Locale, read: impl FnOnce() -> T) -> T {
    set_locale(locale);
    let value = read();
    set_locale(Locale::EnUs);
    value
}

fn line_of_seed<'a>(lines: &'a [String], seed: &str) -> Option<(usize, &'a String)> {
    lines
        .iter()
        .enumerate()
        .find(|(_, line)| line.split_whitespace().any(|word| word == seed))
}

#[test]
fn every_result_colour_reaches_the_contrast_target_in_every_theme() {
    for id in ThemeId::ALL {
        let theme = Theme::new(id, ColorDepth::TrueColor);
        for outcome in RESULTS {
            let style = theme.style(result_token(outcome), Token::Background);
            let ratio = contrast_ratio(rgb_of(style.fg), rgb_of(style.bg));
            assert!(
                ratio >= id.contrast_target(),
                "{id:?} {outcome:?}: {ratio:.2}, target {}",
                id.contrast_target()
            );
        }
    }
}

#[test]
fn the_three_results_have_three_colour_tokens() {
    let tokens = RESULTS.map(result_token);
    assert_ne!(tokens[0], tokens[1]);
    assert_ne!(tokens[1], tokens[2]);
    assert_ne!(tokens[0], tokens[2]);
}

#[test]
fn each_result_word_is_drawn_in_its_own_colour_in_every_theme() {
    let glyphs = Glyphs::new(GlyphMode::Unicode);
    for id in ThemeId::ALL {
        let mut storage = three_games();
        storage.history.push(HistoryRow::open(0, KLONDIKE, 44));
        storage.settings = Some(Settings {
            theme: id,
            ..Settings::default()
        });
        let mut h = open_stats(storage, WIDTH, HEIGHT);
        let lines = h.lines();
        let theme = Theme::new(id, ColorDepth::TrueColor);
        for (seed, word, outcome) in [
            ("11", "Won", Some(Outcome::Won)),
            ("22", "Abandoned", Some(Outcome::Abandoned)),
            ("44", "In progress", None),
        ] {
            let (y, line) = line_of_seed(&lines, seed)
                .unwrap_or_else(|| panic!("{id:?}: no row for seed {seed}"));
            let focused = line.contains(glyphs.focus_left);
            let x = line[..line.find(word).unwrap_or_default()].chars().count();
            let buffer = h.terminal.backend().buffer();
            let expected = if focused {
                theme.color(Token::FocusFg)
            } else {
                theme.color(result_token(outcome))
            };
            let cell = &buffer[(
                u16::try_from(x).unwrap_or_default(),
                u16::try_from(y).unwrap_or_default(),
            )];
            assert_eq!(cell.fg, expected, "{id:?}: {word:?} in {line:?}");
        }
    }
}

#[test]
fn the_unchecked_cell_says_exactly_what_the_legend_says_in_every_offered_locale() {
    for locale in Locale::offered() {
        let (cell, legend) = in_locale(locale, || {
            let glyphs = Glyphs::new(GlyphMode::Unicode);
            (
                format!("{} {}", glyphs.proof_unchecked, t("stats.legend.unchecked")),
                legend_line(&glyphs),
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
            lines.iter().any(|line| line.contains(&legend)),
            "{locale:?}: the legend lacks {legend:?}\n{text}"
        );
        let (_, line) = line_of_seed(&lines, "11")
            .unwrap_or_else(|| panic!("{locale:?}: no row for seed 11\n{text}"));
        assert!(line.contains(&cell), "{locale:?}: {line:?} lacks {cell:?}");
        assert!(
            lines
                .iter()
                .all(|line| line.trim_end().chars().count() <= usize::from(WIDTH)),
            "{locale:?}\n{text}"
        );
    }
}
