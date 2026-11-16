//! The counters of the stock: how often the waste was turned over, which pass of a limited number
//! the game is in, how many row deals are left. Read from [`StockStatus`]; FreeCell has none.

use oxysol_core::{PileRef, Session, StockStatus};
use ratatui::layout::{Position, Rect};

use crate::i18n::tf;
use crate::layout::BoardGeometry;

/// The text of the stock counter for `status`; `stock_len` tells an empty stock from one that
/// still has cards, because "no passes left" only matters once the last pass is used up.
pub(super) fn stock_text_of(status: &StockStatus, stock_len: usize) -> String {
    if let Some(deals) = status.deals_left {
        return tf("board.counter.deals_left", &[("count", &deals.to_string())]);
    }
    let Some(left) = status.recycles_left else {
        return tf(
            "board.counter.turned_over",
            &[("count", &status.recycles.to_string())],
        );
    };
    let pass = (status.recycles + 1).to_string();
    let total = (status.recycles + left + 1).to_string();
    let params = [("pass", pass.as_str()), ("total", total.as_str())];
    match (left, stock_len) {
        (0, 0) => tf("board.counter.no_passes", &params),
        (0, _) => tf("board.counter.last_pass", &params),
        _ => tf("board.counter.pass", &params),
    }
}

/// The stock counter of a running game, `None` for a game without a stock.
pub(super) fn stock_text(session: &Session) -> Option<String> {
    let status = session.game().stock_status()?;
    let len = session
        .game()
        .piles()
        .iter()
        .find(|snapshot| snapshot.pile == PileRef::Stock)
        .map_or(0, |snapshot| snapshot.cards.len());
    Some(stock_text_of(&status, len))
}

/// Where `text` fits beside the stock: the row under the halo of the stock card, when no pile or
/// card occupies it and it lies inside `body`. `None` means there is no room, and the counter is
/// shown in the action line instead.
pub(super) fn beside_stock(geometry: &BoardGeometry, body: Rect, text: &str) -> Option<Rect> {
    let stock = geometry.pile(PileRef::Stock)?.slot;
    let width = u16::try_from(text.chars().count()).unwrap_or(u16::MAX);
    let row = Rect::new(stock.x, stock.bottom().saturating_add(1), width, 1);
    let room = body.contains(Position::new(row.x, row.y));
    let free = geometry.piles.iter().all(|pile| {
        !pile.area.intersects(row)
            && !pile.slot.intersects(row)
            && pile.cards.iter().all(|card| !card.intersects(row))
    });
    (room && free).then(|| row.intersection(body))
}
