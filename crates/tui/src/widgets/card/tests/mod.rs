//! Tests of the card widget. This module holds the fixtures; the tests live in the children.

mod board;
mod dump;
mod faces;
mod marks;
mod slots;

use std::time::Instant;

use oxysol_core::{Card, CardId, Rank, Suit};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::*;
use crate::audio::AudioStatus;
use crate::glyphs::Glyphs;
use crate::i18n::{Locale, set_locale};
use crate::screens::ViewCtx;
use crate::settings::{GlyphMode, Settings};
use crate::test_support::TRUE_COLOR;
use crate::theme::{Theme, ThemeId};

const SIZES: [(u16, u16); 4] = [(13, 11), (11, 9), (9, 7), (7, 5)];
const MODES: [GlyphMode; 2] = [GlyphMode::Unicode, GlyphMode::Ascii];
const MARGIN: u16 = 2;
const SENTINEL: &str = "?";

struct Fixture {
    theme: Theme,
    glyphs: Glyphs,
    settings: Settings,
}

impl Fixture {
    fn new(mode: GlyphMode) -> Self {
        Self {
            theme: Theme::new(ThemeId::Dark, TRUE_COLOR.color),
            glyphs: Glyphs::new(mode),
            settings: Settings::default(),
        }
    }

    fn ctx(&self) -> ViewCtx<'_> {
        ViewCtx {
            theme: &self.theme,
            glyphs: &self.glyphs,
            settings: &self.settings,
            caps: &TRUE_COLOR,
            audio_status: AudioStatus::NoDevice,
            sound_styles: &[],
            has_saved_game: false,
            now: Instant::now(),
        }
    }
}

fn card(rank: Rank, suit: Suit) -> Card {
    Card {
        id: CardId(0),
        suit,
        rank,
    }
}

fn each_locale(mut check: impl FnMut(Locale)) {
    for locale in Locale::ALL {
        set_locale(locale);
        check(locale);
    }
    set_locale(Locale::EnUs);
}

fn every_card() -> Vec<Card> {
    Rank::ALL
        .iter()
        .flat_map(|rank| Suit::ALL.map(|suit| card(*rank, suit)))
        .collect()
}

fn padded_face(
    mode: GlyphMode,
    size: (u16, u16),
    (face, mark, badge): (Face, Mark, Option<usize>),
) -> (Buffer, Rect) {
    let fixture = Fixture::new(mode);
    let whole = Rect::new(0, 0, size.0 + 2 * MARGIN, size.1 + 2 * MARGIN);
    let area = Rect::new(MARGIN, MARGIN, size.0, size.1);
    let mut buf = Buffer::empty(whole);
    for y in whole.top()..whole.bottom() {
        for x in whole.left()..whole.right() {
            buf[(x, y)].set_symbol(SENTINEL);
        }
    }
    draw(&mut buf, area, face, mark, badge, &fixture.ctx());
    (buf, area)
}

fn padded(mode: GlyphMode, size: (u16, u16), card: Card) -> (Buffer, Rect) {
    padded_face(mode, size, (Face::Up(card), Mark::default(), None))
}

fn rows(buf: &Buffer, area: Rect) -> Vec<String> {
    (area.top()..area.bottom())
        .map(|y| {
            (area.left()..area.right())
                .map(|x| buf[(x, y)].symbol())
                .collect()
        })
        .collect()
}

fn render(mode: GlyphMode, size: (u16, u16), card: Card) -> Vec<String> {
    let (buf, area) = padded(mode, size, card);
    rows(&buf, area)
}

fn render_face(
    mode: GlyphMode,
    size: (u16, u16),
    face: Face,
    (mark, badge): (Mark, Option<usize>),
) -> Vec<String> {
    let (buf, area) = padded_face(mode, size, (face, mark, badge));
    rows(&buf, area)
}

fn interior(lines: &[String]) -> Vec<String> {
    let last = lines.len() - 1;
    lines[1..last]
        .iter()
        .map(|line| {
            line.chars()
                .skip(1)
                .take(line.chars().count() - 2)
                .collect()
        })
        .collect()
}
