//! What the board tells the player about the moment: what is going on (`Now`), what can be done
//! (`Possible`, with the keys from the key table) and which actions are usable, for the guide
//! panel to highlight. Everything is read from the controller and from the legal moves the engine
//! lists; no rule is decided here.

use oxysol_core::{Move, PileRef, PileSnapshot, Session, Variant};

use super::choices::{Choice, quick};
use super::cursor::max_depth;
use super::priority;
use super::{BoardScreen, Pick, now};
use crate::action::Action;
use crate::i18n::{t, tf};
use crate::input::primary_key;
use crate::widgets::footer::Segment;
use crate::widgets::text::clip;

/// One thing the player can do, as a short text with its key; the part with the highest
/// `drop_order` goes first when the line is too wide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Part {
    pub text: String,
    pub drop_order: u8,
    /// The action this part runs when clicked; `None` for display-only parts like `Arrows: move`.
    pub action: Option<Action>,
}

/// The moment on the board: what is in progress, what can be done, and which actions work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Guidance {
    pub now: String,
    /// Shorter wordings of `now`, longest first, for when the row is narrow.
    pub shorter: Vec<String>,
    pub possible: Vec<Part>,
    /// Parts after the possible ones: restart, options, the key list and the way out. They are
    /// dropped by `drop_order` like the others.
    pub tail: Vec<Part>,
    pub usable: Vec<Action>,
}

const ALWAYS: [Action; 14] = [
    Action::Up,
    Action::Down,
    Action::Left,
    Action::Right,
    Action::Help,
    Action::Rules,
    Action::Guide,
    Action::Moves,
    Action::Options,
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
        action: Some(action),
    }
}

fn plain(key: &str, drop_order: u8) -> Part {
    Part {
        text: t(key).into_owned(),
        drop_order,
        action: None,
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
        action: None,
    }
}

impl Guidance {
    /// The `Now:` row text for `width` columns: the first wording that fits, else the shortest one
    /// cut to `width`.
    pub(super) fn now_line(&self, width: usize) -> String {
        let lines: Vec<String> = std::iter::once(&self.now)
            .chain(&self.shorter)
            .map(|text| tf("board.action.now", &[("text", text)]))
            .collect();
        let fitting = lines.iter().find(|line| line.chars().count() <= width);
        match (fitting, lines.last()) {
            (Some(line), _) | (None, Some(line)) => clip(line, width),
            (None, None) => String::new(),
        }
    }

    /// The `Do:` row as segments: the prefix, the parts that fit `width` columns (least useful
    /// dropped first, the tail always kept) with their separators, and the suffix. Parts with an
    /// action are buttons.
    pub(super) fn do_segments(&self, width: usize) -> Vec<Segment> {
        let template = t("board.action.possible");
        let (before, after) = template.split_once("{text}").unwrap_or(("", ""));
        let separator = t("board.action.separator");
        let mut segments = vec![Segment::plain(before)];
        for (index, part) in self.visible_parts(width).iter().enumerate() {
            if index > 0 {
                segments.push(Segment::plain(separator.as_ref()));
            }
            segments.push(match part.action {
                Some(action) => Segment::button(action, part.text.as_str()),
                None => Segment::plain(part.text.as_str()),
            });
        }
        segments.push(Segment::plain(after));
        segments
    }

    fn visible_parts(&self, width: usize) -> Vec<Part> {
        let mut parts = self.possible.clone();
        parts.extend(self.tail.iter().cloned());
        while !self.do_row_fits(&parts, width) {
            let Some(index) = parts
                .iter()
                .enumerate()
                .max_by_key(|(_, part)| part.drop_order)
                .map(|(index, _)| index)
            else {
                break;
            };
            parts.remove(index);
        }
        parts
    }

    fn do_row_fits(&self, parts: &[Part], width: usize) -> bool {
        tf("board.action.possible", &[("text", &join(parts))])
            .chars()
            .count()
            <= width
    }

    fn closed(now_key: &str, possible: Vec<Part>, mut usable: Vec<Action>) -> Self {
        usable.extend(ALWAYS);
        Self {
            now: t(now_key).into_owned(),
            shorter: Vec::new(),
            possible,
            tail: tail(true),
            usable,
        }
    }
}

/// The parts that end every `Do:` row: restart, options, pause and moves, the key list, and unless
/// Esc is taken by a pick, the way to the menu, which is kept longest of the tail.
fn tail(with_menu: bool) -> Vec<Part> {
    let mut parts = vec![
        part(Action::Restart, "board.action.restart", priority::RESTART),
        part(Action::Options, "board.action.options", priority::OPTIONS),
        part(Action::Pause, "board.action.pause", priority::PAUSE),
        part(Action::Moves, "board.action.moves", priority::MOVES),
        part(Action::Help, "board.action.help", priority::HELP),
    ];
    if with_menu {
        parts.push(part(Action::Back, "board.action.menu", priority::MENU));
    }
    parts
}

fn won() -> Guidance {
    Guidance::closed(
        "board.action.won",
        vec![part(
            Action::NewGame,
            "board.action.new_game",
            priority::PRIMARY,
        )],
        Vec::new(),
    )
}

fn stuck(session: &Session, undo_allowed: bool) -> Guidance {
    let mut possible = Vec::new();
    let mut usable = Vec::new();
    if undo_allowed && session.can_undo() {
        possible.push(part(Action::Undo, "board.action.undo", priority::UNDO));
        usable.push(Action::Undo);
    }
    possible.push(part(
        Action::NewGame,
        "board.action.new_game",
        priority::PRIMARY,
    ));
    Guidance::closed("board.action.stuck", possible, usable)
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
        if let Some(best) = best {
            possible.push(quick_part(best, priority::QUICK));
            usable.push(Action::QuickMove);
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

fn carrying(piles: &[PileSnapshot], pick: &Pick) -> Guidance {
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
    if let Some(best) = quick(&pick.choices) {
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
