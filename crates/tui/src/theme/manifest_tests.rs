use ratatui::style::Color;

use super::manifest::{builtin_toml, parse_palette, validate_contrast};
use super::palette::swatch;
use super::{Palette, Theme, ThemeError, ThemeId, Token, UserTheme};
use crate::caps::ColorDepth;
use crate::settings::{CardBack, ColorVision, TableColour};

fn expected_dark() -> Palette {
    Palette {
        background: swatch(24, 28, 38, Color::Black),
        surface: swatch(36, 42, 58, Color::Black),
        text: swatch(232, 236, 244, Color::White),
        text_muted: swatch(184, 194, 214, Color::Gray),
        accent: swatch(140, 190, 255, Color::LightCyan),
        border: swatch(110, 122, 150, Color::Gray),
        border_focus: swatch(255, 214, 102, Color::LightYellow),
        focus_fg: swatch(20, 24, 32, Color::Black),
        focus_bg: swatch(255, 214, 102, Color::LightYellow),
        error: swatch(255, 160, 150, Color::LightRed),
        success: swatch(150, 230, 160, Color::LightGreen),
        table: swatch(40, 120, 80, Color::Green),
        table_text: swatch(236, 246, 240, Color::White),
        slot: swatch(170, 220, 190, Color::LightGreen),
        card_face: swatch(250, 250, 244, Color::White),
        card_back: swatch(184, 208, 248, Color::White),
        card_back_mark: swatch(16, 20, 40, Color::Black),
        card_red: swatch(166, 20, 20, Color::Red),
        card_black: swatch(16, 16, 20, Color::Black),
        target: swatch(255, 214, 102, Color::LightYellow),
        selection: swatch(150, 220, 255, Color::LightCyan),
        hint: swatch(255, 196, 110, Color::LightYellow),
        aura_focus: swatch(255, 246, 184, Color::LightYellow),
        aura_carry: swatch(210, 244, 255, Color::LightCyan),
        field_edge: false,
    }
}

fn expected_light() -> Palette {
    Palette {
        background: swatch(244, 246, 248, Color::White),
        surface: swatch(255, 255, 255, Color::White),
        text: swatch(24, 28, 36, Color::Black),
        text_muted: swatch(96, 104, 116, Color::DarkGray),
        accent: swatch(18, 76, 160, Color::Blue),
        border: swatch(140, 146, 158, Color::Gray),
        border_focus: swatch(0, 12, 40, Color::Black),
        focus_fg: swatch(255, 255, 255, Color::White),
        focus_bg: swatch(28, 92, 196, Color::Blue),
        error: swatch(180, 24, 24, Color::Red),
        success: swatch(20, 116, 48, Color::Green),
        table: swatch(32, 114, 70, Color::Green),
        table_text: swatch(245, 252, 248, Color::White),
        slot: swatch(200, 230, 210, Color::White),
        card_face: swatch(255, 255, 255, Color::White),
        card_back: swatch(0, 14, 46, Color::Black),
        card_back_mark: swatch(230, 240, 255, Color::White),
        card_red: swatch(176, 24, 24, Color::Red),
        card_black: swatch(20, 20, 24, Color::Black),
        target: swatch(255, 214, 102, Color::Yellow),
        selection: swatch(80, 200, 255, Color::LightCyan),
        hint: swatch(255, 186, 70, Color::LightYellow),
        aura_focus: swatch(255, 236, 140, Color::Yellow),
        aura_carry: swatch(172, 240, 255, Color::LightCyan),
        field_edge: false,
    }
}

fn expected_high_contrast() -> Palette {
    Palette {
        background: swatch(0, 0, 0, Color::Black),
        surface: swatch(16, 16, 16, Color::Black),
        text: swatch(255, 255, 255, Color::White),
        text_muted: swatch(255, 255, 255, Color::White),
        accent: swatch(0, 255, 255, Color::Cyan),
        border: swatch(255, 255, 255, Color::White),
        border_focus: swatch(255, 255, 0, Color::Yellow),
        focus_fg: swatch(0, 0, 0, Color::Black),
        focus_bg: swatch(255, 255, 0, Color::Yellow),
        error: swatch(255, 128, 128, Color::LightRed),
        success: swatch(0, 255, 0, Color::LightGreen),
        table: swatch(0, 0, 0, Color::Black),
        table_text: swatch(255, 255, 255, Color::White),
        slot: swatch(255, 255, 255, Color::White),
        card_face: swatch(255, 255, 255, Color::White),
        card_back: swatch(0, 160, 255, Color::LightBlue),
        card_back_mark: swatch(0, 0, 0, Color::Black),
        card_red: swatch(160, 0, 0, Color::Red),
        card_black: swatch(0, 0, 0, Color::Black),
        target: swatch(0, 255, 0, Color::LightGreen),
        selection: swatch(0, 255, 255, Color::LightCyan),
        hint: swatch(255, 255, 0, Color::LightYellow),
        aura_focus: swatch(255, 255, 0, Color::Yellow),
        aura_carry: swatch(0, 255, 255, Color::LightCyan),
        field_edge: true,
    }
}

fn expected_monokai() -> Palette {
    Palette {
        background: swatch(39, 40, 34, Color::Black),
        surface: swatch(46, 47, 39, Color::Black),
        text: swatch(248, 248, 242, Color::White),
        text_muted: swatch(185, 182, 164, Color::Gray),
        accent: swatch(102, 217, 239, Color::LightCyan),
        border: swatch(130, 126, 105, Color::Gray),
        border_focus: swatch(230, 219, 116, Color::LightYellow),
        focus_fg: swatch(39, 40, 34, Color::Black),
        focus_bg: swatch(230, 219, 116, Color::LightYellow),
        error: swatch(255, 110, 150, Color::LightRed),
        success: swatch(166, 226, 46, Color::LightGreen),
        table: swatch(46, 128, 70, Color::Green),
        table_text: swatch(255, 255, 255, Color::White),
        slot: swatch(166, 226, 46, Color::LightGreen),
        card_face: swatch(248, 248, 242, Color::White),
        card_back: swatch(30, 31, 26, Color::Black),
        card_back_mark: swatch(230, 219, 116, Color::LightYellow),
        card_red: swatch(180, 24, 48, Color::Red),
        card_black: swatch(39, 40, 34, Color::Black),
        target: swatch(230, 219, 116, Color::LightYellow),
        selection: swatch(118, 224, 244, Color::LightCyan),
        hint: swatch(255, 198, 70, Color::LightYellow),
        aura_focus: swatch(255, 255, 180, Color::LightYellow),
        aura_carry: swatch(210, 255, 255, Color::LightCyan),
        field_edge: false,
    }
}

fn expected_dracula() -> Palette {
    Palette {
        background: swatch(40, 42, 54, Color::Black),
        surface: swatch(68, 71, 90, Color::Black),
        text: swatch(248, 248, 242, Color::White),
        text_muted: swatch(190, 195, 225, Color::Gray),
        accent: swatch(139, 233, 253, Color::LightCyan),
        border: swatch(155, 165, 205, Color::Gray),
        border_focus: swatch(241, 250, 140, Color::LightYellow),
        focus_fg: swatch(40, 42, 54, Color::Black),
        focus_bg: swatch(241, 250, 140, Color::LightYellow),
        error: swatch(255, 155, 155, Color::LightRed),
        success: swatch(80, 250, 123, Color::LightGreen),
        table: swatch(40, 130, 94, Color::Green),
        table_text: swatch(255, 255, 255, Color::White),
        slot: swatch(140, 245, 180, Color::LightGreen),
        card_face: swatch(248, 248, 242, Color::White),
        card_back: swatch(30, 32, 42, Color::Black),
        card_back_mark: swatch(139, 233, 253, Color::LightCyan),
        card_red: swatch(185, 25, 30, Color::Red),
        card_black: swatch(40, 42, 54, Color::Black),
        target: swatch(80, 250, 123, Color::LightGreen),
        selection: swatch(139, 233, 253, Color::LightCyan),
        hint: swatch(241, 250, 140, Color::LightYellow),
        aura_focus: swatch(255, 255, 185, Color::LightYellow),
        aura_carry: swatch(230, 255, 255, Color::LightCyan),
        field_edge: false,
    }
}

#[test]
fn every_builtin_toml_parses_passes_contrast_and_matches_previous_table_token_by_token() {
    let expected = [
        (ThemeId::Dark, expected_dark()),
        (ThemeId::Light, expected_light()),
        (ThemeId::HighContrast, expected_high_contrast()),
        (ThemeId::Monokai, expected_monokai()),
        (ThemeId::Dracula, expected_dracula()),
    ];
    for (id, prev) in expected {
        let parsed = parse_palette(builtin_toml(id));
        assert!(parsed.is_ok(), "{id:?} failed to parse: {parsed:?}");
        let Ok(parsed) = parsed else {
            continue;
        };
        assert!(
            validate_contrast(&parsed, id.contrast_target()).is_ok(),
            "{id:?} failed contrast"
        );
        for token in Token::ALL {
            assert_eq!(parsed.swatch(token), prev.swatch(token), "{id:?} {token:?}");
        }
        assert_eq!(parsed.field_edge, prev.field_edge, "{id:?} field_edge");
        assert_eq!(id.palette(), prev, "{id:?}");
    }
}

#[test]
fn valid_user_theme_loads_and_renders_with_its_colors() {
    let custom_toml = builtin_toml(ThemeId::Dark).replace(
        "background = { rgb = [24, 28, 38], ansi16 = \"black\" }",
        "background = { rgb = [18, 22, 34], ansi16 = \"black\" }",
    );
    let user = UserTheme::from_toml("theme.toml", &custom_toml);
    let Some(&palette) = user.palette() else {
        panic!("expected valid user theme, got {user:?}");
    };
    assert_eq!(palette.background.rgb, [18, 22, 34]);
    let theme = Theme::for_settings(ThemeId::Custom, Some(palette), ColorDepth::TrueColor);
    assert_eq!(theme.id(), ThemeId::Custom);
    assert!(theme.has_custom());
    assert_eq!(theme.color(Token::Background), Color::Rgb(18, 22, 34));
    assert_eq!(
        theme
            .card_back_style(CardBack::Theme, (Token::CardBackMark, Token::CardBack))
            .bg,
        Some(Color::Rgb(184, 208, 248))
    );
    let with_vision = theme.with_color_vision(ColorVision::Protanopia);
    assert_eq!(with_vision.color(Token::Background), Color::Rgb(18, 22, 34));
    let with_lv = theme.with_low_vision(true);
    assert_eq!(with_lv.palette().text_muted, with_lv.palette().text);
    let with_table = theme.with_table(TableColour::Blue);
    assert_eq!(with_table.color(Token::Background), Color::Rgb(18, 22, 34));
}

#[test]
fn unparsable_and_low_contrast_user_themes_are_rejected_with_specific_reasons() {
    let bad_toml = "not valid toml = [";
    assert!(matches!(
        UserTheme::from_toml("C:/data/theme.toml", bad_toml),
        UserTheme::Unreadable { ref path, .. } if path == "C:/data/theme.toml"
    ));

    let low_text = builtin_toml(ThemeId::Dark).replace(
        "text = { rgb = [232, 236, 244], ansi16 = \"white\" }",
        "text = { rgb = [40, 44, 54], ansi16 = \"gray\" }",
    );
    assert!(matches!(
        UserTheme::from_toml("C:/data/theme.toml", &low_text),
        UserTheme::LowContrast {
            fg: Token::Text,
            bg: Token::Background,
            ratio,
            required,
            ..
        } if ratio < required && (required - 4.5).abs() < f32::EPSILON
    ));

    let low_edge = builtin_toml(ThemeId::Dark).replace(
        "table = { rgb = [40, 120, 80], ansi16 = \"green\" }",
        "table = { rgb = [24, 28, 38], ansi16 = \"black\" }",
    );
    assert!(matches!(
        UserTheme::from_toml("C:/data/theme.toml", &low_edge),
        UserTheme::LowContrast {
            bg: Token::Background,
            required,
            ..
        } if (required - 3.0).abs() < f32::EPSILON
    ));
}

#[test]
fn invalid_ansi16_in_user_theme_is_rejected() {
    let bad_ansi = builtin_toml(ThemeId::Dark).replace(
        "background = { rgb = [24, 28, 38], ansi16 = \"black\" }",
        "background = { rgb = [24, 28, 38], ansi16 = \"reset\" }",
    );
    assert!(matches!(
        UserTheme::from_toml("theme.toml", &bad_ansi),
        UserTheme::Unreadable { .. }
    ));
    assert!(matches!(
        parse_palette(&bad_ansi),
        Err(ThemeError::Parse(_))
    ));
}
