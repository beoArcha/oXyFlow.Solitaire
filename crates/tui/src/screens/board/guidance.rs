//! What the board tells the player about the moment: what is going on (`Now`), what can be done
//! (`Possible`, with the keys from the key table) and which actions are usable, for the guide
//! panel to highlight. Everything is read from the controller and from the legal moves the engine
//! lists; no rule is decided here.

use oxysol_core::{Move, PileRef, PileSnapshot, Session, Variant};

use super::choices::{Choice, quick, waste_quick};
use super::cursor::max_depth;
use super::guidance_parts::{ALWAYS, plain, run_part, stuck, tail, won};
pub(super) use super::guidance_parts::{Guidance, Part, part};
use super::priority;
use super::{BoardScreen, Pick, now};
use crate::action::Action;

fn quick_part(best: Choice, drop_order: u8) -> Part {
    let key = match best.target {
        Some(PileRef::Foundation(_)) => "board.action.quick_foundation",
        _ => "board.action.quick",
    };
    part(Action::QuickMove, key, drop_order)
}

fn waste_quick_part(best: Choice, drop_order: u8) -> Part {
    let key = match best.target {
        Some(PileRef::Foundation(_)) => "board.action.waste_quick_foundation",
        _ => "board.action.waste_quick",
    };
    part(Action::WasteQuickMove, key, drop_order)
}

const fn stock_key(variant: Variant) -> &'static str {
    match variant {
        Variant::Spider(_) => "board.action.deal_row",
        _ => "board.action.turn_stock",
    }
}

impl BoardScreen {
    /// The guidance for the position on the board; `None` without a game and while the board is
    /// covered.
    pub(super) fn guidance(&self) -> Option<Guidance> {
        let mut guidance = self.moment()?;
        if self.panel_unavailable() {
            guidance.tail.insert(
                0,
                part(Action::Guide, "board.action.guide", priority::GUIDE),
            );
        }
        Some(guidance)
    }

    fn moment(&self) -> Option<Guidance> {
        let session = self.session.as_ref().filter(|_| !self.covered)?;
        let game = session.game();
        if game.is_won() {
            return Some(won());
        }
        let legal = game.legal_moves();
        if legal.is_empty() {
            return Some(stuck(session, self.undo_allowed));
        }
        let piles = game.piles();
        Some(match &self.pick {
            Some(pick) => carrying(&piles, pick, self.quick_move),
            None => self.browsing(&piles, &legal, session),
        })
    }

    fn now_text(&self, piles: &[PileSnapshot]) -> String {
        let hint = self.hint.filter(|_| self.hint_note);
        match hint {
            Some(mv) => now::hint(piles, mv),
            None => now::browsing(piles, self.cursor.pile, self.cursor.depth),
        }
    }

    fn browsing(&self, piles: &[PileSnapshot], legal: &[Move], session: &Session) -> Guidance {
        let cursor = self.cursor;
        let snapshot = piles.iter().find(|snapshot| snapshot.pile == cursor.pile);
        let on_stock = cursor.pile == PileRef::Stock;
        let on_waste = cursor.pile == PileRef::Waste;
        let stock_ready = legal.contains(&Move::Stock);
        let found = self
            .candidates(cursor.pile, cursor.depth)
            .ok()
            .filter(|_| !on_stock);
        let best = found.as_ref().and_then(|(_, found)| quick(found));
        let waste_best = waste_quick(legal).filter(|_| self.quick_move);
        let runs = snapshot.is_some_and(|snapshot| max_depth(snapshot) > 1);
        let hint = session.game().hint().is_some();
        let variant = session.variant();

        let mut possible = Vec::new();
        let mut usable = Vec::new();
        if found.is_some() {
            possible.push(part(
                Action::Select,
                "board.action.pick_up",
                priority::PRIMARY,
            ));
            usable.push(Action::Select);
        } else if on_stock && stock_ready {
            possible.push(part(Action::Select, stock_key(variant), priority::PRIMARY));
            usable.push(Action::Select);
        }
        if let Some(best) = best.filter(|_| self.quick_move) {
            possible.push(quick_part(best, priority::QUICK));
            usable.push(Action::QuickMove);
        }
        if let Some(waste_best) = waste_best {
            if !on_waste {
                possible.push(waste_quick_part(waste_best, priority::WASTE_QUICK));
            }
            usable.push(Action::WasteQuickMove);
        }
        if stock_ready && !on_stock {
            possible.push(part(Action::Stock, stock_key(variant), priority::STOCK));
        }
        usable.extend(stock_ready.then_some(Action::Stock));
        possible.push(plain("board.action.move", priority::MOVE_KEYS));
        if hint {
            possible.push(part(Action::Hint, "board.action.hint", priority::HINT));
            usable.push(Action::Hint);
        }
        self.push_finish(&mut possible, &mut usable);
        if self.undo_allowed && session.can_undo() {
            possible.push(part(Action::Undo, "board.action.undo", priority::UNDO));
            usable.push(Action::Undo);
        }
        if runs {
            possible.push(run_part(priority::RUN));
        }
        usable.extend(runs.then_some(Action::RunMore));
        usable.extend(runs.then_some(Action::RunLess));
        if self.undo_allowed && session.can_redo() {
            possible.push(part(Action::Redo, "board.action.redo", priority::REDO));
            usable.push(Action::Redo);
        }
        possible.push(part(Action::Rules, "board.action.rules", priority::RULES));
        usable.extend(ALWAYS);
        Guidance {
            now: self.now_text(piles),
            shorter: Vec::new(),
            possible,
            tail: tail(true),
            usable,
        }
    }
}

fn carrying(piles: &[PileSnapshot], pick: &Pick, quick_move: bool) -> Guidance {
    let targets = pick
        .choices
        .iter()
        .filter(|choice| choice.target.is_some())
        .count();
    let mut possible = vec![
        part(Action::Select, "board.action.drop", priority::PRIMARY),
        part(Action::Back, "board.action.put_back", priority::PUT_BACK),
        plain("board.action.choose_target", priority::CHOOSE_TARGET),
    ];
    let mut usable = vec![Action::Select, Action::Back];
    if let Some(best) = quick(&pick.choices).filter(|_| quick_move) {
        possible.push(quick_part(best, priority::QUICK));
        usable.push(Action::QuickMove);
    }
    usable.extend(ALWAYS);
    Guidance {
        now: now::carrying(piles, pick.source, pick.count, targets),
        shorter: now::carrying_shorter(piles, pick.source, pick.count, targets),
        possible,
        tail: tail(false),
        usable,
    }
}
