//! The live preview pane of the settings screen: one face-down card and three face-up cards (the
//! focused card, a carried card and a drop target), drawn on the table of the theme by the card
//! widget and framed by the aura painter of the board, so every look setting shows at once.

use oxysol_core::{Card, CardId, PileRef, Rank, Suit};
use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

use super::plan::{CARDS, card_gap, content_rows, content_width};
use crate::i18n::t;
use crate::layout::{BoardGeometry, CardSize, PileGeometry};
use crate::screens::ViewCtx;
use crate::theme::Token;
use crate::widgets::aura;
use crate::widgets::card::{self, Face, Mark};
use crate::widgets::pile::{CardPos, Marks};

const FOCUS_CARD: Card = Card {
    id: CardId(0),
    suit: Suit::Hearts,
    rank: Rank::Queen,
};
const CARRIED_CARD: Card = Card {
    id: CardId(1),
    suit: Suit::Spades,
    rank: Rank::Jack,
};
const TARGET_CARD: Card = Card {
    id: CardId(2),
    suit: Suit::Diamonds,
    rank: Rank::King,
};

const BACK: u8 = 0;
const FOCUS: u8 = 1;
const CARRIED: u8 = 2;
const TARGET: u8 = 3;

const LABEL_KEYS: [&str; CARDS as usize] = [
    "settings.pane.back",
    "settings.pane.focus",
    "settings.pane.carried",
    "settings.pane.target",
];

fn pile(slot: u8) -> PileRef {
    PileRef::Tableau(slot)
}

fn face_and_mark(slot: u8, ctx: &ViewCtx<'_>) -> (Face, Mark) {
    match slot {
        BACK => (Face::Down, Mark::default()),
        FOCUS => (
            Face::Up(FOCUS_CARD),
            Mark {
                focus: true,
                ..Mark::default()
            },
        ),
        CARRIED => (
            Face::Up(CARRIED_CARD),
            Mark {
                selected: true,
                ..Mark::default()
            },
        ),
        _ => (
            Face::Up(TARGET_CARD),
            Mark {
                target: ctx.settings.target_marks,
                ..Mark::default()
            },
        ),
    }
}

/// The largest card size the player's card-size setting allows that the pane holds.
pub(super) fn card_size(area: Rect, rings: u16, ctx: &ViewCtx<'_>) -> CardSize {
    CardSize::largest_fit_within(ctx.settings.card_size.size(), |size| {
        content_width(size, rings) <= area.width && content_rows(size, rings) <= area.height
    })
}

fn card_rects(area: Rect, size: CardSize, rings: u16) -> [Rect; CARDS as usize] {
    let gap = card_gap(rings);
    let used_width = content_width(size, rings);
    let used_rows = content_rows(size, rings);
    let left = area.x + area.width.saturating_sub(used_width) / 2 + rings;
    let top = area.y + 1 + area.height.saturating_sub(used_rows) / 2 + rings;
    std::array::from_fn(|index| {
        let offset = u16::try_from(index).unwrap_or(0) * (size.width + gap);
        Rect::new(left + offset, top, size.width, size.height)
    })
}

fn geometry(rects: &[Rect; CARDS as usize], size: CardSize) -> BoardGeometry {
    let piles = (0..CARDS)
        .zip(rects)
        .map(|(slot, rect)| PileGeometry {
            pile: pile(u8::try_from(slot).unwrap_or(0)),
            slot: *rect,
            area: *rect,
            cards: vec![*rect],
            collapsed: 0,
        })
        .collect();
    BoardGeometry { card: size, piles }
}

fn marks() -> Marks {
    Marks {
        focus: Some(CardPos {
            pile: pile(FOCUS),
            index: 0,
        }),
        selected: Some(CardPos {
            pile: pile(CARRIED),
            index: 0,
        }),
        targets: vec![pile(TARGET)],
        ..Marks::default()
    }
}

fn truncated(text: &str, width: u16) -> String {
    text.chars().take(usize::from(width)).collect()
}

fn draw_labels(
    frame: &mut Frame,
    (area, rects): (Rect, &[Rect; CARDS as usize]),
    rings: u16,
    ctx: &ViewCtx<'_>,
) {
    let style = ctx.theme.style(Token::TableText, Token::Table);
    let slot = rects.first().map_or(0, |rect| rect.width) + card_gap(rings);
    for (rect, key) in rects.iter().zip(LABEL_KEYS) {
        let x = (rect.x + rect.width / 2)
            .saturating_sub(slot / 2)
            .max(area.x);
        let width = slot.min(area.right().saturating_sub(x));
        let area = Rect::new(x, rect.bottom() + rings, width, 1);
        let text = truncated(&t(key), slot);
        frame.render_widget(
            Paragraph::new(Line::styled(text, style)).alignment(Alignment::Center),
            area,
        );
    }
}

/// Draws the pane into `area`: its title, the four cards with their auras, and a label under each.
pub(super) fn draw(frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
    let table = ctx.theme.style(Token::TableText, Token::Table);
    frame.render_widget(Block::new().style(table), area);
    let title = Line::styled(t("settings.pane.title").into_owned(), table);
    frame.render_widget(
        Paragraph::new(title).alignment(Alignment::Center),
        Rect::new(area.x, area.y, area.width, 1),
    );
    let rings = ctx.settings.aura_size.rings();
    let size = card_size(area, rings, ctx);
    let rects = card_rects(area, size, rings);
    for (slot, rect) in (0..CARDS).zip(rects) {
        let (face, mark) = face_and_mark(u8::try_from(slot).unwrap_or(0), ctx);
        card::draw(frame.buffer_mut(), rect, face, mark, None, ctx);
    }
    aura::draw(
        frame.buffer_mut(),
        area,
        &geometry(&rects, size),
        &marks(),
        ctx,
    );
    draw_labels(frame, (area, &rects), rings, ctx);
}
