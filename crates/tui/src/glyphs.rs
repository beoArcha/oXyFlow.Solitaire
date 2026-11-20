//! Symbols for markers and card suits in the Unicode and ASCII sets. `Glyphs::new` is the plain set
//! of a mode (`Font` draws the Unicode set); `Glyphs::styled` adds the card styles of the settings:
//! the suit symbols and rank labels of a front, the tile of a back and, in `Font` mode, the Private
//! Use Area glyphs of the oXySol card font. The Unicode set uses only WGL4 symbols, which every
//! common terminal font draws.

use oxysol_core::{Color as SuitColor, Rank, Suit};
use ratatui::widgets::BorderType;

pub mod wgl4;
pub use wgl4::is_wgl4;

use crate::card_style::{BackStyle, CardStyles, FrontStyle};
use crate::settings::GlyphMode;
use crate::solver::Proof;
use crate::theme::Token;

/// The symbols one glyph mode draws with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glyphs {
    pub unicode: bool,
    /// Whether the card font is in use: `Font` mode with a style. Never set without a style.
    pub font: bool,
    /// The front style the suit symbols come from, if any.
    pub front: Option<&'static FrontStyle>,
    /// The back style the tile comes from, if any.
    pub back: Option<&'static BackStyle>,
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
    /// The groove of a scroll bar.
    pub scroll_track: &'static str,
    /// The thumb of a scroll bar, which marks the visible part of the text.
    pub scroll_thumb: &'static str,
    /// Sign of an entry of the possible moves panel that can be run now.
    pub entry_on: &'static str,
    /// Sign of an entry of the possible moves panel that cannot be run now.
    pub entry_off: &'static str,
    /// Sign of a game proved winnable by the solver.
    pub proof_winnable: &'static str,
    /// Sign of a game proved unwinnable by the solver.
    pub proof_unwinnable: &'static str,
    /// Sign of a game where the solver reached its limit without a proof.
    pub proof_unknown: &'static str,
    /// Sign of a game not checked by the solver.
    pub proof_unchecked: &'static str,
    /// Mark of a popular game in the game menu.
    pub star: &'static str,
    /// Mark of a game in the favourites.
    pub favourite: &'static str,
    /// Mark of a game that is not in the favourites.
    pub not_favourite: &'static str,
    pub border: BorderType,
}

const UNICODE: Glyphs = Glyphs {
    unicode: true,
    font: false,
    front: None,
    back: None,
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
    scroll_track: "░",
    scroll_thumb: "█",
    entry_on: "•",
    entry_off: "×",
    proof_winnable: "✓",
    proof_unwinnable: "✗",
    proof_unknown: "?",
    proof_unchecked: "·",
    star: "★",
    favourite: "♥",
    not_favourite: "·",
    border: BorderType::Plain,
};

const ASCII: Glyphs = Glyphs {
    unicode: false,
    font: false,
    front: None,
    back: None,
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
    scroll_track: ":",
    scroll_thumb: "#",
    entry_on: "o",
    entry_off: "x",
    proof_winnable: "+",
    proof_unwinnable: "x",
    proof_unknown: "?",
    proof_unchecked: ".",
    star: "*",
    favourite: "!",
    not_favourite: ".",
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

    /// The set of a mode with the symbols of the front and the back style with these ids. An id that
    /// is not a style falls back to the default style.
    pub fn styled(mode: GlyphMode, front: &str, back: &str) -> Self {
        let styles = CardStyles::embedded();
        let mut glyphs = Self::new(mode);
        if let Some(front) = styles.front_or_default(front) {
            glyphs.suits = front.suit_symbols(mode);
            glyphs.front = Some(front);
        }
        if let Some(back) = styles.back_or_default(back) {
            glyphs.face_down = back.tile(mode);
            glyphs.back = Some(back);
        }
        glyphs.font = mode == GlyphMode::Font && (glyphs.front.is_some() || glyphs.back.is_some());
        glyphs
    }

    /// The mode the set draws: `Font` only when the card font is in use.
    pub fn mode(&self) -> GlyphMode {
        if self.font {
            GlyphMode::Font
        } else if self.unicode {
            GlyphMode::Unicode
        } else {
            GlyphMode::Ascii
        }
    }

    /// The label the front style gives to a rank; `None` keeps the localized label.
    pub fn rank_label(&self, rank: Rank) -> Option<&'static str> {
        self.front.and_then(|front| front.rank_label(rank))
    }

    /// The token of the color a suit is drawn in.
    pub fn suit_token(&self, suit: Suit) -> Token {
        self.front.map_or_else(
            || match suit.color() {
                SuitColor::Red => Token::CardRed,
                SuitColor::Black => Token::CardBlack,
            },
            |front| front.suit(suit).token,
        )
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

    /// The sign that marks a solver proof.
    pub fn proof_mark(&self, proof: Option<Proof>) -> &'static str {
        match proof {
            Some(Proof::Winnable) => self.proof_winnable,
            Some(Proof::Unwinnable) => self.proof_unwinnable,
            Some(Proof::Unknown) => self.proof_unknown,
            None => self.proof_unchecked,
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
            &glyphs.scroll_track,
            &glyphs.scroll_thumb,
            &glyphs.entry_on,
            &glyphs.entry_off,
            &glyphs.proof_winnable,
            &glyphs.proof_unwinnable,
            &glyphs.proof_unknown,
            &glyphs.proof_unchecked,
            &glyphs.star,
            &glyphs.favourite,
            &glyphs.not_favourite,
        ]);
        for text in all {
            assert!(text.is_ascii(), "{text}");
        }
    }

    #[test]
    fn the_proof_marks_differ_in_both_sets() {
        for mode in [GlyphMode::Unicode, GlyphMode::Ascii] {
            let glyphs = Glyphs::new(mode);
            let marks = [
                glyphs.proof_winnable,
                glyphs.proof_unwinnable,
                glyphs.proof_unknown,
                glyphs.proof_unchecked,
            ];
            for (index, mark) in marks.iter().enumerate() {
                assert!(!mark.is_empty());
                assert!(marks[index + 1..].iter().all(|other| other != mark));
            }
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
    fn the_entry_signs_differ_in_both_sets() {
        for mode in [GlyphMode::Unicode, GlyphMode::Ascii] {
            let glyphs = Glyphs::new(mode);
            assert_ne!(glyphs.entry_on, glyphs.entry_off);
            assert_eq!(glyphs.entry_on.chars().count(), 1);
            assert_eq!(glyphs.entry_off.chars().count(), 1);
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
