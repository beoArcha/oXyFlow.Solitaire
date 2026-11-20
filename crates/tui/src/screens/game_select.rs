//! Game and variant selection. The games are grouped by family under headings, the favourites
//! first; a star marks the popular games. Enter or a click starts the focused variant; with a seed
//! (from "New game from seed") the variant is dealt from that seed. `f`, or a click on a variant's
//! marks, puts it in the favourites (kept in the settings). `/` or a click on the search field
//! opens a search that filters the list while the player types; Esc clears it first and a second
//! Esc goes back. A toggle under the list switches the undo mode of the next game (`u` or a click)
//! and a second one switches the search for winnable deals; both choices are remembered in the
//! settings. Tab moves the focus between the list and the two toggles.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::layout::Rect;

use super::focus::{step, tab_forward};
use super::{Effect, ViewCtx, variant_label_key};
use crate::action::Action;
use crate::app::Screen;
use crate::i18n::{t, tf};
use crate::mouse::{contains, left_press};
use crate::settings::Settings;
use crate::solver::slow_to_prove;
use crate::widgets::footer::{FooterMap, KeyHint};
use oxysol_core::Variant;

mod catalog;
mod draw;
mod hints;
mod list;
mod rows;
mod search;
mod toggles;

use list::ListView;
use rows::items;

pub(crate) const FOOTER: [KeyHint; 3] = [
    (Action::Select, "menu.footer.play"),
    (Action::Back, "help.action.back"),
    (Action::Help, "help.action.help"),
];

/// The focus ring of the screen: the variants list, the undo toggle, the winnable-deals toggle.
const LIST: usize = 0;
const TOGGLE: usize = 1;
const WINNABLE: usize = 2;
const RING_LEN: usize = 3;

/// Rows of the screen that are not the list: title, gap, search, legend, hint, gap, two toggles,
/// two toggle hints and the footer.
const CHROME_ROWS: u16 = 11;

/// State of the game selection screen.
#[derive(Debug, Default)]
pub struct GameSelect {
    list: ListView,
    seed: Option<u64>,
    focus: usize,
    search: Option<String>,
    search_area: Rect,
    toggle: Rect,
    winnable: Rect,
    footer: FooterMap,
}

fn label_of(variant: Variant, ctx: &ViewCtx<'_>) -> String {
    let name = t(variant_label_key(variant));
    if ctx.settings.winnable_only && slow_to_prove(variant) {
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

    fn variants(&self, ctx: &ViewCtx<'_>) -> Vec<Variant> {
        items(&rows::build(&ctx.settings.favourites, self.query()))
    }

    fn start(&self, variant: Variant, ctx: &ViewCtx<'_>) -> Effect {
        Effect::Start {
            variant,
            seed: self.seed,
            undo_allowed: ctx.settings.undo_allowed,
        }
    }

    fn start_at(&self, index: usize, ctx: &ViewCtx<'_>) -> Effect {
        self.variants(ctx)
            .get(index)
            .map_or(Effect::None, |variant| self.start(*variant, ctx))
    }

    fn toggle_favourite(&mut self, variant: Variant, ctx: &ViewCtx<'_>) -> Effect {
        let mut favourites = ctx.settings.favourites.clone();
        match favourites.iter().position(|saved| *saved == variant) {
            Some(position) => {
                favourites.remove(position);
            }
            None => favourites.push(variant),
        }
        let rows = rows::build(&favourites, self.query());
        if let Some(index) = items(&rows).iter().position(|shown| *shown == variant) {
            self.list.set_focus(index);
        }
        Effect::Apply(Settings {
            favourites,
            ..ctx.settings.clone()
        })
    }

    fn toggle_focused_favourite(&mut self, ctx: &ViewCtx<'_>) -> Effect {
        match self.variants(ctx).get(self.list.focus()).copied() {
            Some(variant) if self.focus == LIST => self.toggle_favourite(variant, ctx),
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
        let focused = self.variants(ctx).get(self.list.focus()).copied();
        self.search = None;
        if let Some(index) = focused.and_then(|variant| {
            self.variants(ctx)
                .iter()
                .position(|shown| *shown == variant)
        }) {
            self.list.set_focus(index);
        }
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
    /// presses the focused toggle, Tab moves the focus, `f` switches the favourite, `/` opens the
    /// search and `u` switches the undo mode.
    pub fn handle(&mut self, key: &KeyEvent, action: Option<Action>, ctx: &ViewCtx<'_>) -> Effect {
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
        match action {
            Some(Action::Undo) => Self::toggle_undo(ctx),
            Some(Action::Finish) => self.toggle_focused_favourite(ctx),
            Some(action) => self.handle_focused(action, ctx),
            None => Effect::None,
        }
    }

    fn handle_focused(&mut self, action: Action, ctx: &ViewCtx<'_>) -> Effect {
        if let Some(forward) = tab_forward(action) {
            self.focus = step(self.focus, RING_LEN, forward);
            return Effect::None;
        }
        if self.focus != LIST {
            match action {
                Action::Select | Action::Left | Action::Right => return self.press_toggle(ctx),
                Action::Up | Action::Down => self.focus = LIST,
                _ => return Effect::None,
            }
        }
        let count = self.variants(ctx).len();
        self.list
            .on_action(action, count)
            .map_or(Effect::None, |index| self.start_at(index, ctx))
    }

    /// Handles a mouse event: the wheel moves focus, a click starts a variant, toggles its
    /// favourite mark, opens the search or presses a toggle.
    pub fn handle_mouse(&mut self, event: &MouseEvent, ctx: &ViewCtx<'_>) -> Effect {
        let pressed = left_press(event);
        if pressed.is_some_and(|cell| contains(self.toggle, cell)) {
            self.focus = TOGGLE;
            return Self::toggle_undo(ctx);
        }
        if pressed.is_some_and(|cell| contains(self.winnable, cell)) {
            self.focus = WINNABLE;
            return Self::toggle_winnable(ctx);
        }
        if pressed.is_some_and(|cell| contains(self.search_area, cell)) {
            self.open_search();
            return Effect::None;
        }
        self.focus = LIST;
        let count = self.variants(ctx).len();
        let Some(click) = self.list.on_mouse(event, count) else {
            return Effect::None;
        };
        match self.variants(ctx).get(click.index).copied() {
            Some(variant) if click.on_mark => self.toggle_favourite(variant, ctx),
            Some(variant) => self.start(variant, ctx),
            None => Effect::None,
        }
    }

    /// The footer button under `cell`, if any.
    pub fn footer_action_at(&self, cell: (u16, u16)) -> Option<Action> {
        self.footer.action_at(cell)
    }
}

#[cfg(test)]
mod focus_tests;
#[cfg(test)]
mod menu_tests;
#[cfg(test)]
mod search_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod winnable_tests;
