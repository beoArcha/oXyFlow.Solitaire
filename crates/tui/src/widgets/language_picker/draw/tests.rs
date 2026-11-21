use std::time::Instant;

use super::*;
use crate::audio::AudioStatus;
use crate::caps::TerminalCaps;
use crate::glyphs::Glyphs;
use crate::settings::{ColorMode, GlyphMode, Settings};
use crate::test_support::{PLAIN, TRUE_COLOR};
use crate::theme::{Theme, ThemeId};

struct Fixture {
    theme: Theme,
    glyphs: Glyphs,
    settings: Settings,
    caps: TerminalCaps,
}

impl Fixture {
    fn new(caps: TerminalCaps, settings: Settings) -> Self {
        Self {
            theme: Theme::new(ThemeId::Dark, caps.color),
            glyphs: Glyphs::new(settings.glyph_mode),
            settings,
            caps,
        }
    }

    fn ctx(&self) -> ViewCtx<'_> {
        ViewCtx {
            theme: &self.theme,
            glyphs: &self.glyphs,
            settings: &self.settings,
            caps: &self.caps,
            audio_status: AudioStatus::NoDevice,
            sound_styles: &[],
            saved: None,
            pointer: None,
            now: Instant::now(),
            font: crate::app::FontFacts::default(),
        }
    }
}

fn text_of(lines: &[Line<'_>]) -> Vec<String> {
    lines
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect()
        })
        .collect()
}

#[test]
fn every_locale_draws_its_flag_in_its_own_colors_with_its_code() {
    let fixture = Fixture::new(TRUE_COLOR, Settings::default());
    let ctx = fixture.ctx();
    assert!(flags_shown(&ctx));
    for locale in Locale::ALL {
        let pixels = flag(locale).pixels;
        let lines = selector_lines(locale, false, true, &ctx);
        let cells: Vec<&Span<'_>> = lines
            .iter()
            .flat_map(|line| line.spans.iter().filter(|span| span.content == "▀"))
            .collect();
        assert_eq!(cells.len(), PIXELS * 2, "{}", locale.id());
        for (index, cell) in cells.iter().enumerate() {
            let (half, column) = (index / PIXELS, index % PIXELS);
            let [r, g, b] = pixels[half * 2][column];
            let [r2, g2, b2] = pixels[half * 2 + 1][column];
            assert_eq!(cell.style.fg, Some(Color::Rgb(r, g, b)), "{}", locale.id());
            assert_eq!(
                cell.style.bg,
                Some(Color::Rgb(r2, g2, b2)),
                "{}",
                locale.id()
            );
        }
        let text = text_of(&lines);
        assert!(text[1].contains(flag(locale).code), "{}", locale.id());
    }
}

#[test]
fn without_unicode_or_at_sixteen_colors_every_locale_shows_its_code_instead() {
    let ansi16 = Settings {
        color_mode: ColorMode::Ansi16,
        ..Settings::default()
    };
    let ascii = Settings {
        glyph_mode: GlyphMode::Ascii,
        ..Settings::for_caps(&PLAIN)
    };
    for (caps, settings) in [(TRUE_COLOR, ansi16), (PLAIN, ascii)] {
        let fixture = Fixture::new(caps, settings);
        let ctx = fixture.ctx();
        assert!(!flags_shown(&ctx));
        for locale in Locale::ALL {
            let text = text_of(&selector_lines(locale, false, false, &ctx));
            assert_eq!(text.len(), 1);
            assert!(text[0].contains(flag(locale).code), "{}", locale.id());
            assert!(!text[0].contains('▀'));
        }
    }
}

#[test]
fn every_list_entry_names_the_language_in_itself_beside_its_code() {
    let fixture = Fixture::new(TRUE_COLOR, Settings::default());
    let ctx = fixture.ctx();
    for (locale, flags) in Locale::ALL
        .into_iter()
        .flat_map(|l| [(l, true), (l, false)])
    {
        let entry = Entry {
            locale,
            highlighted: false,
            current: false,
            width: 30,
            flags,
        };
        let text = text_of(&entry_lines(&entry, &ctx)).concat();
        assert!(text.contains(locale.own_name()), "{text}");
        assert!(text.contains(flag(locale).code), "{text}");
    }
}

fn ascii() -> Fixture {
    Fixture::new(
        PLAIN,
        Settings {
            glyph_mode: GlyphMode::Ascii,
            ..Settings::for_caps(&PLAIN)
        },
    )
}

#[test]
fn at_rest_the_unicode_selector_shows_only_the_flag_code_and_the_down_arrow() {
    let fixture = Fixture::new(TRUE_COLOR, Settings::default());
    let ctx = fixture.ctx();
    let flag_row = "▀".repeat(PIXELS);
    let text = text_of(&selector_lines(Locale::PlPl, false, true, &ctx));
    assert_eq!(text, [flag_row.clone(), format!("{flag_row} PL ▼")]);
}

#[test]
fn focused_unicode_selector_adds_the_side_arrows() {
    let fixture = Fixture::new(TRUE_COLOR, Settings::default());
    let ctx = fixture.ctx();
    let flag_row = "▀".repeat(PIXELS);
    let text = text_of(&selector_lines(Locale::PlPl, true, true, &ctx));
    assert_eq!(
        text,
        [format!("◄ {flag_row} ► ▼"), format!("  {flag_row} PL ")]
    );
}

#[test]
fn at_rest_the_ascii_selector_shows_the_code_and_a_v() {
    let fixture = ascii();
    let ctx = fixture.ctx();
    assert_eq!(
        text_of(&selector_lines(Locale::PlPl, false, false, &ctx)),
        ["PL v"]
    );
}

#[test]
fn focused_ascii_selector_adds_the_angle_arrows() {
    let fixture = ascii();
    let ctx = fixture.ctx();
    assert_eq!(
        text_of(&selector_lines(Locale::PlPl, true, false, &ctx)),
        ["< PL > v"]
    );
}

fn position_of(text: &[String], glyph: &str) -> Option<(u16, u16)> {
    text.iter().enumerate().find_map(|(row, line)| {
        let byte = line.find(glyph)?;
        let column = line[..byte].chars().count();
        Some((u16::try_from(column).ok()?, u16::try_from(row).ok()?))
    })
}

#[test]
fn every_arrow_drawn_is_a_mouse_target_and_the_hidden_ones_are_not() {
    for (fixture, flags) in [
        (Fixture::new(TRUE_COLOR, Settings::default()), true),
        (ascii(), false),
    ] {
        let ctx = fixture.ctx();
        let [left, right, down] = arrows(ctx.glyphs.unicode);
        for focused in [false, true] {
            let (width, height) = body_size(flags, focused);
            let origin = Rect::new(0, 0, width, height);
            let lines = selector_lines(Locale::PlPl, focused, flags, &ctx);
            let text = text_of(&lines);
            assert!(text.len() <= usize::from(height), "{text:?}");
            assert!(
                text.iter().all(|line| columns_of(line) <= width),
                "{text:?}"
            );
            let spots = locate(origin, flags, focused);
            let cases = [
                (left, spots.left, focused),
                (right, spots.right, focused),
                (down, spots.down, true),
            ];
            for (glyph, spot, shown) in cases {
                let drawn = position_of(&text, glyph);
                assert_eq!(drawn.is_some(), shown, "{glyph} in {text:?}");
                match drawn {
                    Some(cell) => assert!(contains(spot, cell), "{glyph} at {cell:?} in {text:?}"),
                    None => assert_eq!(spot.width, 0, "{glyph} is no target at rest"),
                }
            }
        }
    }
}

#[test]
fn the_list_dims_the_two_letter_codes_and_keeps_the_names_plain() {
    let fixture = Fixture::new(TRUE_COLOR, Settings::default());
    let ctx = fixture.ctx();
    let muted = ctx.theme.style(Token::TextMuted, Token::Surface);
    assert_ne!(muted.fg, ctx.theme.surface().fg);
    for flags in [true, false] {
        let entry = Entry {
            locale: Locale::PlPl,
            highlighted: false,
            current: false,
            width: 30,
            flags,
        };
        let lines = entry_lines(&entry, &ctx);
        let spans: Vec<&Span<'_>> = lines.iter().flat_map(|line| line.spans.iter()).collect();
        let code = spans.iter().find(|span| span.content == "PL");
        assert_eq!(code.map(|span| span.style), Some(muted), "flags {flags}");
        let name = spans.iter().find(|span| span.content.contains("Polski"));
        assert_eq!(
            name.map(|span| span.style),
            Some(ctx.theme.surface()),
            "flags {flags}"
        );
    }
}

#[test]
fn the_highlighted_list_entry_keeps_its_code_on_the_focus_colors() {
    let fixture = Fixture::new(TRUE_COLOR, Settings::default());
    let ctx = fixture.ctx();
    let entry = Entry {
        locale: Locale::PlPl,
        highlighted: true,
        current: false,
        width: 30,
        flags: false,
    };
    let lines = entry_lines(&entry, &ctx);
    let code = lines
        .iter()
        .flat_map(|line| line.spans.iter())
        .find(|span| span.content == "PL");
    assert_eq!(code.map(|span| span.style), Some(ctx.theme.focus()));
}
