//! The guide's keys and legend: a usable key carries the available mark and never the focus
//! symbol, and the legend explains both marks in the current glyph set.

use std::time::Instant;

use oxysol_core::{DrawCount, Variant};
use ratatui::text::Line;

use super::{Guide, available_mark, keys, legend};
use crate::action::Action;
use crate::app::FontFacts;
use crate::audio::AudioStatus;
use crate::glyphs::{Glyphs, is_wgl4};
use crate::i18n::{Locale, set_locale, t};
use crate::screens::ViewCtx;
use crate::settings::{GlyphMode, Settings};
use crate::test_support::TRUE_COLOR;
use crate::theme::{Theme, ThemeId};

const MODES: [GlyphMode; 2] = [GlyphMode::Unicode, GlyphMode::Ascii];
const KEYS_WIDTH: usize = 60;
const LEGEND_WIDTH: usize = 200;

fn with_ctx<R>(mode: GlyphMode, check: impl FnOnce(&ViewCtx<'_>) -> R) -> R {
    let glyphs = Glyphs::new(mode);
    let theme = Theme::new(ThemeId::Dark, crate::caps::ColorDepth::TrueColor);
    let settings = Settings::for_caps(&TRUE_COLOR);
    let ctx = ViewCtx {
        theme: &theme,
        glyphs: &glyphs,
        settings: &settings,
        caps: &TRUE_COLOR,
        audio_status: AudioStatus::NoDevice,
        sound_styles: &[],
        saved: None,
        pointer: None,
        now: Instant::now(),
        font: FontFacts::default(),
    };
    check(&ctx)
}

fn text(line: &Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}

fn guide<'a>(usable: &'a [Action], game: &'a [Line<'static>]) -> Guide<'a> {
    Guide {
        variant: Variant::Klondike(DrawCount::One),
        usable,
        undo_allowed: true,
        game,
    }
}

#[test]
fn usable_keys_carry_the_available_mark_and_never_the_focus_symbol() {
    for mode in MODES {
        with_ctx(mode, |ctx| {
            let usable = [Action::Hint, Action::Left];
            let game = [];
            let lines: Vec<String> = keys::keys(&guide(&usable, &game), KEYS_WIDTH, ctx)
                .iter()
                .map(text)
                .collect();
            let mark = available_mark(ctx);
            assert!(
                lines
                    .iter()
                    .all(|line| !line.contains('▶') && !line.starts_with(ctx.glyphs.focus_left)),
                "the Keys section shows the focus symbol: {lines:?}"
            );
            assert_eq!(
                lines.iter().filter(|line| line.starts_with(mark)).count(),
                2,
                "the usable keys are not all marked: {lines:?}"
            );
        });
    }
}

#[test]
fn the_keys_section_lists_n_as_new_game_in_every_locale() {
    for locale in Locale::offered() {
        set_locale(locale);
        for mode in MODES {
            with_ctx(mode, |ctx| {
                let usable = [Action::NewGame];
                let game = [];
                let lines: Vec<String> = keys::keys(&guide(&usable, &game), KEYS_WIDTH, ctx)
                    .iter()
                    .map(text)
                    .collect();
                let label = t("board.guide.new_game");
                let expected: Vec<&str> = std::iter::once("n")
                    .chain(label.split_whitespace())
                    .collect();
                let mark = available_mark(ctx);
                let listed = lines.iter().any(|line| {
                    let words: Vec<&str> = line.split_whitespace().collect();
                    words.first() == Some(&mark) && words[1..] == expected[..]
                });
                assert!(
                    listed,
                    "{locale:?} {mode:?}: no `n` row for new game in {lines:?}"
                );
            });
        }
    }
    set_locale(Locale::EnUs);
}

#[test]
fn the_available_mark_is_wgl4_or_ascii() {
    with_ctx(GlyphMode::Unicode, |ctx| {
        let mark = available_mark(ctx);
        assert!(mark.chars().all(is_wgl4), "{mark:?} is not WGL4");
    });
    with_ctx(GlyphMode::Ascii, |ctx| {
        assert!(available_mark(ctx).is_ascii());
    });
}

#[test]
fn the_legend_explains_the_focus_symbol_and_the_available_mark() {
    for mode in MODES {
        with_ctx(mode, |ctx| {
            let shown: String = legend::lines(LEGEND_WIDTH, ctx).iter().map(text).collect();
            let focus = format!(
                "{} {}",
                ctx.glyphs.focus_left,
                t("board.guide.legend_focus")
            );
            let available = format!(
                "{} {}",
                available_mark(ctx),
                t("board.guide.legend_available")
            );
            assert!(shown.contains(&focus), "no focus entry in {shown:?}");
            assert!(
                shown.contains(&available),
                "no available entry in {shown:?}"
            );
        });
    }
}
