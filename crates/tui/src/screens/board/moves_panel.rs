//! The possible moves panel of the board: the list of entries read from the same guidance as the
//! `Do:` row (so both always agree), its place in the Tab order, and the window that replaces it
//! on a Compact screen. The legal moves come from the engine; nothing is decided here.

use oxysol_core::{PileRef, Session, Variant};

use super::BoardScreen;
use super::choices::quick;
use super::cursor::{Cursor, cycle_edge, end_pile};
use super::status;
use crate::action::Action;
use crate::app::Modal;
use crate::audio::SoundCue;
use crate::i18n::t;
use crate::input::primary_key;
use crate::layout::Class;
use crate::screens::focus::step;
use crate::screens::{Effect, ViewCtx};
use crate::settings::Aid;
use crate::widgets::moves::MoveEntry;
use crate::widgets::overlay::MovesWindow;

/// The request to show the moves panel when it is hidden and to hide it when it is shown.
pub(super) fn toggle_moves(ctx: &ViewCtx<'_>) -> Effect {
    Effect::Quick(Aid::MovesPanel.toggled(&ctx.settings.aids()))
}

const BOARD_ACTIONS: [Action; 10] = [
    Action::Stock,
    Action::Select,
    Action::QuickMove,
    Action::Back,
    Action::Hint,
    Action::Undo,
    Action::Redo,
    Action::Finish,
    Action::Restart,
    Action::NewGame,
];

const fn reason_key(name: Reason) -> &'static str {
    match name {
        Reason::Won => "board.moves.reason.won",
        Reason::Carrying => "board.moves.reason.carrying",
        Reason::NoStock => "board.moves.reason.no_stock",
        Reason::NoPasses => "board.moves.reason.no_passes",
        Reason::NoDeals => "board.moves.reason.no_deals",
        Reason::Blocked => "board.moves.reason.blocked",
        Reason::NothingHere => "board.moves.reason.nothing_here",
        Reason::NoQuick => "board.moves.reason.no_quick",
        Reason::NotCarrying => "board.moves.reason.not_carrying",
        Reason::NoHint => "board.moves.reason.no_hint",
        Reason::NoUndo => "board.moves.reason.no_undo",
        Reason::NoRedo => "board.moves.reason.no_redo",
        Reason::UndoOff => "board.moves.reason.undo_off",
        Reason::NoFinish => "board.moves.reason.no_finish",
    }
}

#[derive(Debug, Clone, Copy)]
enum Reason {
    Won,
    Carrying,
    NoStock,
    NoPasses,
    NoDeals,
    Blocked,
    NothingHere,
    NoQuick,
    NotCarrying,
    NoHint,
    NoUndo,
    NoRedo,
    UndoOff,
    NoFinish,
}

fn stock_len(session: &Session) -> usize {
    session
        .game()
        .piles()
        .iter()
        .find(|snapshot| snapshot.pile == PileRef::Stock)
        .map_or(0, |snapshot| snapshot.cards.len())
}

fn stock_label(session: &Session) -> &'static str {
    match session.variant() {
        Variant::Spider(_) => "board.moves.label.deal",
        _ if stock_len(session) == 0 => "board.moves.label.recycle",
        _ => "board.moves.label.draw",
    }
}

impl BoardScreen {
    /// Whether the side panel is drawn: the setting is on and the screen has room for it.
    pub(super) fn moves_shown(&self) -> bool {
        self.moves_panel && Class::of(self.screen) != Class::Compact
    }

    /// Every board action as an entry, active when the guidance lists it as usable. Empty without
    /// a game and while the board is covered.
    pub fn move_entries(&self) -> Vec<MoveEntry> {
        let (Some(session), Some(guidance)) = (self.session.as_ref(), self.guidance()) else {
            return Vec::new();
        };
        BOARD_ACTIONS
            .into_iter()
            .map(|action| {
                let active = guidance.usable.contains(&action);
                let label = self.entry_label(session, action);
                MoveEntry {
                    action,
                    label: t(label).into_owned(),
                    key: primary_key(action).unwrap_or_default(),
                    active,
                    reason: (!active)
                        .then(|| t(reason_key(self.reason_of(session, action))).into_owned()),
                }
            })
            .collect()
    }

    fn entry_label(&self, session: &Session, action: Action) -> &'static str {
        match action {
            Action::Stock => stock_label(session),
            Action::Select if self.pick.is_some() => "board.moves.label.drop",
            Action::Select if self.cursor.pile == PileRef::Stock => stock_label(session),
            Action::Select => "board.moves.label.pick_up",
            Action::QuickMove if self.quick_to_foundation() => "board.moves.label.foundation",
            Action::QuickMove => "board.moves.label.quick",
            Action::Back => "board.moves.label.put_back",
            Action::Hint => "board.moves.label.hint",
            Action::Undo => "board.moves.label.undo",
            Action::Redo => "board.moves.label.redo",
            Action::Finish => "board.moves.label.finish",
            Action::Restart => "board.moves.label.restart",
            _ => "board.moves.label.new_game",
        }
    }

    fn quick_to_foundation(&self) -> bool {
        let best = match &self.pick {
            Some(pick) => quick(&pick.choices),
            None if self.cursor.pile == PileRef::Stock => None,
            None => self
                .candidates(self.cursor.pile, self.cursor.depth)
                .ok()
                .and_then(|(_, found)| quick(&found)),
        };
        best.is_some_and(|choice| matches!(choice.target, Some(PileRef::Foundation(_))))
    }

    fn reason_of(&self, session: &Session, action: Action) -> Reason {
        if session.game().is_won() {
            return Reason::Won;
        }
        let carrying = self.pick.is_some();
        match action {
            Action::Stock if carrying => Reason::Carrying,
            Action::Stock => stock_reason(session),
            Action::Select => Reason::NothingHere,
            Action::QuickMove => Reason::NoQuick,
            Action::Back => Reason::NotCarrying,
            Action::Hint if carrying => Reason::Carrying,
            Action::Hint => Reason::NoHint,
            Action::Undo | Action::Redo if !self.undo_allowed => Reason::UndoOff,
            Action::Undo | Action::Redo if carrying => Reason::Carrying,
            Action::Undo => Reason::NoUndo,
            Action::Redo => Reason::NoRedo,
            Action::Finish => Reason::NoFinish,
            _ => Reason::Blocked,
        }
    }

    /// The window of the entries, for a screen too narrow for the panel.
    pub(super) fn open_moves_window(&self) -> Effect {
        Effect::Open(Modal::Moves(MovesWindow::new(self.move_entries())))
    }

    /// Takes the focus away from the panel, back to the board.
    pub(super) fn leave_panel(&mut self) {
        self.panel_focus = None;
    }

    /// Tab and Shift+Tab: the piles in reading order, then the entries of the panel, then round
    /// again. Without the panel on screen the ring is the piles only.
    pub(super) fn cycle_focus(&mut self, forward: bool) {
        let entries = self.move_entries().len();
        if let Some(index) = self.panel_focus.filter(|_| self.moves_shown()) {
            let next = step(index, entries, forward);
            let wrapped = if forward {
                next <= index
            } else {
                next >= index
            };
            if wrapped {
                self.leave_panel();
                self.enter_piles(forward);
            } else {
                self.panel_focus = Some(next);
            }
            return;
        }
        self.panel_focus = None;
        let piles = self.piles();
        let Some((pile, wrapped)) = cycle_edge(&piles, self.cursor.pile, forward) else {
            return;
        };
        if wrapped && self.moves_shown() && entries > 0 {
            self.panel_focus = Some(if forward { 0 } else { entries - 1 });
        } else {
            self.cursor = Cursor { pile, depth: 1 };
        }
    }

    fn enter_piles(&mut self, forward: bool) {
        let piles = self.piles();
        if let Some(pile) = end_pile(&piles, !forward) {
            self.cursor = Cursor { pile, depth: 1 };
        }
    }

    /// A key while the panel has the focus: Up and Down choose an entry, Enter and Space run it,
    /// Esc and Left and Right return to the board. `None` for any other action, which the board
    /// handles as usual.
    pub(super) fn panel_key(&mut self, action: Action) -> Option<Effect> {
        let index = self.panel_focus.filter(|_| self.moves_shown())?;
        let entries = self.move_entries();
        match action {
            Action::Up => {
                self.panel_focus = Some(index.saturating_sub(1));
                Some(Effect::None)
            }
            Action::Down => {
                self.panel_focus = Some((index + 1).min(entries.len().saturating_sub(1)));
                Some(Effect::None)
            }
            Action::Select => Some(self.run_entry(entries.get(index))),
            Action::Back | Action::Left | Action::Right => {
                self.leave_panel();
                Some(Effect::None)
            }
            _ => None,
        }
    }

    /// Runs an entry as its key would. An inactive entry does nothing but say so.
    pub(super) fn run_entry(&mut self, entry: Option<&MoveEntry>) -> Effect {
        let Some(entry) = entry else {
            return Effect::None;
        };
        if !entry.active {
            self.status = Some(status::NOT_POSSIBLE);
            self.cue(SoundCue::Illegal);
            return Effect::None;
        }
        if matches!(entry.action, Action::Select | Action::Back) {
            self.leave_panel();
        }
        self.dispatch(entry.action)
    }

    /// A click on the panel: the active entry under `cell` runs, any other cell does nothing.
    pub(super) fn press_panel(&mut self, cell: (u16, u16)) -> Option<Effect> {
        let action = self.moves_map.action_at(cell)?;
        self.leave_panel();
        Some(self.dispatch(action))
    }

    /// Whether a click at `cell` lands on the panel's title row.
    pub(super) fn on_moves_title(&self, cell: (u16, u16)) -> bool {
        self.chrome()
            .moves
            .map(crate::widgets::moves::title_row)
            .is_some_and(|row| crate::mouse::contains(row, cell))
    }
}

fn stock_reason(session: &Session) -> Reason {
    let Some(stock) = session.game().stock_status() else {
        return Reason::Blocked;
    };
    if stock.deals_left == Some(0) {
        return Reason::NoDeals;
    }
    match (stock_len(session), stock.recycles_left) {
        (0, Some(0)) => Reason::NoPasses,
        (0, _) => Reason::NoStock,
        _ => Reason::Blocked,
    }
}
