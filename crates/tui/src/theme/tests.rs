use super::contrast::contrast_ratio;
use super::*;

fn rgb(theme: ThemeId, token: Token) -> [u8; 3] {
    theme.palette().swatch(token).rgb
}

#[test]
fn every_text_pair_reaches_the_theme_target() {
    for id in ThemeId::ALL {
        for (fg, bg) in TEXT_PAIRS {
            let ratio = contrast_ratio(rgb(id, fg), rgb(id, bg));
            assert!(
                ratio >= id.contrast_target(),
                "{id:?}: {fg:?} on {bg:?} is {ratio}"
            );
        }
    }
}

/// Pairs that may stay below 3:1 in one theme. High Contrast keeps its black field: its white text
/// on `Table` must reach 7:1, which caps the luminance of `Table` at 0.1, while the field must
/// reach 3:1 against the black `Background`, which needs at least 0.1. Only a light field with dark
/// text would fix both, and that also moves `BorderFocus`, which dialogs use on the black `Surface`.
/// See `docs/tech/ui-inventory.md` §1.4 and §7.1.
const GRAPHIC_GAPS: [(ThemeId, Token, Token); 1] =
    [(ThemeId::HighContrast, Token::Table, Token::Background)];

#[test]
fn every_graphic_pair_reaches_three_to_one() {
    for id in ThemeId::ALL {
        for (fg, bg) in GRAPHIC_PAIRS {
            if GRAPHIC_GAPS.contains(&(id, fg, bg)) {
                continue;
            }
            let ratio = contrast_ratio(rgb(id, fg), rgb(id, bg));
            assert!(ratio >= 3.0, "{id:?}: {fg:?} on {bg:?} is {ratio}");
        }
    }
}

#[test]
fn every_mark_pair_reaches_three_to_one() {
    for id in ThemeId::ALL {
        for (fg, bg) in MARK_PAIRS {
            let ratio = contrast_ratio(rgb(id, fg), rgb(id, bg));
            assert!(ratio >= 3.0, "{id:?}: {fg:?} on {bg:?} is {ratio}");
        }
    }
}

#[test]
fn red_and_black_cards_differ_in_every_variant() {
    for id in ThemeId::ALL {
        let palette = id.palette();
        assert_ne!(palette.card_red.rgb, palette.card_black.rgb, "{id:?} rgb");
        assert_ne!(
            palette.card_red.ansi16, palette.card_black.ansi16,
            "{id:?} ansi16"
        );
    }
}

#[test]
fn focus_differs_from_plain_text_in_every_variant() {
    for id in ThemeId::ALL {
        let palette = id.palette();
        assert_ne!(palette.focus_bg.rgb, palette.background.rgb, "{id:?}");
        assert_ne!(palette.focus_bg.ansi16, palette.background.ansi16, "{id:?}");
    }
}

#[test]
fn resolution_follows_the_color_depth() {
    let swatch = dark::palette().accent;
    let [r, g, b] = swatch.rgb;
    assert_eq!(swatch.resolve(ColorDepth::TrueColor), Color::Rgb(r, g, b));
    assert_eq!(
        swatch.resolve(ColorDepth::Ansi256),
        Color::Indexed(ansi::rgb_to_ansi256(r, g, b))
    );
    assert_eq!(swatch.resolve(ColorDepth::Ansi16), swatch.ansi16);
}

#[test]
fn sixteen_color_variants_use_only_named_colors() {
    for id in ThemeId::ALL {
        for token in Token::ALL {
            let color = id.palette().swatch(token).ansi16;
            assert!(
                !matches!(color, Color::Rgb(..) | Color::Indexed(_) | Color::Reset),
                "{id:?} {token:?}"
            );
        }
    }
}

#[test]
fn theme_resolves_every_token_for_every_depth() {
    for id in ThemeId::ALL {
        for depth in [
            ColorDepth::Ansi16,
            ColorDepth::Ansi256,
            ColorDepth::TrueColor,
        ] {
            let theme = Theme::new(id, depth);
            assert_eq!(theme.id(), id);
            for token in Token::ALL {
                assert_ne!(theme.color(token), Color::Reset);
            }
        }
    }
}

#[test]
fn styles_carry_the_requested_tokens() {
    let theme = Theme::new(ThemeId::Dark, ColorDepth::TrueColor);
    let style = theme.style(Token::Error, Token::Surface);
    assert_eq!(style.fg, Some(theme.color(Token::Error)));
    assert_eq!(style.bg, Some(theme.color(Token::Surface)));
}

#[test]
fn theme_ids_serialize_as_snake_case() {
    let text = toml::to_string(&std::collections::BTreeMap::from([(
        "theme",
        ThemeId::HighContrast,
    )]));
    assert!(matches!(text, Ok(ref t) if t.contains("high_contrast")));
}

#[test]
fn the_aura_pairs_are_checked_for_contrast_and_differ() {
    assert!(TEXT_PAIRS.contains(&(Token::AuraFocus, Token::Table)));
    assert!(TEXT_PAIRS.contains(&(Token::AuraCarry, Token::Table)));
    for id in ThemeId::ALL {
        let palette = id.palette();
        assert_ne!(palette.aura_focus.rgb, palette.aura_carry.rgb, "{id:?} rgb");
        assert_ne!(
            palette.aura_focus.ansi16, palette.aura_carry.ansi16,
            "{id:?} ansi16"
        );
        assert_ne!(palette.aura_focus.ansi16, palette.table.ansi16, "{id:?}");
        assert_ne!(palette.aura_carry.ansi16, palette.table.ansi16, "{id:?}");
    }
}
