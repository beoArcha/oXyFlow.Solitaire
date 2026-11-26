//! Semantic color themes. A [`Theme`] resolves the tokens of one palette for one color depth;
//! components use tokens only.

pub mod ansi;
pub mod contrast;
pub mod face;
pub mod low_vision;
pub mod manifest;
pub mod palette;
pub mod table;
pub mod table_feel;
pub mod vision;

use oxysol_core::{Color as SuitColor, Suit};
use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

pub use manifest::{ThemeError, UserTheme, parse_palette, validate_contrast};
pub use palette::{GRAPHIC_PAIRS, MARK_PAIRS, Palette, Swatch, TEXT_PAIRS, Token};

use crate::caps::ColorDepth;
use crate::settings::{CardBack, CardFaceTint, ColorVision, TableColour};

/// A built-in theme, or the player's custom theme from `theme.toml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeId {
    Dark,
    Light,
    HighContrast,
    Monokai,
    Dracula,
    Custom,
}

impl ThemeId {
    /// Every built-in theme, in the order of the settings screen.
    pub const ALL: [ThemeId; 5] = [
        ThemeId::Dark,
        ThemeId::Light,
        ThemeId::HighContrast,
        ThemeId::Monokai,
        ThemeId::Dracula,
    ];

    /// Every theme including `Custom`, in the order of the settings screen.
    pub const ALL_WITH_CUSTOM: [ThemeId; 6] = [
        ThemeId::Dark,
        ThemeId::Light,
        ThemeId::HighContrast,
        ThemeId::Monokai,
        ThemeId::Dracula,
        ThemeId::Custom,
    ];

    /// The palette of the built-in theme (`Custom` falls back to `Dark`).
    pub fn palette(self) -> Palette {
        manifest::builtin_palette(self)
    }

    /// The minimum WCAG contrast ratio of every pair in [`TEXT_PAIRS`].
    pub const fn contrast_target(self) -> f32 {
        match self {
            ThemeId::Dark
            | ThemeId::Light
            | ThemeId::Monokai
            | ThemeId::Dracula
            | ThemeId::Custom => 4.5,
            ThemeId::HighContrast => 7.0,
        }
    }
}

/// The tokens of one theme, resolved for one color depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    id: ThemeId,
    depth: ColorDepth,
    vision: ColorVision,
    low_vision: bool,
    base: Palette,
    palette: Palette,
    custom: Option<Palette>,
}

/// The colour-vision palette of `id`. The remap of a custom theme is made for dark themes, so a
/// custom theme keeps its own colours when the remap would break its contrast.
fn vision_palette(id: ThemeId, base: Palette, vision: ColorVision) -> Palette {
    let remapped = vision::remap(id, base, vision);
    if id == ThemeId::Custom && validate_contrast(&remapped, id.contrast_target()).is_err() {
        return base;
    }
    remapped
}

fn palette_for(id: ThemeId, base: Palette, vision: ColorVision, low_vision: bool) -> Palette {
    let remapped = vision_palette(id, base, vision);
    if low_vision {
        low_vision::remap(remapped)
    } else {
        remapped
    }
}

impl Theme {
    /// Resolves built-in `id` for a terminal of the given color depth.
    pub fn new(id: ThemeId, depth: ColorDepth) -> Self {
        let base = id.palette();
        Self {
            id,
            depth,
            vision: ColorVision::Normal,
            low_vision: false,
            base,
            palette: base,
            custom: None,
        }
    }

    /// Resolves `id` with an optional user-defined `custom` palette. Falls back to `Dark` when
    /// `id` is `Custom` without a loaded palette.
    pub fn for_settings(id: ThemeId, custom: Option<Palette>, depth: ColorDepth) -> Self {
        let (effective_id, base) = match (id, custom) {
            (ThemeId::Custom, Some(palette)) => (ThemeId::Custom, palette),
            (ThemeId::Custom, None) => (ThemeId::Dark, ThemeId::Dark.palette()),
            (other, _) => (other, other.palette()),
        };
        Self {
            id: effective_id,
            depth,
            vision: ColorVision::Normal,
            low_vision: false,
            base,
            palette: base,
            custom,
        }
    }

    /// Whether a valid user theme is loaded and offered as `Custom`.
    pub const fn has_custom(&self) -> bool {
        self.custom.is_some()
    }

    /// The loaded user palette, if any.
    pub const fn custom_palette(&self) -> Option<&Palette> {
        self.custom.as_ref()
    }

    /// The un-remapped base palette of the active theme.
    pub const fn base_palette(&self) -> &Palette {
        &self.base
    }

    /// This theme with its meaning tokens adapted for `vision`.
    pub fn with_color_vision(&self, vision: ColorVision) -> Self {
        Self {
            vision,
            palette: palette_for(self.id, self.base, vision, self.low_vision),
            ..*self
        }
    }

    /// This theme with low-vision contrast (7:1 for every text pair, no dim text) enabled or off.
    pub fn with_low_vision(&self, low_vision: bool) -> Self {
        Self {
            low_vision,
            palette: palette_for(self.id, self.base, self.vision, low_vision),
            ..*self
        }
    }

    /// The resolved palette.
    pub fn palette(&self) -> &Palette {
        &self.palette
    }

    /// Which theme this is.
    pub fn id(&self) -> ThemeId {
        self.id
    }

    /// Whether the board draws a one-cell frame around the playing field in this theme.
    pub fn field_edge(&self) -> bool {
        self.palette.field_edge
    }

    /// The color to emit for a token.
    pub fn color(&self, token: Token) -> Color {
        self.palette.swatch(token).resolve(self.depth)
    }

    /// A style with the given foreground and background tokens.
    pub fn style(&self, fg: Token, bg: Token) -> Style {
        Style::new().fg(self.color(fg)).bg(self.color(bg))
    }

    /// Default text on the screen background.
    pub fn base(&self) -> Style {
        self.style(Token::Text, Token::Background)
    }

    /// Default text on a dialog surface.
    pub fn surface(&self) -> Style {
        self.style(Token::Text, Token::Surface)
    }

    /// The focused list item.
    pub fn focus(&self) -> Style {
        self.style(Token::FocusFg, Token::FocusBg)
    }

    /// The active tab in an overlay window: bold, underlined accent text on the surface.
    pub fn active_tab(&self) -> Style {
        self.style(Token::Accent, Token::Surface)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    }

    /// The focused action button in an overlay dialog: focus colors in bold reverse video.
    pub fn focused_button(&self) -> Style {
        self.style(Token::FocusBg, Token::FocusFg)
            .add_modifier(Modifier::BOLD | Modifier::REVERSED)
    }

    /// This theme with its felt replaced by the felt of `colour` (or the shade of it that passes the
    /// contrast tests here). Every token of the table, the texture and the cards on it follow.
    pub fn with_table(&self, colour: TableColour) -> Self {
        let target = if self.low_vision {
            7.0
        } else {
            self.id.contrast_target()
        };
        let felt = table::felt_with_target(target, &self.palette, colour);
        Self {
            palette: Palette {
                table: felt.swatch,
                ..self.palette
            },
            ..*self
        }
    }

    /// The fill and suit style of a face-up card for `tint`. `suit` is the token of its suit color,
    /// `CardRed` or `CardBlack`; `Theme` gives the same style as the theme's own face tokens.
    pub fn card_face_style(&self, tint: CardFaceTint, suit: Token) -> Style {
        let colors = face::face_colors(tint, &self.palette);
        let ink = if suit == Token::CardBlack {
            colors.black
        } else {
            colors.red
        };
        Style::new()
            .fg(ink.resolve(self.depth))
            .bg(colors.face.resolve(self.depth))
    }

    /// The fill and suit style of a face-up card for `tint` and `suit`, supporting the four-color
    /// deck (spades black, hearts red, diamonds blue, clubs green) adapted to `self.vision`.
    pub fn card_suit_style(&self, tint: CardFaceTint, suit: Suit, four_color: bool) -> Style {
        if !four_color {
            let token = match suit.color() {
                SuitColor::Red => Token::CardRed,
                SuitColor::Black => Token::CardBlack,
            };
            return self.card_face_style(tint, token);
        }
        let colors = face::suit_colors(tint, &self.palette, self.vision);
        Style::new()
            .fg(colors.for_suit(suit).resolve(self.depth))
            .bg(colors.face.resolve(self.depth))
    }

    /// A style for the given card back palette, with the pattern and background tokens of the back
    /// style: `Theme` resolves them in this theme, the others in the theme of that name.
    pub fn card_back_style(
        &self,
        card_back: CardBack,
        (mark, background): (Token, Token),
    ) -> Style {
        match card_back {
            CardBack::Theme => self.style(mark, background),
            CardBack::Monokai => Theme::new(ThemeId::Monokai, self.depth)
                .with_low_vision(self.low_vision)
                .style(mark, background),
            CardBack::Dracula => Theme::new(ThemeId::Dracula, self.depth)
                .with_low_vision(self.low_vision)
                .style(mark, background),
        }
    }
}

#[cfg(test)]
mod manifest_tests;
#[cfg(test)]
mod tests;
