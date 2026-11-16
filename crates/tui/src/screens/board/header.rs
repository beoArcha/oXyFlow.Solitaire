//! The header line of the board: game, seed, playing time, move count and, in a game without undo,
//! the "No undo" badge. The seed is a click target that copies it, so the header also says where
//! that target is.

use std::time::Duration;

use oxysol_core::Session;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::clock::format_elapsed;
use crate::i18n::{t, tf};
use crate::screens::variant_label_key;

/// The header text and the characters of it that spell the seed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Header {
    text: String,
    seed_start: usize,
    seed_len: usize,
}

fn text_of(session: &Session, elapsed: Duration, undo_allowed: bool) -> String {
    let text = tf(
        "board.header.format",
        &[
            ("variant", &t(variant_label_key(session.variant()))),
            ("seed", &session.seed().to_string()),
            ("time", &format_elapsed(elapsed)),
            ("moves", &session.moves_made().to_string()),
        ],
    );
    if undo_allowed {
        return text;
    }
    format!(
        "{text}{}{}",
        t("board.action.separator"),
        t("board.header.no_undo")
    )
}

/// Character index of the seed in `text`, searched after the game name so that a digit in the name
/// is not taken for the seed.
fn seed_start(text: &str, variant: &str, seed: &str) -> usize {
    let skipped = text.strip_prefix(variant).map_or(0, str::len);
    let found = text.get(skipped..).and_then(|rest| rest.find(seed));
    let byte = skipped + found.unwrap_or(0);
    text.get(..byte).map_or(0, |head| head.chars().count())
}

impl Header {
    pub(super) fn new(session: &Session, elapsed: Duration, undo_allowed: bool) -> Self {
        let text = text_of(session, elapsed, undo_allowed);
        let seed = session.seed().to_string();
        let variant = t(variant_label_key(session.variant())).into_owned();
        Self {
            seed_start: seed_start(&text, &variant, &seed),
            seed_len: seed.chars().count(),
            text,
        }
    }

    fn width(&self) -> usize {
        self.text.chars().count()
    }

    fn left(&self, area: Rect) -> u16 {
        let spare = usize::from(area.width).saturating_sub(self.width());
        area.x + u16::try_from(spare / 2).unwrap_or(0)
    }

    /// The cells of the seed when the header is centered in `area`.
    pub(super) fn seed_cells(&self, area: Rect) -> Rect {
        let start = u16::try_from(self.seed_start).unwrap_or(0);
        let len = u16::try_from(self.seed_len).unwrap_or(0);
        Rect::new(self.left(area) + start, area.y, len, 1).intersection(area)
    }

    /// The header as a line: the seed underlined, so that it reads as something to press.
    pub(super) fn line(&self, style: Style) -> Line<'static> {
        let chars: Vec<char> = self.text.chars().collect();
        let end = (self.seed_start + self.seed_len).min(chars.len());
        let start = self.seed_start.min(end);
        let piece = |from: usize, to: usize| chars[from..to].iter().collect::<String>();
        Line::from(vec![
            Span::styled(piece(0, start), style),
            Span::styled(piece(start, end), style.add_modifier(Modifier::UNDERLINED)),
            Span::styled(piece(end, chars.len()), style),
        ])
        .centered()
    }
}
