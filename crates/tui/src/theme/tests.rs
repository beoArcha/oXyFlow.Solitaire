use super::contrast::contrast_ratio;
use super::*;
use crate::settings::CardBack;

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

#[test]
fn every_graphic_pair_reaches_three_to_one() {
    for id in ThemeId::ALL {
        for (fg, bg) in GRAPHIC_PAIRS {
            let ratio = contrast_ratio(rgb(id, fg), rgb(id, bg));
            assert!(ratio >= 3.0, "{id:?}: {fg:?} on {bg:?} is {ratio}");
        }
    }
}

#[test]
fn a_theme_without_the_field_frame_has_a_felt_that_reaches_three_to_one_on_the_background() {
    let framed: Vec<ThemeId> = ThemeId::ALL
        .into_iter()
        .filter(|id| id.palette().field_edge)
        .collect();
    assert!(framed.len() < ThemeId::ALL.len());
    for id in ThemeId::ALL.into_iter().filter(|id| !framed.contains(id)) {
        assert_eq!(
            id.palette().field_edge_pair(),
            (Token::Table, Token::Background)
        );
        let ratio = contrast_ratio(rgb(id, Token::Table), rgb(id, Token::Background));
        assert!(
            ratio >= 3.0,
            "{id:?}: the felt on the background is {ratio}"
        );
    }
}

#[test]
fn a_theme_with_the_field_frame_has_a_border_that_reaches_three_to_one_on_the_background() {
    for id in ThemeId::ALL
        .into_iter()
        .filter(|id| id.palette().field_edge)
    {
        assert_eq!(
            id.palette().field_edge_pair(),
            (Token::Border, Token::Background)
        );
        let ratio = contrast_ratio(rgb(id, Token::Border), rgb(id, Token::Background));
        assert!(ratio >= 3.0, "{id:?}: the field frame is {ratio}");
    }
}

#[test]
fn only_a_theme_whose_felt_is_the_background_draws_the_field_frame() {
    for id in ThemeId::ALL {
        let felt = contrast_ratio(rgb(id, Token::Table), rgb(id, Token::Background));
        assert_eq!(
            id.palette().field_edge,
            felt < 3.0,
            "{id:?}: felt is {felt}"
        );
    }
    assert!(ThemeId::HighContrast.palette().field_edge);
    assert!(!ThemeId::Dark.palette().field_edge);
    assert!(!ThemeId::Light.palette().field_edge);
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

#[test]
fn card_back_count_mark_reaches_contrast_target_on_every_back() {
    for id in ThemeId::ALL {
        let (fg, bg) = (rgb(id, Token::CardBackMark), rgb(id, Token::CardBack));
        let ratio = contrast_ratio(fg, bg);
        assert!(
            ratio >= 4.5,
            "{id:?}: theme card back count mark contrast is {ratio}"
        );
    }
    let monokai_ratio = contrast_ratio(
        rgb(ThemeId::Monokai, Token::CardBackMark),
        rgb(ThemeId::Monokai, Token::CardBack),
    );
    assert!(
        monokai_ratio >= 4.5,
        "Monokai card back count mark contrast is {monokai_ratio}"
    );
    let dracula_ratio = contrast_ratio(
        rgb(ThemeId::Dracula, Token::CardBackMark),
        rgb(ThemeId::Dracula, Token::CardBack),
    );
    assert!(
        dracula_ratio >= 4.5,
        "Dracula card back count mark contrast is {dracula_ratio}"
    );
}
