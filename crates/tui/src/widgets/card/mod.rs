//! The card widget v1: a front with rank and suit in the corners and a large rank and suit
//! pictogram in the centre, a back, and an empty slot, in the International style. Style-driven
//! rendering arrives in P5-02.
//!
//! The rank and suit sit in the top border row, so a card overlapped by the next card of its
//! column still shows them. Focus and selection change the border shape; focus, selection, target
//! and hint each add a marker symbol. None of them depends on color. The Unicode borders are WGL4.

use oxysol_core::{Card, Color as CardColor, Rank};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

mod art;
mod face;
mod frame;
mod paint;
mod rank_font;
#[cfg(test)]
mod tests;

use self::face::paint_center;
use self::frame::{edges_for, paint_frame};
use self::paint::{Painter, centered, width_of};
use crate::glyphs::Glyphs;
use crate::i18n::t;
use crate::screens::ViewCtx;
use crate::theme::Token;

/// What a card position shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    Up(Card),
    Down,
    Slot,
}

/// Highlights of a card, strongest first: selected, focus, target, hint.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Mark {
    pub focus: bool,
    pub selected: bool,
    pub target: bool,
    pub hint: bool,
}

impl Mark {
    fn token(self) -> Option<Token> {
        if self.selected {
            Some(Token::Selection)
        } else if self.focus {
            Some(Token::BorderFocus)
        } else if self.target {
            Some(Token::Target)
        } else if self.hint {
            Some(Token::Hint)
        } else {
            None
        }
    }

    fn symbols(self, glyphs: &Glyphs) -> Vec<&'static str> {
        [
            (self.selected, glyphs.selected),
            (self.focus, glyphs.focus_left),
            (self.target, glyphs.target),
            (self.hint, "?"),
        ]
        .into_iter()
        .filter_map(|(on, symbol)| on.then_some(symbol))
        .collect()
    }
}

/// The localized label of a rank: `A`, `2` … `10`, `J`, `Q`, `K`.
pub fn rank_text(rank: Rank) -> String {
    let key = match rank {
        Rank::Ace => "board.rank.ace",
        Rank::Two => "board.rank.two",
        Rank::Three => "board.rank.three",
        Rank::Four => "board.rank.four",
        Rank::Five => "board.rank.five",
        Rank::Six => "board.rank.six",
        Rank::Seven => "board.rank.seven",
        Rank::Eight => "board.rank.eight",
        Rank::Nine => "board.rank.nine",
        Rank::Ten => "board.rank.ten",
        Rank::Jack => "board.rank.jack",
        Rank::Queen => "board.rank.queen",
        Rank::King => "board.rank.king",
    };
    t(key).into_owned()
}

/// The corner label of a card: rank and suit, for example `10♥` or `10H`.
pub fn label(card: Card, glyphs: &Glyphs) -> String {
    format!("{}{}", rank_text(card.rank), glyphs.suit(card.suit))
}

fn color_token(card: Card) -> Token {
    match card.suit.color() {
        CardColor::Red => Token::CardRed,
        CardColor::Black => Token::CardBlack,
    }
}

struct Look {
    fill: Style,
    border: Style,
}

fn look(face: Face, mark: Mark, ctx: &ViewCtx<'_>) -> Look {
    let theme = ctx.theme;
    let fill = match face {
        Face::Up(card) => theme.style(color_token(card), Token::CardFace),
        Face::Down => theme.style(Token::CardBackMark, Token::CardBack),
        Face::Slot => theme.style(Token::Slot, Token::Table),
    };
    let border = match mark.token() {
        Some(token) => theme.style(token, Token::Table),
        None => match face {
            Face::Slot => theme.style(Token::Slot, Token::Table),
            _ => fill,
        },
    };
    Look { fill, border }
}

fn paint_top_row(
    painter: &mut Painter<'_>,
    area: Rect,
    face: Face,
    mark: Mark,
    ctx: &ViewCtx<'_>,
    look: &Look,
) {
    let inner = area.width.saturating_sub(2);
    let label_width = match face {
        Face::Up(card) => {
            let text = label(card, ctx.glyphs);
            painter.text(area.x + 1, area.y, &text, look.fill);
            width_of(&text)
        }
        _ => 0,
    };
    let room = usize::from(inner.saturating_sub(label_width));
    let symbols = mark.symbols(ctx.glyphs);
    let shown: String = symbols.into_iter().take(room).collect();
    let x = area.right() - 1 - width_of(&shown);
    painter.text(x, area.y, &shown, look.border);
}

fn paint_bottom_row(
    painter: &mut Painter<'_>,
    area: Rect,
    face: Face,
    badge: Option<usize>,
    ctx: &ViewCtx<'_>,
    look: &Look,
) {
    let text = match (badge, face) {
        (Some(count), _) => Some(count.to_string()),
        (None, Face::Up(card)) => Some(label(card, ctx.glyphs)),
        _ => None,
    };
    let Some(text) = text else { return };
    let x = (area.right() - 1).saturating_sub(width_of(&text));
    let style = match face {
        Face::Slot => look.border,
        _ => look.fill,
    };
    painter.text(x.max(area.x + 1), area.bottom() - 1, &text, style);
}

/// Draws one card position into `area`. `badge` is a count shown in the bottom corner (the size
/// of a stock). The widget draws the rows that fit `area`; a card overlapped by another keeps its
/// top rows.
pub fn draw(
    buf: &mut Buffer,
    area: Rect,
    face: Face,
    mark: Mark,
    badge: Option<usize>,
    ctx: &ViewCtx<'_>,
) {
    if area.width < 3 || area.height < 2 {
        return;
    }
    let look = look(face, mark, ctx);
    let mut painter = Painter { buf, clip: area };
    let interior = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);
    match face {
        Face::Up(_) => painter.fill(interior, " ", look.fill),
        Face::Down => painter.fill(interior, ctx.glyphs.face_down, look.fill),
        Face::Slot => painter.fill(interior, " ", look.border),
    }
    paint_frame(&mut painter, area, edges_for(ctx.glyphs, mark), look.border);
    paint_top_row(&mut painter, area, face, mark, ctx, &look);
    paint_bottom_row(&mut painter, area, face, badge, ctx, &look);
    if let Face::Up(card) = face {
        paint_center(&mut painter, area, card, ctx.glyphs, look.fill);
    }
}

/// Draws the one-row stand-in for `count` collapsed face-down cards.
pub fn draw_collapsed(buf: &mut Buffer, area: Rect, count: usize, ctx: &ViewCtx<'_>) {
    let style = ctx.theme.style(Token::CardBackMark, Token::CardBack);
    let mut painter = Painter { buf, clip: area };
    painter.fill(area, ctx.glyphs.face_down, style);
    let text = format!(" x{count} ");
    painter.text(centered(area, &text), area.y, &text, style);
}
