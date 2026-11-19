//! Game and variant selection. Enter or a click starts the focused variant; with a seed (from
//! "New game from seed") the variant is dealt from that seed. A toggle under the list switches the
//! undo mode of the next game (`u` or a click) and a second one switches the search for winnable
//! deals; both choices are remembered in the settings. Tab moves the focus between the list and
//! the two toggles.

use ratatui::Frame;
use ratatui::crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use super::focus::{step, tab_forward};
use super::{Effect, ViewCtx, variant_label_key};
use crate::action::Action;
use crate::app::Screen;
use crate::i18n::{t, tf};
use crate::mouse::{contains, left_press};
use crate::settings::Settings;
use crate::solver::slow_to_prove;
use crate::theme::Token;
use crate::widgets::footer::{self, FooterMap};
use crate::widgets::menu_list::MenuList;
use oxysol_core::{DrawCount, PyramidRules, SuitCount, Variant};

pub(crate) const FOOTER: [Action; 3] = [Action::Select, Action::Back, Action::Help];

fn difficulty_key(variant: Variant) -> &'static str {
    match variant {
        Variant::Klondike(DrawCount::One)
        | Variant::Spider(SuitCount::One)
        | Variant::Pyramid(PyramidRules::Relaxed) => "menu.difficulty.easy",
        Variant::Klondike(DrawCount::Three)
        | Variant::Spider(SuitCount::Two)
        | Variant::FreeCell => "menu.difficulty.medium",
        Variant::Spider(SuitCount::Four) | Variant::Pyramid(PyramidRules::Classic) => {
            "menu.difficulty.hard"
        }
    }
}

const fn game_hint_key(variant: Variant) -> &'static str {
    match variant {
        Variant::Klondike(_) => "menu.game_hint.klondike",
        Variant::Spider(_) => "menu.game_hint.spider",
        Variant::FreeCell => "menu.game_hint.freecell",
        Variant::Pyramid(_) => "menu.game_hint.pyramid",
    }
}

/// The one-line hint under the list: the difficulty word and what the game asks for.
fn hint_text(variant: Variant) -> String {
    let level = t(difficulty_key(variant)).into_owned();
    let description = t(game_hint_key(variant)).into_owned();
    tf(
        "menu.variant_hint",
        &[
            ("level", level.as_str()),
            ("description", description.as_str()),
        ],
    )
}

/// The focus ring of the screen: the variants list, the undo toggle, the winnable-deals toggle.
const LIST: usize = 0;
const TOGGLE: usize = 1;
const WINNABLE: usize = 2;
const RING_LEN: usize = 3;

/// State of the game selection screen.
#[derive(Debug, Default)]
pub struct GameSelect {
    list: MenuList,
    seed: Option<u64>,
    focus: usize,
    toggle: Rect,
    winnable: Rect,
    footer: FooterMap,
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

    fn start(&self, index: usize, ctx: &ViewCtx<'_>) -> Effect {
        Variant::ALL
            .get(index)
            .map_or(Effect::None, |variant| Effect::Start {
                variant: *variant,
                seed: self.seed,
                undo_allowed: ctx.settings.undo_allowed,
            })
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

    /// Handles a key press mapped to `action`: Enter starts the focused variant or presses the
    /// focused toggle, Tab moves the focus, `u` switches the undo mode, Esc goes back.
    pub fn handle(&mut self, _key: &KeyEvent, action: Option<Action>, ctx: &ViewCtx<'_>) -> Effect {
        match action {
            Some(Action::Back) => Effect::Goto(Screen::MainMenu),
            Some(Action::Undo) => Self::toggle_undo(ctx),
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
        self.list
            .on_action(action, Variant::ALL.len())
            .map_or(Effect::None, |index| self.start(index, ctx))
    }

    /// Handles a mouse event: the wheel moves focus, a click starts a variant or presses a toggle.
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
        self.focus = LIST;
        self.list
            .on_mouse(event, Variant::ALL.len())
            .map_or(Effect::None, |index| self.start(index, ctx))
    }

    /// The footer button under `cell`, if any.
    pub fn footer_action_at(&self, cell: (u16, u16)) -> Option<Action> {
        self.footer.action_at(cell)
    }

    fn title(&self) -> String {
        match self.seed {
            Some(seed) => tf("menu.select_seed", &[("seed", &seed.to_string())]),
            None => t("common.select.title").into_owned(),
        }
    }

    fn hint_line(&self) -> String {
        Variant::ALL
            .get(self.list.focus())
            .map_or_else(String::new, |variant| hint_text(*variant))
    }

    fn variant_labels(ctx: &ViewCtx<'_>) -> Vec<String> {
        Variant::ALL
            .iter()
            .map(|variant| {
                let name = t(variant_label_key(*variant));
                if ctx.settings.winnable_only && slow_to_prove(*variant) {
                    tf("common.select.slow_search", &[("variant", &name)])
                } else {
                    name.into_owned()
                }
            })
            .collect()
    }

    /// Draws the title, the variants, the two toggles and the key hints.
    pub fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
        let [
            _,
            title_area,
            _,
            list_area,
            hint_area,
            _,
            undo_area,
            winnable_area,
            undo_hint_area,
            winnable_hint_area,
            _,
            footer_area,
        ] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(u16::try_from(Variant::ALL.len()).unwrap_or(0)),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(2),
            Constraint::Length(1),
        ])
        .areas(area);
        let title = Line::styled(
            self.title(),
            ctx.theme.style(Token::Accent, Token::Background),
        );
        frame.render_widget(
            Paragraph::new(title).alignment(Alignment::Center),
            title_area,
        );
        self.list.draw(
            frame,
            list_area,
            &Self::variant_labels(ctx),
            ctx,
            self.focus == LIST,
        );
        let hint = Line::styled(
            self.hint_line(),
            ctx.theme.style(Token::TextMuted, Token::Background),
        );
        frame.render_widget(Paragraph::new(hint).alignment(Alignment::Center), hint_area);
        self.toggle = toggles::draw_undo(frame, undo_area, self.focus == TOGGLE, ctx);
        self.winnable = toggles::draw_winnable(frame, winnable_area, self.focus == WINNABLE, ctx);
        toggles::draw_hints(frame, [undo_hint_area, winnable_hint_area], ctx);
        self.footer = footer::draw(
            frame,
            footer_area,
            &footer::key_segments(&FOOTER),
            Alignment::Center,
            footer::key_style(ctx),
            ctx,
        );
    }
}

mod toggles;

#[cfg(test)]
mod focus_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod winnable_tests;
