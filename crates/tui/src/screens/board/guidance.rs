//! What the board tells the player about the moment: what is going on (`Now`), what can be done
//! (`Possible`, with the keys from the key table) and which actions are usable, for the guide
//! panel to highlight. Everything is read from the controller and from the legal moves the engine
//! lists; no rule is decided here.

use oxysol_core::{Move, PileRef, PileSnapshot, Session, Variant};

use super::choices::{Choice, quick};
use super::cursor::max_depth;
use super::{BoardScreen, Pick, now};
use crate::action::Action;
use crate::i18n::{t, tf};
use crate::input::primary_key;
use crate::widgets::text::clip;

/// One thing the player can do, as a short text with its key; the part with the highest
/// `drop_order` goes first when the line is too wide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Part {
    pub text: String,
    pub drop_order: u8,
}

/// The moment on the board: what is in progress, what can be done, and which actions work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Guidance {
    pub now: String,
    pub possible: Vec<Part>,
    /// Parts that are never dropped: the way to the key list and the way out.
    pub tail: Vec<Part>,
    pub usable: Vec<Action>,
}

const ALWAYS: [Action; 12] = [
    Action::Up,
    Action::Down,
    Action::Left,
    Action::Right,
    Action::Help,
    Action::Rules,
    Action::Guide,
    Action::Restart,
    Action::NewGame,
    Action::Pause,
    Action::Menu,
    Action::Quit,
];

pub(super) fn part(action: Action, key: &str, drop_order: u8) -> Part {
    let name = primary_key(action).unwrap_or_default();
    Part {
        text: tf(key, &[("key", &name)]),
        drop_order,
    }
}

fn plain(key: &str, drop_order: u8) -> Part {
    Part {
        text: t(key).into_owned(),
        drop_order,
    }
}

fn quick_part(best: Choice, drop_order: u8) -> Part {
    let key = match best.target {
        Some(PileRef::Foundation(_)) => "board.action.quick_foundation",
        _ => "board.action.quick",
    };
    part(Action::QuickMove, key, drop_order)
}

const fn stock_key(variant: Variant) -> &'static str {
    match variant {
        Variant::Spider(_) => "board.action.deal_row",
        _ => "board.action.turn_stock",
    }
}

fn join(parts: &[Part]) -> String {
    parts
        .iter()
        .map(|part| part.text.as_str())
        .collect::<Vec<_>>()
        .join(&t("board.action.separator"))
}

fn run_part(drop_order: u8) -> Part {
    let keys = [Action::RunMore, Action::RunLess]
        .into_iter()
        .filter_map(primary_key)
        .collect::<Vec<_>>()
        .join("/");
    Part {
        text: tf("board.action.run", &[("keys", &keys)]),
        drop_order,
    }
}

impl Guidance {
    /// The `Now:` row text cut to `width` columns.
    pub(super) fn now_line(&self, width: usize) -> String {
        clip(&tf("board.action.now", &[("text", &self.now)]), width)
    }

    /// The `Do:` row: the parts that fit `width` columns, least useful dropped first, then the
    /// tail, which always stays.
    pub(super) fn possible_line(&self, width: usize) -> String {
        let mut parts = self.possible.clone();
        loop {
            let all: Vec<Part> = parts.iter().chain(&self.tail).cloned().collect();
            let line = tf("board.action.possible", &[("text", &join(&all))]);
            if line.chars().count() <= width || parts.is_empty() {
                return clip(&line, width);
            }
            let least = parts
                .iter()
                .enumerate()
                .max_by_key(|(_, part)| part.drop_order)
                .map(|(index, _)| index);
            if let Some(index) = least {
                parts.remove(index);
            }
        }
    }

    fn closed(now_key: &str, possible: Vec<Part>, mut usable: Vec<Action>) -> Self {
        usable.extend(ALWAYS);
        Self {
            now: t(now_key).into_owned(),
            possible,
            tail: tail(true),
            usable,
        }
    }
}

/// The parts that end every `Do:` row: how to see all keys and, unless Esc is taken by a pick,
/// how to leave for the menu.
fn tail(with_menu: bool) -> Vec<Part> {
    let mut parts = vec![part(Action::Help, "board.action.help", 0)];
    if with_menu {
        parts.push(part(Action::Back, "board.action.menu", 0));
    }
    parts
}

fn won() -> Guidance {
    Guidance::closed(
        "board.action.won",
        vec![part(Action::NewGame, "board.action.new_game", 0)],
        Vec::new(),
    )
}

fn stuck(session: &Session, undo_allowed: bool) -> Guidance {
    let mut possible = Vec::new();
    let mut usable = Vec::new();
    if undo_allowed && session.can_undo() {
        possible.push(part(Action::Undo, "board.action.undo", 3));
        usable.push(Action::Undo);
    }
    possible.push(part(Action::Restart, "board.action.restart", 0));
    possible.push(part(Action::NewGame, "board.action.new_game", 1));
    Guidance::closed("board.action.stuck", possible, usable)
}

impl BoardScreen {
    /// The guidance for the position on the board; `None` without a game and while the board is
    /// covered.
    pub(super) fn guidance(&self) -> Option<Guidance> {
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
            Some(pick) => carrying(&piles, pick),
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
        let stock_ready = legal.contains(&Move::Stock);
        let found = self
            .candidates(cursor.pile, cursor.depth)
            .ok()
            .filter(|_| !on_stock);
        let best = found.as_ref().and_then(|(_, found)| quick(found));
        let deeper = snapshot.is_some_and(|snapshot| cursor.depth < max_depth(snapshot));
        let shallower = cursor.depth > 1;
        let hint = session.game().hint().is_some();
        let variant = session.variant();

        let mut possible = Vec::new();
        let mut usable = Vec::new();
        if found.is_some() {
            possible.push(part(Action::Select, "board.action.pick_up", 0));
            usable.push(Action::Select);
        } else if on_stock && stock_ready {
            possible.push(part(Action::Select, stock_key(variant), 0));
            usable.push(Action::Select);
        }
        if let Some(best) = best {
            possible.push(quick_part(best, 1));
            usable.push(Action::QuickMove);
        }
        if stock_ready && !on_stock {
            possible.push(part(Action::Stock, stock_key(variant), 2));
        }
        usable.extend(stock_ready.then_some(Action::Stock));
        possible.push(plain("board.action.move", 6));
        if hint {
            possible.push(part(Action::Hint, "board.action.hint", 3));
            usable.push(Action::Hint);
        }
        self.push_finish(&mut possible, &mut usable);
        if self.undo_allowed && session.can_undo() {
            possible.push(part(Action::Undo, "board.action.undo", 4));
            usable.push(Action::Undo);
        }
        if deeper || shallower {
            possible.push(run_part(5));
        }
        usable.extend(deeper.then_some(Action::RunMore));
        usable.extend(shallower.then_some(Action::RunLess));
        if self.undo_allowed && session.can_redo() {
            possible.push(part(Action::Redo, "board.action.redo", 7));
            usable.push(Action::Redo);
        }
        possible.push(part(Action::Rules, "board.action.rules", 9));
        usable.extend(ALWAYS);
        Guidance {
            now: self.now_text(piles),
            possible,
            tail: tail(true),
            usable,
        }
    }
}

fn carrying(piles: &[PileSnapshot], pick: &Pick) -> Guidance {
    let targets = pick
        .choices
        .iter()
        .filter(|choice| choice.target.is_some())
        .count();
    let mut possible = vec![
        part(Action::Select, "board.action.drop", 0),
        part(Action::Back, "board.action.put_back", 1),
        plain("board.action.choose_target", 2),
    ];
    let mut usable = vec![Action::Select, Action::Back];
    if let Some(best) = quick(&pick.choices) {
        possible.push(quick_part(best, 3));
        usable.push(Action::QuickMove);
    }
    usable.extend(ALWAYS);
    Guidance {
        now: now::carrying(piles, pick.source, pick.count, targets),
        possible,
        tail: tail(false),
        usable,
    }
}
