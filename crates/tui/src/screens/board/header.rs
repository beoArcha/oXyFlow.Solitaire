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
    let skipped = if text.starts_with(variant) {
        variant.len()
    } else {
        0
    };
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

    /// Where the line starts: the same rule the renderer uses to center it.
    fn left(&self, area: Rect) -> u16 {
        let offset = (usize::from(area.width) / 2).saturating_sub(self.width() / 2);
        area.x + u16::try_from(offset).unwrap_or(0)
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

#[cfg(test)]
mod tests {
    use oxysol_core::{DrawCount, Variant};

    use super::*;
    use crate::i18n::{Locale, set_locale};

    fn header(seed: u64, undo_allowed: bool) -> Header {
        let session = Session::new(Variant::Klondike(DrawCount::One), seed);
        Header::new(&session, Duration::from_secs(75), undo_allowed)
    }

    #[test]
    fn the_seed_is_found_after_the_game_name_even_when_the_name_holds_digits() {
        for seed in [1, 42, u64::MAX] {
            let header = header(seed, true);
            let text: Vec<char> = header.text.chars().collect();
            let spelled: String = text[header.seed_start..header.seed_start + header.seed_len]
                .iter()
                .collect();
            assert_eq!(spelled, seed.to_string(), "{}", header.text);
        }
    }

    #[test]
    fn the_seed_cells_are_where_the_centered_line_draws_the_seed() {
        for undo_allowed in [true, false] {
            for width in [100_u16, 101, 120] {
                let header = header(42, undo_allowed);
                let area = Rect::new(0, 0, width, 1);
                let mut buffer = ratatui::buffer::Buffer::empty(area);
                ratatui::widgets::Widget::render(
                    ratatui::widgets::Paragraph::new(header.line(Style::default())),
                    area,
                    &mut buffer,
                );
                let cells = header.seed_cells(area);
                let drawn: String = (cells.left()..cells.right())
                    .map(|x| buffer[(x, 0)].symbol().to_owned())
                    .collect();
                assert_eq!(drawn, "42", "width {width}");
            }
        }
    }

    #[test]
    fn the_header_names_moves_and_the_badge_in_both_languages() {
        for (locale, moves, badge) in [
            (Locale::EnUs, "Moves: 0", "No undo"),
            (Locale::PlPl, "Ruchy: 0", "Bez cofania"),
        ] {
            set_locale(locale);
            assert!(header(7, true).text.contains(moves));
            assert!(!header(7, true).text.contains(badge));
            assert!(header(7, false).text.contains(badge));
        }
        set_locale(Locale::EnUs);
    }
}
