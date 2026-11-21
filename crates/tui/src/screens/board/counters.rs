//! The counters of the stock: how often the waste was recycled into the stock, which pass of a limited number
//! the game is in, how many row deals are left. Read from [`StockStatus`]; FreeCell has none.

use oxysol_core::{Move, PileRef, Session, StockStatus};

use crate::i18n::tf;

/// The text of the stock counter for `status`; `stock_len` tells an empty stock from one that
/// still has cards, because "no passes left" only matters once the last pass is used up.
pub(super) fn stock_text_of(status: &StockStatus, stock_len: usize) -> String {
    if let Some(deals) = status.deals_left {
        return tf("board.counter.deals_left", &[("count", &deals.to_string())]);
    }
    let Some(left) = status.recycles_left else {
        return tf(
            "board.counter.recycled",
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

/// Whether an empty stock can be turned over again: the engine lists [`Move::Stock`] as legal then,
/// and only then (the waste has cards and a pass is left). The board does not repeat that rule.
pub(super) fn can_recycle(session: &Session) -> bool {
    let stock_empty = session
        .game()
        .piles()
        .iter()
        .find(|snapshot| snapshot.pile == PileRef::Stock)
        .is_some_and(|snapshot| snapshot.cards.is_empty());
    stock_empty && session.game().legal_moves().contains(&Move::Stock)
}
