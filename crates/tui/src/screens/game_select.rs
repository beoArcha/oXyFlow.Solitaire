//! Game and variant selection. Enter or a click starts the focused variant; with a seed (from
//! "New game from seed") the variant is dealt from that seed. A toggle under the list switches the
//! undo mode of the next game (`u` or a click); the choice is remembered in the settings. Tab moves
//! the focus between the list and the toggle.

use ratatui::Frame;
use ratatui::crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use super::focus::{button_style, marker, step, tab_forward};
use super::{Effect, ViewCtx, variant_label_key};
use crate::action::Action;
use crate::app::Screen;
use crate::i18n::{t, tf};
use crate::input::primary_key;
use crate::mouse::{contains, left_press};
use crate::settings::Settings;
use crate::theme::Token;
use crate::widgets::footer::{self, FooterMap};
use crate::widgets::menu_list::MenuList;
use oxysol_core::Variant;

pub(crate) const FOOTER: [Action; 3] = [Action::Select, Action::Back, Action::Help];

/// The focus ring of the screen: the variants list, then the undo toggle.
const LIST: usize = 0;
const TOGGLE: usize = 1;
const RING_LEN: usize = 2;

/// State of the game selection screen.
#[derive(Debug, Default)]
pub struct GameSelect {
    list: MenuList,
    seed: Option<u64>,
    focus: usize,
    toggle: Rect,
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

    /// Handles a key press mapped to `action`: Enter starts the focused variant or presses the
    /// toggle, Tab moves the focus, `u` switches the undo mode, Esc goes back.
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
        if self.focus == TOGGLE {
            match action {
                Action::Select | Action::Left | Action::Right => return Self::toggle_undo(ctx),
                Action::Up | Action::Down => self.focus = LIST,
                _ => return Effect::None,
            }
        }
        self.list
            .on_action(action, Variant::ALL.len())
            .map_or(Effect::None, |index| self.start(index, ctx))
    }

    /// Handles a mouse event: the wheel moves focus, a click starts a variant or presses the undo
    /// toggle.
    pub fn handle_mouse(&mut self, event: &MouseEvent, ctx: &ViewCtx<'_>) -> Effect {
        if left_press(event).is_some_and(|cell| contains(self.toggle, cell)) {
            self.focus = TOGGLE;
            return Self::toggle_undo(ctx);
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

    fn draw_toggle(&mut self, frame: &mut Frame, areas: [Rect; 2], ctx: &ViewCtx<'_>) {
        let [toggle_area, hint_area] = areas;
        let state = if ctx.settings.undo_allowed {
            t("common.select.undo_on")
        } else {
            t("common.select.undo_off")
        };
        let focused = self.focus == TOGGLE;
        let label = tf("common.select.undo", &[("state", &state)]);
        let text = format!("{} {label}", marker(ctx, focused));
        let width = u16::try_from(text.chars().count()).unwrap_or(u16::MAX);
        let x = toggle_area.x + toggle_area.width.saturating_sub(width) / 2;
        self.toggle = Rect::new(x, toggle_area.y, width, 1).intersection(toggle_area);
        frame.render_widget(
            Paragraph::new(Line::styled(text, button_style(ctx, focused)))
                .alignment(Alignment::Center),
            toggle_area,
        );
        let key = primary_key(Action::Undo).unwrap_or_default();
        let hint = tf("common.select.undo_hint", &[("key", &key)]);
        let muted = ctx.theme.style(Token::TextMuted, Token::Background);
        frame.render_widget(
            Paragraph::new(Line::styled(hint, muted)).alignment(Alignment::Center),
            hint_area,
        );
    }

    /// Draws the title, the variants, the undo toggle and the key hints.
    pub fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
        let [
            _,
            title_area,
            _,
            list_area,
            _,
            toggle_area,
            hint_area,
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
        let labels: Vec<String> = Variant::ALL
            .iter()
            .map(|variant| t(variant_label_key(*variant)).into_owned())
            .collect();
        self.list
            .draw(frame, list_area, &labels, ctx, self.focus == LIST);
        self.draw_toggle(frame, [toggle_area, hint_area], ctx);
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

#[cfg(test)]
mod focus_tests;
#[cfg(test)]
mod tests;
