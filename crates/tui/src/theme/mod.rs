//! Semantic color themes. A [`Theme`] resolves the tokens of one palette for one color depth;
//! components use tokens only.

pub mod ansi;
pub mod contrast;
pub mod dark;
pub mod high_contrast;
pub mod light;
pub mod palette;

use ratatui::style::{Color, Style};
use serde::{Deserialize, Serialize};

pub use palette::{Palette, Swatch, TEXT_PAIRS, Token};

use crate::caps::ColorDepth;

/// A built-in theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeId {
    Dark,
    Light,
    HighContrast,
}

impl ThemeId {
    /// Every theme, in the order of the settings screen.
    pub const ALL: [ThemeId; 3] = [ThemeId::Dark, ThemeId::Light, ThemeId::HighContrast];

    /// The palette of the theme.
    pub fn palette(self) -> Palette {
        match self {
            ThemeId::Dark => dark::palette(),
            ThemeId::Light => light::palette(),
            ThemeId::HighContrast => high_contrast::palette(),
        }
    }

    /// The minimum WCAG contrast ratio of every pair in [`TEXT_PAIRS`].
    pub const fn contrast_target(self) -> f32 {
        match self {
            ThemeId::Dark | ThemeId::Light => 4.5,
            ThemeId::HighContrast => 7.0,
        }
    }
}

/// The tokens of one theme, resolved for one color depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    id: ThemeId,
    depth: ColorDepth,
    palette: Palette,
}

impl Theme {
    /// Resolves `id` for a terminal of the given color depth.
    pub fn new(id: ThemeId, depth: ColorDepth) -> Self {
        Self {
            id,
            depth,
            palette: id.palette(),
        }
    }

    /// Which theme this is.
    pub fn id(&self) -> ThemeId {
        self.id
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
}

#[cfg(test)]
mod tests;
