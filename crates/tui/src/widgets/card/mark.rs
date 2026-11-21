//! The highlights of a card position and the symbols that carry them.

use crate::glyphs::Glyphs;
use crate::theme::Token;

/// The part a hinted card plays in the hinted move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hint {
    /// The card (or pile) the hint moves.
    Source,
    /// The pile the hint moves to.
    Target,
}

/// Highlights of a card, strongest first: selected, focus, target, hint. `blocked` is not a
/// highlight: a card that a card still on the table covers is drawn in the blocked style, with a
/// dimmer border, unless one of the highlights is on it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Mark {
    pub focus: bool,
    pub selected: bool,
    pub target: bool,
    pub hint: Option<Hint>,
    pub blocked: bool,
}

impl Mark {
    pub(super) fn token(self) -> Option<Token> {
        if self.selected {
            Some(Token::Selection)
        } else if self.focus {
            Some(Token::BorderFocus)
        } else if self.target {
            Some(Token::Target)
        } else if self.hint.is_some() {
            Some(Token::Hint)
        } else {
            None
        }
    }

    /// The symbols of the top row, most important first. A target is marked in the corners and
    /// in the middle of an empty slot instead.
    pub(super) fn symbols(self, glyphs: &Glyphs) -> Vec<&'static str> {
        let hint = self.hint.map(|role| match role {
            Hint::Source => glyphs.hint_source,
            Hint::Target => glyphs.hint_target,
        });
        [
            self.selected.then_some(glyphs.selected),
            self.focus.then_some(glyphs.focus_left),
            hint,
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}
