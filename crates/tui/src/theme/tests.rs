use super::contrast::contrast_ratio;
use super::vision::{delta_e, remap, simulate};
use super::*;
use crate::settings::{CardFaceTint, ColorVision};

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

/// The Theme, Monokai and Dracula backs draw from these palettes, so every palette's back mark is checked.
#[test]
fn card_back_count_mark_reaches_text_target_on_every_back() {
    for id in ThemeId::ALL {
        let ratio = contrast_ratio(rgb(id, Token::CardBackMark), rgb(id, Token::CardBack));
        assert!(
            ratio >= id.contrast_target(),
            "{id:?}: card back count mark is {ratio}"
        );
    }
}

#[test]
fn every_card_face_tint_keeps_both_suits_at_the_contrast_target_on_its_face() {
    for id in ThemeId::ALL {
        for tint in CardFaceTint::ALL {
            let colors = face::face_colors(tint, &id.palette());
            let target = if tint == CardFaceTint::Theme {
                id.contrast_target()
            } else {
                7.0
            };
            for (name, ink) in [("red", colors.red), ("black", colors.black)] {
                let ratio = contrast_ratio(ink.rgb, colors.face.rgb);
                assert!(
                    ratio >= target,
                    "{id:?} {tint:?}: {name} on the face is {ratio}"
                );
            }
        }
    }
}

#[test]
fn every_card_face_tint_tells_the_suits_apart_in_rgb_and_in_sixteen_colors() {
    for id in ThemeId::ALL {
        for tint in CardFaceTint::ALL {
            let colors = face::face_colors(tint, &id.palette());
            assert_ne!(colors.red.rgb, colors.black.rgb, "{id:?} {tint:?} rgb");
            assert_ne!(
                colors.red.ansi16, colors.black.ansi16,
                "{id:?} {tint:?} ansi16"
            );
        }
    }
}

#[test]
fn the_card_face_style_is_the_theme_face_for_the_theme_tint_and_the_tint_colors_otherwise() {
    for id in ThemeId::ALL {
        let theme = Theme::new(id, ColorDepth::TrueColor);
        for suit in [Token::CardRed, Token::CardBlack] {
            assert_eq!(
                theme.card_face_style(CardFaceTint::Theme, suit),
                theme.style(suit, Token::CardFace)
            );
        }
    }
    let theme = Theme::new(ThemeId::Dark, ColorDepth::TrueColor);
    let colors = face::face_colors(CardFaceTint::Dark, &ThemeId::Dark.palette());
    let style = theme.card_face_style(CardFaceTint::Dark, Token::CardBlack);
    assert_eq!(style.bg, Some(colors.face.resolve(ColorDepth::TrueColor)));
    assert_eq!(style.fg, Some(colors.black.resolve(ColorDepth::TrueColor)));
}

#[test]
fn every_color_vision_mode_keeps_contrast_targets() {
    for id in ThemeId::ALL {
        for vision in ColorVision::ALL {
            let palette = remap(id, id.palette(), vision);
            for (fg, bg) in TEXT_PAIRS {
                let ratio = contrast_ratio(palette.swatch(fg).rgb, palette.swatch(bg).rgb);
                assert!(
                    ratio >= id.contrast_target(),
                    "{id:?} {vision:?}: {fg:?} on {bg:?} is {ratio}"
                );
            }
            for (fg, bg) in GRAPHIC_PAIRS.into_iter().chain(MARK_PAIRS) {
                let ratio = contrast_ratio(palette.swatch(fg).rgb, palette.swatch(bg).rgb);
                assert!(
                    ratio >= 3.0,
                    "{id:?} {vision:?}: {fg:?} on {bg:?} is {ratio}"
                );
            }
        }
    }
}

fn assert_simulated_pair(
    id: ThemeId,
    vision: ColorVision,
    (a, b): (Token, Token),
    palette: &Palette,
) {
    let min_delta = if vision == ColorVision::Monochrome {
        match (a, b) {
            (Token::AuraFocus, Token::AuraCarry) => 1.5,
            (Token::Selection, Token::Target) => 8.0,
            _ => 10.0,
        }
    } else {
        12.0
    };
    let sim_a = simulate(palette.swatch(a).rgb, vision);
    let sim_b = simulate(palette.swatch(b).rgb, vision);
    let diff = delta_e(sim_a, sim_b);
    assert!(
        diff >= min_delta,
        "{id:?} {vision:?}: {a:?} vs {b:?} simulated delta_e = {diff}"
    );
}

#[test]
fn simulated_meaning_tokens_maintain_minimum_color_difference() {
    let pairs = [
        (Token::Success, Token::Error),
        (Token::Target, Token::Hint),
        (Token::Selection, Token::Target),
        (Token::AuraFocus, Token::AuraCarry),
        (Token::CardRed, Token::CardBlack),
    ];
    for id in ThemeId::ALL {
        for vision in ColorVision::ALL {
            let palette = remap(id, id.palette(), vision);
            for pair in pairs {
                assert_simulated_pair(id, vision, pair, &palette);
            }
        }
    }
}

fn assert_four_color_tint(id: ThemeId, vision: ColorVision, tint: CardFaceTint, palette: &Palette) {
    let suits = [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs];
    let min_delta = if vision == ColorVision::Monochrome {
        7.0
    } else {
        12.0
    };
    let colors = face::suit_colors(tint, palette, vision);
    let target = if tint == CardFaceTint::Theme {
        id.contrast_target()
    } else {
        7.0
    };
    for suit in suits {
        let ratio = contrast_ratio(colors.for_suit(suit).rgb, colors.face.rgb);
        assert!(
            ratio >= target,
            "{id:?} {vision:?} {tint:?} {suit:?}: contrast = {ratio}"
        );
    }
    for (i, &a) in suits.iter().enumerate() {
        for &b in &suits[i + 1..] {
            let sim_a = simulate(colors.for_suit(a).rgb, vision);
            let sim_b = simulate(colors.for_suit(b).rgb, vision);
            let diff = delta_e(sim_a, sim_b);
            assert!(
                diff >= min_delta,
                "{id:?} {vision:?} {tint:?}: {a:?} vs {b:?} delta_e = {diff}"
            );
        }
    }
}

#[test]
fn four_color_suits_keep_contrast_and_distinctness_under_simulation() {
    for id in ThemeId::ALL {
        for vision in ColorVision::ALL {
            let palette = remap(id, id.palette(), vision);
            for tint in CardFaceTint::ALL {
                assert_four_color_tint(id, vision, tint, &palette);
            }
        }
    }
}
