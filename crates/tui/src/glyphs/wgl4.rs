//! Validation and test suite for Windows Glyph List 4 (WGL4) character support.

/// Returns whether a character is part of the Windows Glyph List 4 (WGL4) character set.
pub const fn is_wgl4(c: char) -> bool {
    matches!(
        c,
        '\u{0020}'..='\u{007E}'
            | '\u{00A0}'..='\u{00FF}'
            | '\u{0100}'..='\u{017F}'
            | '\u{0192}'
            | '\u{02C6}'
            | '\u{02DC}'
            | '\u{0386}'..='\u{03CE}'
            | '\u{0401}'..='\u{044F}'
            | '\u{0451}'..='\u{045C}'
            | '\u{045E}'..='\u{045F}'
            | '\u{0490}'..='\u{0491}'
            | '\u{2013}'
            | '\u{2014}'
            | '\u{2018}'..='\u{201E}'
            | '\u{2020}'..='\u{2022}'
            | '\u{2026}'
            | '\u{2030}'
            | '\u{2039}'
            | '\u{203A}'
            | '\u{20AC}'
            | '\u{2122}'
            | '\u{2190}'..='\u{2195}'
            | '\u{21A8}'
            | '\u{2215}'
            | '\u{2219}'
            | '\u{221A}'
            | '\u{221E}'
            | '\u{2229}'
            | '\u{2248}'
            | '\u{2260}'
            | '\u{2261}'
            | '\u{2264}'
            | '\u{2265}'
            | '\u{2310}'
            | '\u{2320}'
            | '\u{2321}'
            | '─'
            | '│'
            | '┌'
            | '┐'
            | '└'
            | '┘'
            | '├'
            | '┤'
            | '┬'
            | '┴'
            | '┼'
            | '\u{2550}'..='\u{256C}'
            | '▀'
            | '▄'
            | '█'
            | '▌'
            | '▐'
            | '░'
            | '▒'
            | '▓'
            | '■'
            | '□'
            | '▪'
            | '▫'
            | '▬'
            | '▲'
            | '►'
            | '▼'
            | '◄'
            | '◊'
            | '○'
            | '●'
            | '◘'
            | '◙'
            | '◦'
            | '\u{263A}'..='\u{263C}'
            | '♀'
            | '♂'
            | '♠'
            | '♣'
            | '♥'
            | '♦'
            | '♪'
            | '♫'
            | '✓'
            | '✗'
            | '★'
            | '\n'
            | '\r'
            | '\t'
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::AudioStatus;
    use crate::glyphs::Glyphs;
    use crate::layout::{chrome, compute};
    use crate::screens::ViewCtx;
    use crate::settings::{GlyphMode, Settings};
    use crate::test_support::{Harness, TRUE_COLOR};
    use crate::theme::{Theme, ThemeId};
    use crate::widgets::card::{self, Face, Hint, Mark, Placeholder};
    use crate::widgets::pile::{self, CardPos, Marks};
    use oxysol_core::{PileRef, Variant, deal};
    use ratatui::buffer::Buffer;
    use ratatui::crossterm::event::KeyCode;
    use ratatui::layout::Rect;
    use std::time::Instant;

    fn assert_buffer_wgl4(buf: &Buffer) {
        let area = buf.area;
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let sym = buf[(x, y)].symbol();
                for ch in sym.chars() {
                    assert!(
                        is_wgl4(ch),
                        "char {ch:?} (U+{:04X}) at ({x}, {y}) outside WGL4",
                        ch as u32
                    );
                }
            }
        }
    }

    fn test_ctx<'a>(glyphs: &'a Glyphs, theme: &'a Theme, settings: &'a Settings) -> ViewCtx<'a> {
        ViewCtx {
            theme,
            glyphs,
            settings,
            caps: &TRUE_COLOR,
            audio_status: AudioStatus::NoDevice,
            sound_styles: &[],
            saved: None,
            pointer: None,
            now: Instant::now(),
            font: crate::app::FontFacts::default(),
        }
    }

    #[test]
    fn every_unicode_glyph_field_is_wgl4() {
        let glyphs = Glyphs::new(GlyphMode::Unicode);
        for suit in glyphs.suits {
            for ch in suit.chars() {
                assert!(is_wgl4(ch), "suit {suit:?} has non-WGL4 char {ch:?}");
            }
        }
        for ch in glyphs.focus_left.chars() {
            assert!(is_wgl4(ch), "focus_left has non-WGL4 char {ch:?}");
        }
        for ch in glyphs.focus_right.chars() {
            assert!(is_wgl4(ch), "focus_right has non-WGL4 char {ch:?}");
        }
        for ch in glyphs.selected.chars() {
            assert!(is_wgl4(ch), "selected has non-WGL4 char {ch:?}");
        }
        for ch in glyphs.face_down.chars() {
            assert!(is_wgl4(ch), "face_down has non-WGL4 char {ch:?}");
        }
        for (name, symbol) in [
            ("target", glyphs.target),
            ("hint_source", glyphs.hint_source),
            ("hint_target", glyphs.hint_target),
            ("recycle", glyphs.recycle),
            ("spent", glyphs.spent),
            ("aura_focus", glyphs.aura_focus),
            ("aura_carry", glyphs.aura_carry),
            ("aura_target", glyphs.aura_target),
            ("edge", glyphs.edge),
            ("scroll_track", glyphs.scroll_track),
            ("scroll_thumb", glyphs.scroll_thumb),
            ("entry_on", glyphs.entry_on),
            ("entry_off", glyphs.entry_off),
            ("proof_winnable", glyphs.proof_winnable),
            ("proof_unwinnable", glyphs.proof_unwinnable),
            ("proof_unknown", glyphs.proof_unknown),
            ("proof_unchecked", glyphs.proof_unchecked),
        ] {
            for ch in symbol.chars() {
                assert!(is_wgl4(ch), "{name} has non-WGL4 char {ch:?}");
            }
        }
    }

    #[test]
    fn rejects_non_wgl4_glyphs() {
        let non_wgl4 = [
            '▶', '◀', '╭', '╮', '╰', '╯', '┏', '┓', '┗', '┛', '━', '┃', '┄', '┆', '╌', '╎',
        ];
        for ch in non_wgl4 {
            assert!(!is_wgl4(ch), "glyph {ch:?} should be rejected");
        }
    }

    #[test]
    fn main_menu_and_help_are_all_wgl4() {
        let mut h = Harness::new();
        h.lines();
        assert_buffer_wgl4(h.terminal.backend().buffer());
        h.press(KeyCode::Char('?'));
        h.lines();
        assert_buffer_wgl4(h.terminal.backend().buffer());
    }

    #[test]
    fn settings_screen_is_all_wgl4() {
        let mut h = Harness::new();
        h.press(KeyCode::Down);
        h.press(KeyCode::Down);
        h.press(KeyCode::Enter);
        h.lines();
        assert_buffer_wgl4(h.terminal.backend().buffer());
    }

    #[test]
    fn game_selection_screen_is_all_wgl4() {
        let mut h = Harness::new();
        h.press(KeyCode::Enter);
        h.lines();
        assert_buffer_wgl4(h.terminal.backend().buffer());
    }

    #[test]
    fn board_screens_for_every_variant_are_all_wgl4() {
        for (index, _) in Variant::ALL.into_iter().enumerate() {
            let mut h = Harness::new();
            h.press(KeyCode::Enter);
            for _ in 0..index {
                h.press(KeyCode::Down);
            }
            h.press(KeyCode::Enter);
            h.lines();
            assert_buffer_wgl4(h.terminal.backend().buffer());
            h.type_char('h');
            h.lines();
            assert_buffer_wgl4(h.terminal.backend().buffer());
            h.press(KeyCode::Enter);
            h.lines();
            assert_buffer_wgl4(h.terminal.backend().buffer());
        }
    }

    #[test]
    fn all_mark_states_and_borders_are_all_wgl4() {
        let glyphs = Glyphs::new(GlyphMode::Unicode);
        let theme = Theme::new(ThemeId::Dark, TRUE_COLOR.color);
        let settings = Settings::default();
        let ctx = test_ctx(&glyphs, &theme, &settings);
        let area = Rect::new(0, 0, 7, 5);
        let sample_card = deal(Variant::FreeCell, 1).piles()[8].cards[0].card;
        let mark_cases = [
            Mark::default(),
            Mark {
                focus: true,
                ..Mark::default()
            },
            Mark {
                selected: true,
                ..Mark::default()
            },
            Mark {
                target: true,
                ..Mark::default()
            },
            Mark {
                hint: Some(Hint::Source),
                ..Mark::default()
            },
            Mark {
                hint: Some(Hint::Target),
                ..Mark::default()
            },
            Mark {
                focus: true,
                selected: true,
                target: true,
                hint: Some(Hint::Target),
                blocked: false,
            },
        ];
        for mark in mark_cases {
            let mut buf = Buffer::empty(area);
            card::draw(&mut buf, area, Face::Up(sample_card), mark, Some(42), &ctx);
            assert_buffer_wgl4(&buf);
            for placeholder in [
                Placeholder::None,
                Placeholder::Ace,
                Placeholder::Recycle,
                Placeholder::Spent,
            ] {
                let mut slot_buf = Buffer::empty(area);
                let face = Face::Slot(placeholder);
                card::draw(&mut slot_buf, area, face, mark, None, &ctx);
                assert_buffer_wgl4(&slot_buf);
            }
            let mut down_buf = Buffer::empty(area);
            card::draw(&mut down_buf, area, Face::Down, mark, None, &ctx);
            assert_buffer_wgl4(&down_buf);
        }
    }

    #[test]
    fn pile_rendering_with_marks_for_every_variant_is_all_wgl4() {
        let glyphs = Glyphs::new(GlyphMode::Unicode);
        let theme = Theme::new(ThemeId::Dark, TRUE_COLOR.color);
        let settings = Settings::default();
        let ctx = test_ctx(&glyphs, &theme, &settings);
        let bounds = Rect::new(0, 0, 100, 30);
        for variant in Variant::ALL {
            let piles = deal(variant, 7).piles();
            let body = chrome(bounds).body;
            let geometry = compute(variant, &piles, body);
            let target_pile = piles.first().map(|p| p.pile).unwrap_or(PileRef::Stock);
            let marks = Marks {
                blocked: Vec::new(),
                focus: Some(CardPos {
                    pile: target_pile,
                    index: 0,
                }),
                selected: Some(CardPos {
                    pile: target_pile,
                    index: 0,
                }),
                targets: vec![target_pile],
                hints: vec![CardPos {
                    pile: target_pile,
                    index: 0,
                }],
            };
            let mut buf = Buffer::empty(bounds);
            for (snapshot, placed) in piles.iter().zip(&geometry.piles) {
                pile::draw(&mut buf, snapshot, placed, &marks, true, &ctx);
            }
            assert_buffer_wgl4(&buf);
        }
    }
}
