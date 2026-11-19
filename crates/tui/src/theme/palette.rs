//! The token vocabulary of a theme and the data a palette must provide for each token.

use ratatui::style::Color;

use super::ansi::rgb_to_ansi256;
use crate::caps::ColorDepth;

/// A semantic color slot. Components ask for tokens, never for concrete colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Token {
    /// Screen background.
    Background,
    /// Background of dialogs and panels.
    Surface,
    /// Primary text on `Background` and `Surface`.
    Text,
    /// Secondary text on `Background` and `Surface`.
    TextMuted,
    /// Titles and highlights on `Background` and `Surface`.
    Accent,
    /// Border of an unfocused element.
    Border,
    /// Border of the focused element.
    BorderFocus,
    /// Text of the focused list item, on `FocusBg`.
    FocusFg,
    /// Background of the focused list item.
    FocusBg,
    /// Error text on `Surface`.
    Error,
    /// Success text on `Surface`.
    Success,
    /// Playing-field background.
    Table,
    /// Text on `Table`.
    TableText,
    /// Outline of an empty pile slot, on `Table`.
    Slot,
    /// Face of a card.
    CardFace,
    /// Face-down card background.
    CardBack,
    /// Pattern drawn on `CardBack`.
    CardBackMark,
    /// Rank and suit of red cards, on `CardFace`.
    CardRed,
    /// Rank and suit of black cards, on `CardFace`.
    CardBlack,
    /// Highlight of a legal move target, on `Table`.
    Target,
    /// Highlight of the selected card, on `Table`.
    Selection,
    /// Highlight of a hinted card, on `Table`.
    Hint,
    /// Halo around the focused card, on `Table`.
    AuraFocus,
    /// Halo around the picked-up run, on `Table`.
    AuraCarry,
}

impl Token {
    /// Every token.
    pub const ALL: [Token; 24] = [
        Token::Background,
        Token::Surface,
        Token::Text,
        Token::TextMuted,
        Token::Accent,
        Token::Border,
        Token::BorderFocus,
        Token::FocusFg,
        Token::FocusBg,
        Token::Error,
        Token::Success,
        Token::Table,
        Token::TableText,
        Token::Slot,
        Token::CardFace,
        Token::CardBack,
        Token::CardBackMark,
        Token::CardRed,
        Token::CardBlack,
        Token::Target,
        Token::Selection,
        Token::Hint,
        Token::AuraFocus,
        Token::AuraCarry,
    ];
}

/// One token's color for every color depth: exact RGB for true color (and, converted, for 256
/// colors) and a named ANSI color for 16-color terminals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Swatch {
    pub rgb: [u8; 3],
    pub ansi16: Color,
}

/// Shorthand constructor used by the palette files.
pub const fn swatch(r: u8, g: u8, b: u8, ansi16: Color) -> Swatch {
    Swatch {
        rgb: [r, g, b],
        ansi16,
    }
}

impl Swatch {
    /// The color to emit for a terminal of the given depth.
    pub fn resolve(&self, depth: ColorDepth) -> Color {
        let [r, g, b] = self.rgb;
        match depth {
            ColorDepth::TrueColor => Color::Rgb(r, g, b),
            ColorDepth::Ansi256 => Color::Indexed(rgb_to_ansi256(r, g, b)),
            ColorDepth::Ansi16 => self.ansi16,
        }
    }
}

/// Text on background pairs that are drawn together; each must reach the theme's contrast target.
pub const TEXT_PAIRS: [(Token, Token); 17] = [
    (Token::Text, Token::Background),
    (Token::Text, Token::Surface),
    (Token::TextMuted, Token::Background),
    (Token::TextMuted, Token::Surface),
    (Token::Accent, Token::Background),
    (Token::Accent, Token::Surface),
    (Token::FocusFg, Token::FocusBg),
    (Token::Error, Token::Background),
    (Token::Error, Token::Surface),
    (Token::Success, Token::Background),
    (Token::Success, Token::Surface),
    (Token::TableText, Token::Table),
    (Token::CardRed, Token::CardFace),
    (Token::CardBlack, Token::CardFace),
    (Token::CardBackMark, Token::CardBack),
    (Token::AuraFocus, Token::Table),
    (Token::AuraCarry, Token::Table),
];

/// Graphics that carry meaning on the felt; each must reach 3:1 (WCAG 1.4.11). The edge between the
/// felt and the screen is [`Palette::field_edge_pair`], which depends on the theme.
pub const GRAPHIC_PAIRS: [(Token, Token); 6] = [
    (Token::BorderFocus, Token::Table),
    (Token::CardBack, Token::Table),
    (Token::Slot, Token::Table),
    (Token::Target, Token::Table),
    (Token::Selection, Token::Table),
    (Token::Hint, Token::Table),
];

/// Marks drawn on the felt and on dialog surfaces; each must reach 3:1 in every theme, including the
/// marks that a 16-color terminal draws in its named colors.
pub const MARK_PAIRS: [(Token, Token); 6] = [
    (Token::Target, Token::Table),
    (Token::Selection, Token::Table),
    (Token::Hint, Token::Table),
    (Token::BorderFocus, Token::Table),
    (Token::Slot, Token::Table),
    (Token::Border, Token::Surface),
];

/// Every token of one theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub background: Swatch,
    pub surface: Swatch,
    pub text: Swatch,
    pub text_muted: Swatch,
    pub accent: Swatch,
    pub border: Swatch,
    pub border_focus: Swatch,
    pub focus_fg: Swatch,
    pub focus_bg: Swatch,
    pub error: Swatch,
    pub success: Swatch,
    pub table: Swatch,
    pub table_text: Swatch,
    pub slot: Swatch,
    pub card_face: Swatch,
    pub card_back: Swatch,
    pub card_back_mark: Swatch,
    pub card_red: Swatch,
    pub card_black: Swatch,
    pub target: Swatch,
    pub selection: Swatch,
    pub hint: Swatch,
    pub aura_focus: Swatch,
    pub aura_carry: Swatch,
    /// Whether the board draws a one-cell `Border` frame around the felt. A theme sets it when its
    /// felt has no edge of its own against `Background`.
    pub field_edge: bool,
}

impl Palette {
    /// The pair whose contrast makes the edge of the playing field: `Border` on `Background` when the
    /// frame is drawn, otherwise the felt itself, `Table` on `Background`. Either must reach 3:1.
    pub fn field_edge_pair(&self) -> (Token, Token) {
        if self.field_edge {
            (Token::Border, Token::Background)
        } else {
            (Token::Table, Token::Background)
        }
    }

    /// The swatch of a token.
    pub fn swatch(&self, token: Token) -> Swatch {
        match token {
            Token::Background => self.background,
            Token::Surface => self.surface,
            Token::Text => self.text,
            Token::TextMuted => self.text_muted,
            Token::Accent => self.accent,
            Token::Border => self.border,
            Token::BorderFocus => self.border_focus,
            Token::FocusFg => self.focus_fg,
            Token::FocusBg => self.focus_bg,
            Token::Error => self.error,
            Token::Success => self.success,
            Token::Table => self.table,
            Token::TableText => self.table_text,
            Token::Slot => self.slot,
            Token::CardFace => self.card_face,
            Token::CardBack => self.card_back,
            Token::CardBackMark => self.card_back_mark,
            Token::CardRed => self.card_red,
            Token::CardBlack => self.card_black,
            Token::Target => self.target,
            Token::Selection => self.selection,
            Token::Hint => self.hint,
            Token::AuraFocus => self.aura_focus,
            Token::AuraCarry => self.aura_carry,
        }
    }
}
