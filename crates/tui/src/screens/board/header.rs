//! The game information of the board, in the one place the screen shows it: variant, seed, playing
//! time, moves, the stock counter and, in a game without undo, the "No undo" badge. A Compact
//! screen draws the facts in one header row, a Standard or Wide screen in the `Game` box of the
//! side panel. The seed is a click target that copies it, so it is underlined in both places.

use std::time::Duration;

use oxysol_core::Session;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use super::counters;
use crate::clock::format_elapsed;
use crate::i18n::{t, tf};
use crate::screens::variant_label_key;

/// Index of the seed among the facts, and so the row of it in the `Game` box.
pub(super) const SEED_ROW: u16 = 1;

/// The facts of the game, each one a short text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Facts {
    facts: Vec<String>,
    counter: Option<usize>,
    seed_at: usize,
    seed_len: usize,
}

/// The header row: where it is drawn, what is drawn and the cells of the seed.
pub(super) struct Row {
    pub area: Rect,
    pub line: Line<'static>,
    pub seed: Rect,
}

fn cells(text: &str) -> usize {
    text.chars().count()
}

fn span_of(text: &str, from: usize, to: usize) -> String {
    text.chars().skip(from).take(to - from).collect()
}

impl Facts {
    pub(super) fn new(session: &Session, elapsed: Duration, undo_allowed: bool) -> Self {
        let seed = session.seed().to_string();
        let seed_fact = tf("board.header.seed", &[("seed", &seed)]);
        let seed_at = seed_fact
            .find(&seed)
            .map_or(0, |byte| cells(&seed_fact[..byte]));
        let mut facts = vec![
            t(variant_label_key(session.variant())).into_owned(),
            seed_fact,
            tf("board.header.time", &[("time", &format_elapsed(elapsed))]),
            tf(
                "board.header.moves",
                &[("moves", &session.moves_made().to_string())],
            ),
        ];
        let counter = counters::stock_text(session).map(|text| {
            facts.push(text);
            facts.len() - 1
        });
        if !undo_allowed {
            facts.push(t("board.header.no_undo").into_owned());
        }
        Self {
            facts,
            counter,
            seed_at,
            seed_len: cells(&seed),
        }
    }

    fn joined_width(facts: &[&String]) -> usize {
        let separator = cells(&t("board.action.separator"));
        facts.iter().map(|fact| cells(fact)).sum::<usize>()
            + separator * facts.len().saturating_sub(1)
    }

    fn fitting(&self, width: usize) -> Vec<&String> {
        let mut shown: Vec<&String> = self.facts.iter().collect();
        if Self::joined_width(&shown) > width {
            if let Some(index) = self.counter {
                shown.remove(index);
            }
        }
        shown
    }

    /// The header row for `area`: the facts that fit, centered, the seed underlined.
    pub(super) fn row(&self, area: Rect, style: Style) -> Row {
        let width = usize::from(area.width);
        let shown = self.fitting(width);
        let separator = t("board.action.separator");
        let total = Self::joined_width(&shown).min(width);
        let left = (width / 2).saturating_sub(total / 2);
        let mut spans = Vec::new();
        let mut offset = 0;
        let mut seed = Rect::new(area.x, area.y, 0, 1);
        for (index, fact) in shown.iter().enumerate() {
            if index > 0 {
                spans.push(Span::styled(separator.to_string(), style));
                offset += cells(&separator);
            }
            if index == usize::from(SEED_ROW) {
                let start = u16::try_from(left + offset + self.seed_at).unwrap_or(0);
                let len = u16::try_from(self.seed_len).unwrap_or(0);
                seed = Rect::new(area.x + start, area.y, len, 1).intersection(area);
            }
            spans.extend(self.spans(index, fact, style));
            offset += cells(fact);
        }
        let x = area.x + u16::try_from(left).unwrap_or(0);
        let rest = area.width.saturating_sub(x - area.x);
        Row {
            area: Rect::new(x, area.y, rest, 1),
            line: Line::from(spans),
            seed,
        }
    }

    fn spans(&self, index: usize, fact: &str, style: Style) -> Vec<Span<'static>> {
        if index != usize::from(SEED_ROW) {
            return vec![Span::styled(fact.to_owned(), style)];
        }
        let end = self.seed_at + self.seed_len;
        vec![
            Span::styled(span_of(fact, 0, self.seed_at), style),
            Span::styled(
                span_of(fact, self.seed_at, end),
                style.add_modifier(Modifier::UNDERLINED),
            ),
            Span::styled(span_of(fact, end, cells(fact)), style),
        ]
    }

    /// The facts as the lines of the `Game` box, one per fact, the seed underlined.
    pub(super) fn lines(&self, style: Style) -> Vec<Line<'static>> {
        self.facts
            .iter()
            .enumerate()
            .map(|(index, fact)| Line::from(self.spans(index, fact, style)))
            .collect()
    }

    /// The cells of the seed when the facts are the lines of a box whose text starts at `inner`.
    pub(super) fn seed_in_box(&self, inner: Rect) -> Rect {
        let start = u16::try_from(self.seed_at).unwrap_or(0);
        let len = u16::try_from(self.seed_len).unwrap_or(0);
        Rect::new(inner.x + start, inner.y + SEED_ROW, len, 1).intersection(inner)
    }
}
