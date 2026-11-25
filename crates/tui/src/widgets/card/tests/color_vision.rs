use oxysol_core::{Card, CardId, Rank, Suit};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::{Fixture, SIZES};
use crate::settings::{ColorVision, GlyphMode};
use crate::widgets::card::{Face, Mark, draw};

#[test]
fn four_color_deck_gives_four_distinct_suit_colors_at_every_card_size() {
    let suits = [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs];
    for vision in ColorVision::ALL {
        let mut fixture = Fixture::new(GlyphMode::Unicode);
        fixture.settings.color_vision = vision;
        fixture.settings.four_color_deck = true;
        let view = fixture.ctx();
        for (width, height) in SIZES {
            let mut fg_colors = Vec::new();
            for (index, suit) in suits.iter().enumerate() {
                let card = Card {
                    id: CardId(u16::try_from(index).unwrap_or(0)),
                    suit: *suit,
                    rank: Rank::Ace,
                };
                let area = Rect::new(0, 0, width, height);
                let mut buf = Buffer::empty(area);
                draw(&mut buf, area, Face::Up(card), Mark::default(), None, &view);
                let fg = buf[(1, 0)].fg;
                fg_colors.push(fg);
            }
            for (i, c1) in fg_colors.iter().enumerate() {
                for c2 in &fg_colors[i + 1..] {
                    assert_ne!(
                        c1, c2,
                        "vision={vision:?} size={width}x{height} colors={fg_colors:?}"
                    );
                }
            }
        }
    }
}
