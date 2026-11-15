//! The settings screen. Stub: P3-05 adds the options; until then it only shows a notice and
//! leads back.

use ratatui::Frame;
use ratatui::crossterm::event::KeyEvent;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use super::{Effect, ViewCtx};
use crate::action::Action;
use crate::app::Screen;
use crate::i18n::t;
use crate::theme::Token;
use crate::widgets::footer;

const FOOTER: [Action; 2] = [Action::Back, Action::Help];

/// State of the settings screen.
#[derive(Debug, Default)]
pub struct SettingsScreen;

impl SettingsScreen {
    /// A settings screen.
    pub fn new() -> Self {
        Self
    }

    /// Handles a key press mapped to `action`; Esc leads back to the main menu.
    pub fn handle(
        &mut self,
        _key: &KeyEvent,
        action: Option<Action>,
        _ctx: &ViewCtx<'_>,
    ) -> Effect {
        match action {
            Some(Action::Back) => Effect::Goto(Screen::MainMenu),
            _ => Effect::None,
        }
    }

    /// Draws the title, the notice and the key hints.
    pub fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
        let [_, title_area, notice_area, _, footer_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(2),
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
        let notice = Line::raw(t("common.settings.placeholder").into_owned());
        frame.render_widget(
            Paragraph::new(notice).alignment(Alignment::Center),
            notice_area,
        );
        footer::draw(frame, footer_area, &FOOTER, ctx);
    }
}
