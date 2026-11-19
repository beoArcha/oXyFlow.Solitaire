//! The counters of the stock: how often the waste was turned over, which pass of a limited number
//! the game is in, how many row deals are left. Read from [`StockStatus`]; FreeCell has none.

use oxysol_core::{PileRef, Session, StockStatus};

use crate::i18n::tf;

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

/// Whether the waste can be turned back into an empty stock: the waste has cards and the game
/// still allows a pass (`recycles_left` is not `Some(0)`, the engine's word for "no recycle").
pub(super) fn can_recycle(session: &Session) -> bool {
    let waste = session
        .game()
        .piles()
        .iter()
        .find(|snapshot| snapshot.pile == PileRef::Waste)
        .is_some_and(|snapshot| !snapshot.cards.is_empty());
    let passes_left = session
        .game()
        .stock_status()
        .is_some_and(|status| status.recycles_left != Some(0));
    waste && passes_left
}
