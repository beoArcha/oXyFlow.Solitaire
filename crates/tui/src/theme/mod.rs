//! Semantic color themes. A [`Theme`] resolves the tokens of one palette for one color depth;
//! components use tokens only.

pub mod ansi;
pub mod contrast;
pub mod dark;
pub mod dracula;
pub mod face;
pub mod high_contrast;
pub mod light;
pub mod low_vision;
pub mod monokai;
pub mod palette;
pub mod table;
pub mod table_feel;
pub mod vision;

use oxysol_core::{Color as SuitColor, Suit};
use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

pub use palette::{GRAPHIC_PAIRS, MARK_PAIRS, Palette, Swatch, TEXT_PAIRS, Token};

use crate::caps::ColorDepth;
use crate::settings::{CardBack, CardFaceTint, ColorVision, TableColour};

/// A built-in theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeId {
    Dark,
    Light,
    HighContrast,
    Monokai,
    Dracula,
}

impl ThemeId {
    /// Every theme, in the order of the settings screen.
    pub const ALL: [ThemeId; 5] = [
        ThemeId::Dark,
        ThemeId::Light,
        ThemeId::HighContrast,
        ThemeId::Monokai,
        ThemeId::Dracula,
    ];

    /// The palette of the theme.
    pub fn palette(self) -> Palette {
        match self {
            ThemeId::Dark => dark::palette(),
            ThemeId::Light => light::palette(),
            ThemeId::HighContrast => high_contrast::palette(),
            ThemeId::Monokai => monokai::palette(),
            ThemeId::Dracula => dracula::palette(),
        }
    }

    /// The minimum WCAG contrast ratio of every pair in [`TEXT_PAIRS`].
    pub const fn contrast_target(self) -> f32 {
        match self {
            ThemeId::Dark | ThemeId::Light | ThemeId::Monokai | ThemeId::Dracula => 4.5,
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
    palette: Palette,
}

fn palette_for(id: ThemeId, vision: ColorVision, low_vision: bool) -> Palette {
    let remapped = vision::remap(id, id.palette(), vision);
    if low_vision {
        low_vision::remap(remapped)
    } else {
        remapped
    }
}

impl Theme {
    /// Resolves `id` for a terminal of the given color depth.
    pub fn new(id: ThemeId, depth: ColorDepth) -> Self {
        Self {
            id,
            depth,
            vision: ColorVision::Normal,
            low_vision: false,
            palette: id.palette(),
        }
    }

    /// This theme with its meaning tokens adapted for `vision`.
    pub fn with_color_vision(&self, vision: ColorVision) -> Self {
        Self {
            vision,
            palette: palette_for(self.id, vision, self.low_vision),
            ..*self
        }
    }

    /// This theme with low-vision contrast (7:1 for every text pair, no dim text) enabled or off.
    pub fn with_low_vision(&self, low_vision: bool) -> Self {
        Self {
            low_vision,
            palette: palette_for(self.id, self.vision, low_vision),
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
mod tests;
