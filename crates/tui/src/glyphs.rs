//! Symbols for markers and card suits in the Unicode and ASCII sets. `Font` mode (the oXySol
//! card font) uses the Unicode set until the font exists.

use oxysol_core::Suit;
use ratatui::widgets::BorderType;

use crate::settings::GlyphMode;

/// The symbols one glyph mode draws with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glyphs {
    pub unicode: bool,
    pub suits: [&'static str; 4],
    pub focus_left: &'static str,
    pub focus_right: &'static str,
    pub selected: &'static str,
    pub face_down: &'static str,
    pub border: BorderType,
}

const UNICODE: Glyphs = Glyphs {
    unicode: true,
    suits: ["♣", "♦", "♥", "♠"],
    focus_left: "▶",
    focus_right: "◀",
    selected: "●",
    face_down: "▒",
    border: BorderType::Rounded,
};

const ASCII: Glyphs = Glyphs {
    unicode: false,
    suits: ["C", "D", "H", "S"],
    focus_left: ">",
    focus_right: "<",
    selected: "*",
    face_down: "#",
    border: BorderType::Plain,
};

impl Glyphs {
    /// The symbol set of a glyph mode.
    pub fn new(mode: GlyphMode) -> Self {
        match mode {
            GlyphMode::Font | GlyphMode::Unicode => UNICODE,
            GlyphMode::Ascii => ASCII,
        }
    }

    /// The symbol of a suit.
    pub fn suit(&self, suit: Suit) -> &'static str {
        match suit {
            Suit::Clubs => self.suits[0],
            Suit::Diamonds => self.suits[1],
            Suit::Hearts => self.suits[2],
            Suit::Spades => self.suits[3],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_set_is_pure_ascii() {
        let glyphs = Glyphs::new(GlyphMode::Ascii);
        let all = glyphs.suits.iter().chain([
            &glyphs.focus_left,
            &glyphs.focus_right,
            &glyphs.selected,
            &glyphs.face_down,
        ]);
        for text in all {
            assert!(text.is_ascii(), "{text}");
        }
    }

    #[test]
    fn suits_are_distinct_in_both_sets() {
        for mode in [GlyphMode::Unicode, GlyphMode::Ascii] {
            let glyphs = Glyphs::new(mode);
            let mut seen: Vec<_> = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades]
                .into_iter()
                .map(|suit| glyphs.suit(suit))
                .collect();
            seen.sort_unstable();
            seen.dedup();
            assert_eq!(seen.len(), 4);
        }
    }

    #[test]
    fn font_mode_falls_back_to_unicode() {
        assert_eq!(
            Glyphs::new(GlyphMode::Font),
            Glyphs::new(GlyphMode::Unicode)
        );
    }
}
