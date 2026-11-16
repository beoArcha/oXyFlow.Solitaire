//! Symbols for markers and card suits in the Unicode and ASCII sets. `Font` mode (the oXySol
//! card font) uses the Unicode set until the font exists. The Unicode set uses only WGL4 symbols,
//! which every common terminal font draws.

use oxysol_core::Suit;
use ratatui::widgets::BorderType;

pub mod wgl4;
pub use wgl4::is_wgl4;

use crate::settings::GlyphMode;

/// The symbols one glyph mode draws with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glyphs {
    pub unicode: bool,
    pub suits: [&'static str; 4],
    pub focus_left: &'static str,
    pub focus_right: &'static str,
    pub selected: &'static str,
    /// Drawn in both top corners of a drop target and in the middle of an empty target slot.
    pub target: &'static str,
    /// Top-row marker of the card a hint moves.
    pub hint_source: &'static str,
    /// Top-row marker of the pile a hint moves to.
    pub hint_target: &'static str,
    /// Placeholder of an empty stock that can be turned over again.
    pub recycle: &'static str,
    /// Placeholder of an empty stock with no pass left.
    pub spent: &'static str,
    pub face_down: &'static str,
    /// Halo around the focused card.
    pub aura_focus: &'static str,
    /// Halo around the picked-up run.
    pub aura_carry: &'static str,
    /// Weaker halo around a valid drop target.
    pub aura_target: &'static str,
    /// Vertical line that separates a side panel from the board.
    pub edge: &'static str,
    pub border: BorderType,
}

const UNICODE: Glyphs = Glyphs {
    unicode: true,
    suits: ["♣", "♦", "♥", "♠"],
    focus_left: "►",
    focus_right: "◄",
    selected: "●",
    target: "+",
    hint_source: "?→",
    hint_target: "→?",
    recycle: "↑",
    spent: "x",
    face_down: "▒",
    aura_focus: "░",
    aura_carry: "▓",
    aura_target: "·",
    edge: "│",
    border: BorderType::Plain,
};

const ASCII: Glyphs = Glyphs {
    unicode: false,
    suits: ["c", "d", "h", "s"],
    focus_left: ">",
    focus_right: "<",
    selected: "*",
    target: "+",
    hint_source: "?>",
    hint_target: ">?",
    recycle: "^",
    spent: "x",
    face_down: "#",
    aura_focus: ":",
    aura_carry: "=",
    aura_target: ".",
    edge: "|",
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
            &glyphs.target,
            &glyphs.hint_source,
            &glyphs.hint_target,
            &glyphs.recycle,
            &glyphs.spent,
            &glyphs.face_down,
            &glyphs.aura_focus,
            &glyphs.aura_carry,
            &glyphs.aura_target,
            &glyphs.edge,
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
    fn the_three_auras_differ_in_both_sets() {
        for mode in [GlyphMode::Unicode, GlyphMode::Ascii] {
            let glyphs = Glyphs::new(mode);
            let auras = [glyphs.aura_focus, glyphs.aura_carry, glyphs.aura_target];
            for (index, aura) in auras.iter().enumerate() {
                assert!(!aura.is_empty());
                assert!(auras[index + 1..].iter().all(|other| other != aura));
                assert_ne!(*aura, glyphs.face_down);
                assert_ne!(*aura, glyphs.selected);
            }
        }
    }

    #[test]
    fn ascii_suits_are_lowercase_so_no_rank_letter_is_a_suit_letter() {
        let glyphs = Glyphs::new(GlyphMode::Ascii);
        assert_eq!(glyphs.suits, ["c", "d", "h", "s"]);
    }

    #[test]
    fn the_card_marks_differ_in_both_sets() {
        for mode in [GlyphMode::Unicode, GlyphMode::Ascii] {
            let glyphs = Glyphs::new(mode);
            let marks = [
                glyphs.selected,
                glyphs.focus_left,
                glyphs.target,
                glyphs.hint_source,
                glyphs.hint_target,
            ];
            for (index, mark) in marks.iter().enumerate() {
                assert!(!mark.is_empty());
                assert!(marks[index + 1..].iter().all(|other| other != mark));
            }
            assert_eq!(glyphs.hint_source.chars().count(), 2);
            assert_eq!(glyphs.hint_target.chars().count(), 2);
        }
    }

    #[test]
    fn font_mode_falls_back_to_unicode() {
        assert_eq!(
            Glyphs::new(GlyphMode::Font),
            Glyphs::new(GlyphMode::Unicode)
        );
    }

    #[test]
    fn unicode_set_has_no_rounded_heavy_or_dashed_lines() {
        let glyphs = Glyphs::new(GlyphMode::Unicode);
        assert_eq!(glyphs.border, BorderType::Plain);
        assert_ne!(glyphs.focus_left, "▶");
        assert_ne!(glyphs.focus_right, "◀");
    }
}
