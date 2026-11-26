//! Game and variant selection. The games are grouped by family under headings, the favourites
//! first; a star marks the popular games. Enter or a click starts the focused variant; with a seed
//! (from "New game from seed") the variant is dealt from that seed. `f`, or a click on a variant's
//! marks, puts it in the favourites (kept in the settings). Enter, Space or a click on a heading,
//! and Left / Right on a heading or on a variant of it, fold and unfold the group (remembered in
//! the settings). `/` or a click on the search field opens a search that filters the list while the
//! player types and opens matching folded groups; Esc clears it first and a second Esc goes back.
//! A toggle under the list switches the undo mode of the next game (`u` or a click) and a second
//! one switches the search for winnable deals (`w` or a click). Tab moves the focus between the
//! list and the two toggles.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::layout::Rect;

use super::focus::{step, tab_forward};
use super::{Effect, ViewCtx, variant_label_key};
use crate::action::Action;
use crate::app::Screen;
use crate::i18n::{t, tf};
use crate::mouse::{contains, left_press};
use crate::settings::{GameGroup, Settings};
use crate::solver::slow_to_prove;
use crate::widgets::footer::FooterMap;
#[cfg(test)]
use crate::widgets::footer::KeyHint;
use oxysol_core::Variant;

mod catalog;
mod draw;
mod hints;
mod list;
mod rows;
mod search;
mod toggles;

use list::{ListView, MouseOutcome};
use rows::{FocusItem, Heading, Row, items};

#[cfg(test)]
pub(crate) const FOOTER: [KeyHint; 5] = [
    (Action::Select, "menu.footer.play"),
    (Action::Back, "help.action.back"),
    (Action::Help, "help.action.help"),
    (Action::Finish, "menu.footer.favorite"),
    (Action::Undo, "help.action.undo"),
];

/// The key of the winnable-deals toggle. It is not in the key table, so the screen reads it itself.
pub(crate) const WINNABLE_KEY: char = 'w';

/// The focus ring of the screen: the variants list, the undo toggle, the winnable-deals toggle.
const LIST: usize = 0;
const TOGGLE: usize = 1;
const WINNABLE: usize = 2;
const RING_LEN: usize = 3;
const RING_WITHOUT_WINNABLE: usize = 2;

/// Rows of the screen that are not the list: title, gap, search, legend, gap, hint, gap, two
/// toggles, two toggle hints and the footer.
const CHROME_ROWS: u16 = 12;

/// What Enter on a variant means: a game starts (`Play`), or the game is chosen before its seed
/// (`BeforeSeed`, the first step of "New game from seed"). The caller takes the variant from the
/// `Effect::Start` the list returns in that case.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Purpose {
    #[default]
    Play,
    BeforeSeed,
}

/// State of the game selection screen.
#[derive(Debug, Default)]
pub struct GameSelect {
    list: ListView,
    folded: Vec<GameGroup>,
    seed: Option<u64>,
    purpose: Purpose,
    focus: usize,
    search: Option<String>,
    search_area: Rect,
    toggle: Rect,
    winnable: Rect,
    footer: FooterMap,
}

#[cfg(feature = "dev")]
#[derive(Debug, Clone)]
pub(crate) struct DevGameSelect {
    pub searching: bool,
    pub focus_target: &'static str,
    pub focus_index: Option<usize>,
    pub variant: Option<Variant>,
    pub label: Option<String>,
}

/// The name of `variant` in the list. With winnable deals on, it also says when the search can
/// take long.
fn label_of(variant: Variant, winnable: bool) -> String {
    let name = t(variant_label_key(variant));
    if !winnable {
        name.into_owned()
    } else if slow_to_prove(variant) {
        tf("common.select.slow_search", &[("variant", &name)])
    } else {
        name.into_owned()
    }
}

impl GameSelect {
    /// A list focused on the first variant; the variants are dealt from fresh seeds.
    pub fn new() -> Self {
        Self::default()
    }

    /// A list whose variants are dealt from `seed`.
    pub fn for_seed(seed: u64) -> Self {
        Self {
            seed: Some(seed),
            ..Self::default()
        }
    }

    /// The list of "New game from seed": the game is chosen before its seed, and Enter names that
    /// next step.
    pub fn before_seed() -> Self {
        Self {
            purpose: Purpose::BeforeSeed,
            ..Self::default()
        }
    }

    /// Whether the winnable-deals toggle is offered. "New game from seed" does not offer it: the
    /// game it lists is dealt from the seed chosen next, as typed, so the toggle would only mislead.
    pub fn offers_winnable(&self) -> bool {
        self.purpose == Purpose::Play
    }

    fn ring_len(&self) -> usize {
        if self.offers_winnable() {
            RING_LEN
        } else {
            RING_WITHOUT_WINNABLE
        }
    }

    /// Whether typed characters are the search text: the field is open and the list has the focus.
    pub fn captures_text(&self) -> bool {
        self.search.is_some() && self.focus == LIST
    }

    /// Whether a search is open, so that Esc closes it before it leaves the screen.
    pub fn searching(&self) -> bool {
        self.search.is_some()
    }

    fn query(&self) -> &str {
        self.search.as_deref().unwrap_or("")
    }

    fn filtering(&self) -> bool {
        !self.query().trim().is_empty()
    }

    fn rows(&self, ctx: &ViewCtx<'_>) -> Vec<Row> {
        rows::build(
            &ctx.settings.favourites,
            &ctx.settings.folded_families,
            self.query(),
        )
    }

    fn start(&self, variant: Variant, ctx: &ViewCtx<'_>) -> Effect {
        Effect::Start {
            variant,
            seed: self.seed,
            undo_allowed: ctx.settings.undo_allowed,
        }
    }

    fn toggle_fold(&mut self, heading: Heading, ctx: &ViewCtx<'_>) -> Effect {
        let group = heading.group();
        let mut folded_families = ctx.settings.folded_families.clone();
        if let Some(pos) = folded_families.iter().position(|saved| *saved == group) {
            folded_families.remove(pos);
        } else {
            folded_families.push(group);
            self.list.set_focus_item(FocusItem::Heading(heading));
        }
        self.folded = folded_families.clone();
        let rows = rows::build(&ctx.settings.favourites, &folded_families, self.query());
        self.list
            .keep_resolved_in_view(&rows, &ctx.settings.favourites);
        Effect::Apply(Settings {
            folded_families,
            ..ctx.settings.clone()
        })
    }

    fn toggle_favourite(&mut self, variant: Variant, ctx: &ViewCtx<'_>) -> Effect {
        let mut favourites = ctx.settings.favourites.clone();
        match favourites.iter().position(|saved| *saved == variant) {
            Some(position) => {
                favourites.remove(position);
            }
            None => favourites.push(variant),
        }
        let rows = rows::build(&favourites, &ctx.settings.folded_families, self.query());
        if items(&rows).contains(&variant) {
            self.list.set_focus_item(FocusItem::Variant(variant));
        } else {
            self.list
                .set_focus_item(FocusItem::Heading(Heading::Favourites));
        }
        self.list.keep_resolved_in_view(&rows, &favourites);
        Effect::Apply(Settings {
            favourites,
            ..ctx.settings.clone()
        })
    }

    fn toggle_focused_favourite(&mut self, ctx: &ViewCtx<'_>) -> Effect {
        if self.focus != LIST {
            return Effect::None;
        }
        let rows = self.rows(ctx);
        match self.list.focused_item(&rows, &ctx.settings.favourites) {
            Some(FocusItem::Variant(variant)) => self.toggle_favourite(variant, ctx),
            _ => Effect::None,
        }
    }

    fn toggle_undo(ctx: &ViewCtx<'_>) -> Effect {
        Effect::Apply(Settings {
            undo_allowed: !ctx.settings.undo_allowed,
            ..ctx.settings.clone()
        })
    }

    fn toggle_winnable(ctx: &ViewCtx<'_>) -> Effect {
        Effect::Apply(Settings {
            winnable_only: !ctx.settings.winnable_only,
            ..ctx.settings.clone()
        })
    }

    fn press_toggle(&self, ctx: &ViewCtx<'_>) -> Effect {
        if self.focus == WINNABLE {
            Self::toggle_winnable(ctx)
        } else {
            Self::toggle_undo(ctx)
        }
    }

    fn open_search(&mut self) {
        self.search.get_or_insert_with(String::new);
        self.focus = LIST;
    }

    fn close_search(&mut self, ctx: &ViewCtx<'_>) {
        let open_rows = self.rows(ctx);
        let focused = self.list.focused_item(&open_rows, &ctx.settings.favourites);
        self.search = None;
        if let Some(item) = focused {
            self.list.set_focus_item(item);
        }
        let restored_rows = self.rows(ctx);
        self.list
            .keep_resolved_in_view(&restored_rows, &ctx.settings.favourites);
    }

    fn type_key(&mut self, key: &KeyEvent) -> bool {
        let Some(text) = self.search.as_mut() else {
            return false;
        };
        match key.code {
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => text.push(c),
            KeyCode::Backspace => {
                text.pop();
            }
            _ => return false,
        }
        self.list.reset();
        true
    }

    /// Handles a key press mapped to `action`. Esc closes the search, then goes back. While the
    /// search is open characters are typed into it; otherwise Enter starts the focused variant or
    /// folds the focused heading or presses the focused toggle, Tab moves the focus, `f` switches
    /// the favourite, `/` opens the search, `u` switches undo and Left/Right fold the group.
    pub fn handle(&mut self, key: &KeyEvent, action: Option<Action>, ctx: &ViewCtx<'_>) -> Effect {
        self.folded = ctx.settings.folded_families.clone();
        if action == Some(Action::Back) {
            if self.search.is_none() {
                return Effect::Goto(Screen::MainMenu);
            }
            self.close_search(ctx);
            return Effect::None;
        }
        if self.captures_text() && self.type_key(key) {
            return Effect::None;
        }
        if key.code == KeyCode::Char('/') && !self.captures_text() {
            self.open_search();
            return Effect::None;
        }
        if key.code == KeyCode::Char(WINNABLE_KEY) && !self.captures_text() {
            return if self.offers_winnable() {
                Self::toggle_winnable(ctx)
            } else {
                Effect::None
            };
        }
        if self.focus == LIST && self.handle_list_page_key(key.code, ctx) {
            return Effect::None;
        }
        match action {
            Some(Action::Undo) => Self::toggle_undo(ctx),
            Some(Action::Finish) => self.toggle_focused_favourite(ctx),
            Some(action) => self.handle_focused(action, ctx),
            None => Effect::None,
        }
    }

    fn handle_list_page_key(&mut self, code: KeyCode, ctx: &ViewCtx<'_>) -> bool {
        let rows = self.rows(ctx);
        match code {
            KeyCode::PageUp => self.list.page_by(true, &rows, &ctx.settings.favourites),
            KeyCode::PageDown => self.list.page_by(false, &rows, &ctx.settings.favourites),
            KeyCode::Home => self.list.home(&rows),
            KeyCode::End => self.list.end(&rows),
            _ => return false,
        }
        true
    }

    fn handle_focused(&mut self, action: Action, ctx: &ViewCtx<'_>) -> Effect {
        if let Some(forward) = tab_forward(action) {
            self.focus = step(self.focus, self.ring_len(), forward);
            return Effect::None;
        }
        let rows = self.rows(ctx);
        if self.focus != LIST {
            match action {
                Action::Select | Action::Left | Action::Right => return self.press_toggle(ctx),
                Action::Up => {
                    self.focus = LIST;
                    self.list.focus_last(&rows);
                }
                Action::Down => {
                    self.focus = LIST;
                    self.list.focus_first(&rows);
                }
                _ => {}
            }
            return Effect::None;
        }
        match action {
            Action::Up | Action::Down => {
                self.list
                    .step_action(action, &rows, &ctx.settings.favourites);
                Effect::None
            }
            Action::Left | Action::Right => self.fold_current(&rows, ctx),
            Action::Select => self.activate_current(&rows, ctx),
            _ => Effect::None,
        }
    }

    fn fold_current(&mut self, rows: &[Row], ctx: &ViewCtx<'_>) -> Effect {
        if self.filtering() {
            return Effect::None;
        }
        match self.list.focused_item(rows, &ctx.settings.favourites) {
            Some(FocusItem::Heading(heading)) => self.toggle_fold(heading, ctx),
            Some(FocusItem::Variant(variant)) => {
                let heading = Heading::of(variant, &ctx.settings.favourites);
                self.toggle_fold(heading, ctx)
            }
            None => Effect::None,
        }
    }

    fn activate_current(&mut self, rows: &[Row], ctx: &ViewCtx<'_>) -> Effect {
        match self.list.focused_item(rows, &ctx.settings.favourites) {
            Some(FocusItem::Variant(variant)) => self.start(variant, ctx),
            Some(FocusItem::Heading(heading)) if !self.filtering() => {
                self.toggle_fold(heading, ctx)
            }
            _ => Effect::None,
        }
    }

    /// Handles a mouse event: the wheel scrolls the list, clicks on edge marks scroll by a page,
    /// clicks on headings fold/unfold, and clicks on variants toggle favourites or start games.
    pub fn handle_mouse(&mut self, event: &MouseEvent, ctx: &ViewCtx<'_>) -> Effect {
        self.folded = ctx.settings.folded_families.clone();
        let pressed = left_press(event);
        if pressed.is_some_and(|cell| contains(self.toggle, cell)) {
            self.focus = TOGGLE;
            return Self::toggle_undo(ctx);
        }
        if self.offers_winnable() && pressed.is_some_and(|cell| contains(self.winnable, cell)) {
            self.focus = WINNABLE;
            return Self::toggle_winnable(ctx);
        }
        if pressed.is_some_and(|cell| contains(self.search_area, cell)) {
            self.open_search();
            return Effect::None;
        }
        let rows = self.rows(ctx);
        let Some(outcome) = self.list.on_mouse(event, &rows, &ctx.settings.favourites) else {
            return Effect::None;
        };
        self.focus = LIST;
        match outcome {
            MouseOutcome::Handled => Effect::None,
            MouseOutcome::Fold(heading) if !self.filtering() => self.toggle_fold(heading, ctx),
            MouseOutcome::Fold(_) => Effect::None,
            MouseOutcome::ToggleFavourite(variant) => self.toggle_favourite(variant, ctx),
            MouseOutcome::Start(variant) => self.start(variant, ctx),
        }
    }

    /// The footer button under `cell`, if any.
    pub fn footer_action_at(&self, cell: (u16, u16)) -> Option<Action> {
        self.footer.action_at(cell)
    }

    #[cfg(feature = "dev")]
    pub(crate) fn dev_state(&self, favourites: &[Variant]) -> DevGameSelect {
        let (focus_target, focus_index, variant, label) = match self.focus {
            TOGGLE => ("undo_toggle", None, None, None),
            WINNABLE => ("winnable_toggle", None, None, None),
            _ => {
                let rows = rows::build(favourites, &self.folded, self.query());
                let row_idx = self.list.focused_row(&rows, favourites);
                let focusable_idx = row_idx.map(|pos| {
                    rows[..=pos]
                        .iter()
                        .filter(|row| row.is_focusable())
                        .count()
                        .saturating_sub(1)
                });
                match self.list.focused_item(&rows, favourites) {
                    Some(FocusItem::Heading(heading)) => (
                        "heading",
                        focusable_idx,
                        None,
                        Some(t(heading.title_key()).into_owned()),
                    ),
                    Some(FocusItem::Variant(v)) => {
                        let var_idx = items(&rows).iter().position(|item| *item == v);
                        ("variant", var_idx, Some(v), None)
                    }
                    None => ("variant", None, None, None),
                }
            }
        };
        DevGameSelect {
            searching: self.searching(),
            focus_target,
            focus_index,
            variant,
            label,
        }
    }
}

#[cfg(test)]
mod focus_tests;
#[cfg(test)]
mod fold_scroll_tests;
#[cfg(test)]
mod menu_tests;
#[cfg(test)]
mod polish_tests;
#[cfg(test)]
mod search_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod winnable_tests;
