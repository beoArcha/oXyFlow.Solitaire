//! The settings screen: one row per option. Keys and mouse change the focused row; the result
//! leaves as [`Effect::Apply`] and `App` applies and saves it.

mod options;
#[cfg(test)]
mod tests;

use ratatui::Frame;
use ratatui::crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use self::options::{FIELDS, Step, changed, line, unavailable_reason_key};
use super::{Effect, ViewCtx};
use crate::action::Action;
use crate::app::Screen;
use crate::i18n::t;
use crate::theme::Token;
use crate::widgets::footer;
use crate::widgets::menu_list::MenuList;

const FOOTER: [Action; 3] = [Action::Select, Action::Back, Action::Help];

/// State of the settings screen: the focused row.
#[derive(Debug, Default)]
pub struct SettingsScreen {
    list: MenuList,
}

fn change_row(index: usize, step: Step, ctx: &ViewCtx<'_>) -> Effect {
    let Some(field) = FIELDS.get(index).copied() else {
        return Effect::None;
    };
    if field.is_audio() && !ctx.audio_status.is_ready() {
        return Effect::None;
    }
    Effect::Apply(changed(field, ctx.settings, step, ctx.sound_styles))
}

fn draw_reason(frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
    let Some(key) = unavailable_reason_key(ctx.audio_status) else {
        return;
    };
    let style = ctx.theme.style(Token::TextMuted, Token::Background);
    let text = Line::styled(t(key).into_owned(), style);
    frame.render_widget(Paragraph::new(text).alignment(Alignment::Center), area);
}

impl SettingsScreen {
    /// A settings screen focused on its first row.
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles a key press mapped to `action`. Left and right change the focused row, Enter
    /// changes it forward, Esc leads back to the main menu.
    pub fn handle(&mut self, _key: &KeyEvent, action: Option<Action>, ctx: &ViewCtx<'_>) -> Effect {
        match action {
            Some(Action::Back) => Effect::Goto(Screen::MainMenu),
            Some(Action::Left) => change_row(self.list.focus(), Step::Previous, ctx),
            Some(Action::Right) => change_row(self.list.focus(), Step::Next, ctx),
            Some(action) => self
                .list
                .on_action(action, FIELDS.len())
                .map_or(Effect::None, |index| change_row(index, Step::Next, ctx)),
            None => Effect::None,
        }
    }

    /// Handles a mouse event: the wheel moves focus, a click changes the clicked row forward.
    pub fn handle_mouse(&mut self, event: &MouseEvent, ctx: &ViewCtx<'_>) -> Effect {
        self.list
            .on_mouse(event, FIELDS.len())
            .map_or(Effect::None, |index| change_row(index, Step::Next, ctx))
    }

    /// Draws the title, the rows, the reason why sound is unavailable and the key hints.
    pub fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
        let [_, title_area, _, list_area, reason_area, _, footer_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(u16::try_from(FIELDS.len()).unwrap_or(0)),
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);
        let title = Line::styled(
            t("common.settings.title").into_owned(),
            ctx.theme.style(Token::Accent, Token::Background),
        );
        frame.render_widget(
            Paragraph::new(title).alignment(Alignment::Center),
            title_area,
        );
        let labels: Vec<String> = FIELDS
            .iter()
            .map(|field| {
                line(
                    *field,
                    ctx.settings,
                    ctx.audio_status.is_ready(),
                    ctx.sound_styles,
                )
            })
            .collect();
        self.list.draw(frame, list_area, &labels, ctx);
        draw_reason(frame, reason_area, ctx);
        footer::draw(frame, footer_area, &FOOTER, ctx);
    }
}
