//! Parsing and contrast validation of theme TOML files (embedded built-ins and user `theme.toml`).

use std::sync::OnceLock;

use ratatui::style::Color;
use serde::Deserialize;
use thiserror::Error;

use super::ThemeId;
use super::contrast::contrast_ratio;
use super::palette::{GRAPHIC_PAIRS, MARK_PAIRS, Palette, Swatch, TEXT_PAIRS, Token, swatch};

const DARK_TEXT_TARGET: f32 = 4.5;
const GRAPHIC_TARGET: f32 = 3.0;

const DARK_TOML: &str = include_str!("../../../../assets/themes/dark.toml");
const LIGHT_TOML: &str = include_str!("../../../../assets/themes/light.toml");
const HIGH_CONTRAST_TOML: &str = include_str!("../../../../assets/themes/high_contrast.toml");
const MONOKAI_TOML: &str = include_str!("../../../../assets/themes/monokai.toml");
const DRACULA_TOML: &str = include_str!("../../../../assets/themes/dracula.toml");

const FALLBACK_PALETTE: Palette = Palette {
    background: swatch(0, 0, 0, Color::Black),
    surface: swatch(0, 0, 0, Color::Black),
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
};

/// Why a theme TOML file failed to parse or failed contrast validation.
#[derive(Debug, Error)]
pub enum ThemeError {
    #[error("invalid theme TOML")]
    Parse(#[from] toml::de::Error),
    #[error("{fg:?} on {bg:?} has contrast {ratio:.2}, needs at least {required:.1}")]
    Contrast {
        fg: Token,
        bg: Token,
        ratio: f32,
        required: f32,
    },
}

/// Result of loading the optional user theme (`theme.toml` in the data directory) at startup.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum UserTheme {
    /// No `theme.toml` file was present.
    #[default]
    Missing,
    /// A valid user theme palette ready to be offered as `Custom`.
    Valid(Palette),
    /// The file existed at `path` but could not be read or parsed.
    Unreadable { path: String, detail: String },
    /// The file parsed at `path` but failed a contrast check.
    LowContrast {
        path: String,
        fg: Token,
        bg: Token,
        ratio: f32,
        required: f32,
    },
}

impl UserTheme {
    /// Parses `text` (loaded from `path`) and validates it against the Dark theme contrast rules.
    pub fn from_toml(path: impl Into<String>, text: &str) -> Self {
        let path = path.into();
        let palette = match parse_palette(text) {
            Ok(palette) => palette,
            Err(error) => {
                return Self::Unreadable {
                    path,
                    detail: error.to_string(),
                };
            }
        };
        match validate_contrast(&palette, DARK_TEXT_TARGET) {
            Ok(()) => Self::Valid(palette),
            Err(ThemeError::Contrast {
                fg,
                bg,
                ratio,
                required,
            }) => Self::LowContrast {
                path,
                fg,
                bg,
                ratio,
                required,
            },
            Err(ThemeError::Parse(error)) => Self::Unreadable {
                path,
                detail: error.to_string(),
            },
        }
    }

    /// The valid user palette, if one was loaded.
    pub const fn palette(&self) -> Option<&Palette> {
        match self {
            Self::Valid(palette) => Some(palette),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Ansi16 {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
}

impl From<Ansi16> for Color {
    fn from(value: Ansi16) -> Self {
        match value {
            Ansi16::Black => Color::Black,
            Ansi16::Red => Color::Red,
            Ansi16::Green => Color::Green,
            Ansi16::Yellow => Color::Yellow,
            Ansi16::Blue => Color::Blue,
            Ansi16::Magenta => Color::Magenta,
            Ansi16::Cyan => Color::Cyan,
            Ansi16::Gray => Color::Gray,
            Ansi16::DarkGray => Color::DarkGray,
            Ansi16::LightRed => Color::LightRed,
            Ansi16::LightGreen => Color::LightGreen,
            Ansi16::LightYellow => Color::LightYellow,
            Ansi16::LightBlue => Color::LightBlue,
            Ansi16::LightMagenta => Color::LightMagenta,
            Ansi16::LightCyan => Color::LightCyan,
            Ansi16::White => Color::White,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSwatch {
    rgb: [u8; 3],
    ansi16: Ansi16,
}

impl From<RawSwatch> for Swatch {
    fn from(raw: RawSwatch) -> Self {
        Self {
            rgb: raw.rgb,
            ansi16: raw.ansi16.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPalette {
    #[serde(default)]
    field_edge: bool,
    background: RawSwatch,
    surface: RawSwatch,
    text: RawSwatch,
    text_muted: RawSwatch,
    accent: RawSwatch,
    border: RawSwatch,
    border_focus: RawSwatch,
    focus_fg: RawSwatch,
    focus_bg: RawSwatch,
    error: RawSwatch,
    success: RawSwatch,
    table: RawSwatch,
    table_text: RawSwatch,
    slot: RawSwatch,
    card_face: RawSwatch,
    card_back: RawSwatch,
    card_back_mark: RawSwatch,
    card_red: RawSwatch,
    card_black: RawSwatch,
    target: RawSwatch,
    selection: RawSwatch,
    hint: RawSwatch,
    aura_focus: RawSwatch,
    aura_carry: RawSwatch,
}

impl From<RawPalette> for Palette {
    fn from(raw: RawPalette) -> Self {
        Self {
            background: raw.background.into(),
            surface: raw.surface.into(),
            text: raw.text.into(),
            text_muted: raw.text_muted.into(),
            accent: raw.accent.into(),
            border: raw.border.into(),
            border_focus: raw.border_focus.into(),
            focus_fg: raw.focus_fg.into(),
            focus_bg: raw.focus_bg.into(),
            error: raw.error.into(),
            success: raw.success.into(),
            table: raw.table.into(),
            table_text: raw.table_text.into(),
            slot: raw.slot.into(),
            card_face: raw.card_face.into(),
            card_back: raw.card_back.into(),
            card_back_mark: raw.card_back_mark.into(),
            card_red: raw.card_red.into(),
            card_black: raw.card_black.into(),
            target: raw.target.into(),
            selection: raw.selection.into(),
            hint: raw.hint.into(),
            aura_focus: raw.aura_focus.into(),
            aura_carry: raw.aura_carry.into(),
            field_edge: raw.field_edge,
        }
    }
}

/// Parses a theme TOML document into a [`Palette`].
pub fn parse_palette(text: &str) -> Result<Palette, ThemeError> {
    let raw: RawPalette = toml::from_str(text)?;
    Ok(raw.into())
}

fn check_pairs(
    palette: &Palette,
    pairs: &[(Token, Token)],
    required: f32,
) -> Result<(), ThemeError> {
    for &(fg, bg) in pairs {
        let ratio = contrast_ratio(palette.swatch(fg).rgb, palette.swatch(bg).rgb);
        if ratio < required {
            return Err(ThemeError::Contrast {
                fg,
                bg,
                ratio,
                required,
            });
        }
    }
    Ok(())
}

/// Checks every text pair against `text_target` and every graphic, mark and field-edge pair
/// against 3.0.
pub fn validate_contrast(palette: &Palette, text_target: f32) -> Result<(), ThemeError> {
    check_pairs(palette, &TEXT_PAIRS, text_target)?;
    check_pairs(palette, &GRAPHIC_PAIRS, GRAPHIC_TARGET)?;
    check_pairs(palette, &MARK_PAIRS, GRAPHIC_TARGET)?;
    check_pairs(palette, &[palette.field_edge_pair()], GRAPHIC_TARGET)
}

pub(super) fn builtin_toml(id: ThemeId) -> &'static str {
    match id {
        ThemeId::Dark | ThemeId::Custom => DARK_TOML,
        ThemeId::Light => LIGHT_TOML,
        ThemeId::HighContrast => HIGH_CONTRAST_TOML,
        ThemeId::Monokai => MONOKAI_TOML,
        ThemeId::Dracula => DRACULA_TOML,
    }
}

fn load_builtin(id: ThemeId) -> Palette {
    match parse_palette(builtin_toml(id)) {
        Ok(palette) => palette,
        Err(error) => {
            tracing::error!(?id, %error, "embedded built-in theme failed to parse");
            FALLBACK_PALETTE
        }
    }
}

pub(super) fn builtin_palette(id: ThemeId) -> Palette {
    static DARK: OnceLock<Palette> = OnceLock::new();
    static LIGHT: OnceLock<Palette> = OnceLock::new();
    static HIGH_CONTRAST: OnceLock<Palette> = OnceLock::new();
    static MONOKAI: OnceLock<Palette> = OnceLock::new();
    static DRACULA: OnceLock<Palette> = OnceLock::new();

    match id {
        ThemeId::Dark | ThemeId::Custom => *DARK.get_or_init(|| load_builtin(ThemeId::Dark)),
        ThemeId::Light => *LIGHT.get_or_init(|| load_builtin(ThemeId::Light)),
        ThemeId::HighContrast => *HIGH_CONTRAST.get_or_init(|| load_builtin(ThemeId::HighContrast)),
        ThemeId::Monokai => *MONOKAI.get_or_init(|| load_builtin(ThemeId::Monokai)),
        ThemeId::Dracula => *DRACULA.get_or_init(|| load_builtin(ThemeId::Dracula)),
    }
}
