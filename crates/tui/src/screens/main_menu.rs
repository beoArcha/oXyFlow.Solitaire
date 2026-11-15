//! The main menu. Stub: P3-04 adds Resume, New game from seed and Help, and the final layout.

use ratatui::Frame;
use ratatui::crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use super::{Effect, ViewCtx};
use crate::action::Action;
use crate::app::Screen;
use crate::i18n::t;
use crate::theme::Token;
use crate::widgets::footer;
use crate::widgets::logo::logo;
use crate::widgets::menu_list::MenuList;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Item {
    NewGame,
    Settings,
    Quit,
}

const ITEMS: [Item; 3] = [Item::NewGame, Item::Settings, Item::Quit];

const FOOTER: [Action; 4] = [Action::Select, Action::Back, Action::Quit, Action::Help];

fn label_key(item: Item) -> &'static str {
    match item {
        Item::NewGame => "common.menu.new_game",
        Item::Settings => "common.menu.settings",
        Item::Quit => "common.menu.quit",
    }
}

fn effect_of(item: Item) -> Effect {
    match item {
        Item::NewGame => Effect::Goto(Screen::GameSelect),
        Item::Settings => Effect::Goto(Screen::Settings),
        Item::Quit => Effect::Quit,
    }
}

/// State of the main menu.
#[derive(Debug, Default)]
pub struct MainMenu {
    list: MenuList,
}

impl MainMenu {
    /// A menu focused on its first item.
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles a key press mapped to `action`.
    pub fn handle(
        &mut self,
        _key: &KeyEvent,
        action: Option<Action>,
        _ctx: &ViewCtx<'_>,
    ) -> Effect {
        match action {
            Some(Action::Quit) => Effect::Quit,
            Some(action) => self
                .list
                .on_action(action, ITEMS.len())
                .and_then(|index| ITEMS.get(index))
                .map_or(Effect::None, |item| effect_of(*item)),
            None => Effect::None,
        }
    }

    /// Handles a mouse event: the wheel moves focus, a click activates an item.
    pub fn handle_mouse(&mut self, event: &MouseEvent, _ctx: &ViewCtx<'_>) -> Effect {
        self.list
            .on_mouse(event, ITEMS.len())
            .and_then(|index| ITEMS.get(index))
            .map_or(Effect::None, |item| effect_of(*item))
    }

    /// Draws the logo, the items and the key hints.
    pub fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
        let rows = logo(ctx.glyphs.unicode);
        let logo_height = u16::try_from(rows.len()).unwrap_or(0);
        let [_, logo_area, _, list_area, _, footer_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(logo_height),
            Constraint::Length(2),
            Constraint::Length(u16::try_from(ITEMS.len()).unwrap_or(0)),
            Constraint::Fill(2),
            Constraint::Length(1),
        ])
        .areas(area);
        let lines: Vec<Line> = rows
            .iter()
            .map(|row| Line::styled(*row, ctx.theme.style(Token::Accent, Token::Background)))
            .collect();
        frame.render_widget(
            Paragraph::new(lines).alignment(Alignment::Center),
            logo_area,
        );
        let labels: Vec<String> = ITEMS
            .iter()
            .map(|item| t(label_key(*item)).into_owned())
            .collect();
        self.list.draw(frame, list_area, &labels, ctx);
        footer::draw(frame, footer_area, &FOOTER, ctx);
    }
}
