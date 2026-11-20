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
