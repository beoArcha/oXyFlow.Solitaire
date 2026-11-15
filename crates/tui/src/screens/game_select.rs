//! Game and variant selection. Stub: a flat list of every variant; P3-04 groups it by game and
//! adds the typed-seed flow.

use ratatui::Frame;
use ratatui::crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use super::{Effect, ViewCtx, variant_label_key};
use crate::action::Action;
use crate::app::Screen;
use crate::i18n::t;
use crate::theme::Token;
use crate::widgets::footer;
use crate::widgets::menu_list::MenuList;
use oxysol_core::Variant;

const FOOTER: [Action; 3] = [Action::Select, Action::Back, Action::Help];

fn start(index: usize) -> Effect {
    Variant::ALL
        .get(index)
        .map_or(Effect::None, |variant| Effect::Start {
            variant: *variant,
            seed: None,
        })
}

/// State of the game selection screen.
#[derive(Debug, Default)]
pub struct GameSelect {
    list: MenuList,
}

impl GameSelect {
    /// A list focused on the first variant.
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles a key press mapped to `action`: Enter starts the focused variant, Esc goes back.
    pub fn handle(
        &mut self,
        _key: &KeyEvent,
        action: Option<Action>,
        _ctx: &ViewCtx<'_>,
    ) -> Effect {
        match action {
            Some(Action::Back) => Effect::Goto(Screen::MainMenu),
            Some(action) => self
                .list
                .on_action(action, Variant::ALL.len())
                .map_or(Effect::None, start),
            None => Effect::None,
        }
    }

    /// Handles a mouse event: the wheel moves focus, a click starts a variant.
    pub fn handle_mouse(&mut self, event: &MouseEvent, _ctx: &ViewCtx<'_>) -> Effect {
        self.list
            .on_mouse(event, Variant::ALL.len())
            .map_or(Effect::None, start)
    }

    /// Draws the title, the variants and the key hints.
    pub fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
        let [_, title_area, _, list_area, _, footer_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(u16::try_from(Variant::ALL.len()).unwrap_or(0)),
            Constraint::Fill(2),
            Constraint::Length(1),
        ])
        .areas(area);
        let title = Line::styled(
            t("common.select.title").into_owned(),
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
        self.list.draw(frame, list_area, &labels, ctx);
        footer::draw(frame, footer_area, &FOOTER, ctx);
    }
}
