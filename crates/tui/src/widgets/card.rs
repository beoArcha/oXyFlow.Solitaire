//! The card widget v1: a front with rank and suit in the corners, a back, and an empty slot, in
//! the International style. Style-driven rendering arrives in P5-02.
//!
//! The rank and suit sit in the top border row, so a card overlapped by the next card of its
//! column still shows them. Focus, selection, target and hint change the border shape and add a
//! marker symbol: none of them depends on color.

use oxysol_core::{Card, Color as CardColor, Rank};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Style;

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
            (self.target, "+"),
            (self.hint, "?"),
        ]
        .into_iter()
        .filter_map(|(on, symbol)| on.then_some(symbol))
        .collect()
    }
}

struct Edges {
    corners: [&'static str; 4],
    horizontal: &'static str,
    vertical: &'static str,
}

const fn edges(
    corners: [&'static str; 4],
    horizontal: &'static str,
    vertical: &'static str,
) -> Edges {
    Edges {
        corners,
        horizontal,
        vertical,
    }
}

struct EdgeSets {
    normal: Edges,
    selected: Edges,
    focus: Edges,
    target: Edges,
    hint: Edges,
}

const UNICODE_EDGES: EdgeSets = EdgeSets {
    normal: edges(["┌", "┐", "└", "┘"], "─", "│"),
    selected: edges(["╒", "╕", "╘", "╛"], "═", "│"),
    focus: edges(["╔", "╗", "╚", "╝"], "═", "║"),
    target: edges(["╓", "╖", "╙", "╜"], "─", "║"),
    hint: edges(["░", "░", "░", "░"], "░", "░"),
};

const ASCII_EDGES: EdgeSets = EdgeSets {
    normal: edges(["+", "+", "+", "+"], "-", "|"),
    selected: edges(["*", "*", "*", "*"], "*", "*"),
    focus: edges(["@", "@", "@", "@"], "=", "@"),
    target: edges(["+", "+", "+", "+"], "~", "!"),
    hint: edges(["+", "+", "+", "+"], ".", ":"),
};

fn edges_for(glyphs: &Glyphs, mark: Mark) -> &'static Edges {
    let sets = if glyphs.unicode {
        &UNICODE_EDGES
    } else {
        &ASCII_EDGES
    };
    if mark.selected {
        &sets.selected
    } else if mark.focus {
        &sets.focus
    } else if mark.target {
        &sets.target
    } else if mark.hint {
        &sets.hint
    } else {
        &sets.normal
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

struct Painter<'a> {
    buf: &'a mut Buffer,
    clip: Rect,
}

impl Painter<'_> {
    fn put(&mut self, x: u16, y: u16, symbol: &str, style: Style) {
        if !self.clip.contains(Position::new(x, y)) {
            return;
        }
        if let Some(cell) = self.buf.cell_mut((x, y)) {
            cell.set_symbol(symbol).set_style(style);
        }
    }

    fn text(&mut self, x: u16, y: u16, text: &str, style: Style) {
        let mut at = x;
        for symbol in text.chars() {
            self.put(at, y, symbol.encode_utf8(&mut [0; 4]), style);
            at = at.saturating_add(1);
        }
    }

    fn fill(&mut self, area: Rect, symbol: &str, style: Style) {
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                self.put(x, y, symbol, style);
            }
        }
    }
}

fn width_of(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(u16::MAX)
}

fn centered(area: Rect, text: &str) -> u16 {
    area.x + area.width.saturating_sub(width_of(text)) / 2
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

fn paint_frame(painter: &mut Painter<'_>, area: Rect, edges: &Edges, border: Style) {
    let (left, right) = (area.left(), area.right() - 1);
    let (top, bottom) = (area.top(), area.bottom() - 1);
    for x in left + 1..right {
        painter.put(x, top, edges.horizontal, border);
        painter.put(x, bottom, edges.horizontal, border);
    }
    for y in top + 1..bottom {
        painter.put(left, y, edges.vertical, border);
        painter.put(right, y, edges.vertical, border);
    }
    let [top_left, top_right, bottom_left, bottom_right] = edges.corners;
    painter.put(left, top, top_left, border);
    painter.put(right, top, top_right, border);
    painter.put(left, bottom, bottom_left, border);
    painter.put(right, bottom, bottom_right, border);
}

fn paint_center(
    painter: &mut Painter<'_>,
    area: Rect,
    card: Card,
    ctx: &ViewCtx<'_>,
    style: Style,
) {
    let middle = area.y + area.height / 2;
    let suit = ctx.glyphs.suit(card.suit);
    painter.text(centered(area, suit), middle, suit, style);
    if area.height >= 7 {
        let rank = rank_text(card.rank);
        painter.text(centered(area, &rank), middle - 1, &rank, style);
    }
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
        paint_center(&mut painter, area, card, ctx, look.fill);
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
