//! Game and variant selection. Enter or a click starts the focused variant; with a seed (from
//! "New game from seed") the variant is dealt from that seed.

use ratatui::Frame;
use ratatui::crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use super::{Effect, ViewCtx, variant_label_key};
use crate::action::Action;
use crate::app::Screen;
use crate::i18n::{t, tf};
use crate::theme::Token;
use crate::widgets::footer;
use crate::widgets::menu_list::MenuList;
use oxysol_core::Variant;

const FOOTER: [Action; 3] = [Action::Select, Action::Back, Action::Help];

/// State of the game selection screen.
#[derive(Debug, Default)]
pub struct GameSelect {
    list: MenuList,
    seed: Option<u64>,
}

impl GameSelect {
    /// A list focused on the first variant; the variants are dealt from fresh seeds.
    pub fn new() -> Self {
        Self::default()
    }

    /// A list whose variants are dealt from `seed`.
    pub fn for_seed(seed: u64) -> Self {
        Self {
            list: MenuList::new(),
            seed: Some(seed),
        }
    }

    fn start(&self, index: usize) -> Effect {
        Variant::ALL
            .get(index)
            .map_or(Effect::None, |variant| Effect::Start {
                variant: *variant,
                seed: self.seed,
            })
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
                .map_or(Effect::None, |index| self.start(index)),
            None => Effect::None,
        }
    }

    /// Handles a mouse event: the wheel moves focus, a click starts a variant.
    pub fn handle_mouse(&mut self, event: &MouseEvent, _ctx: &ViewCtx<'_>) -> Effect {
        self.list
            .on_mouse(event, Variant::ALL.len())
            .map_or(Effect::None, |index| self.start(index))
    }

    fn title(&self) -> String {
        match self.seed {
            Some(seed) => tf("menu.select_seed", &[("seed", &seed.to_string())]),
            None => t("common.select.title").into_owned(),
        }
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
        self.list.draw(frame, list_area, &labels, ctx);
        footer::draw(frame, footer_area, &FOOTER, ctx);
    }
}
