//! The main menu: Resume (when a save exists), the new-game entries, Settings, How to play, Help and Quit.
//! "New game from seed" asks for a seed, then lists the games dealt from it; the menu keeps that
//! flow to itself, so the seed travels in the `Effect::Start` it returns.

mod seed;

use ratatui::Frame;
use ratatui::crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use self::seed::{SeedField, Step};
use super::game_select::GameSelect;
use super::{Effect, ViewCtx};
use crate::action::Action;
use crate::app::{Modal, Screen};
use crate::i18n::t;
use crate::theme::Token;
use crate::widgets::footer;
use crate::widgets::logo::logo;
use crate::widgets::menu_list::MenuList;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Item {
    Resume,
    NewGame,
    NewGameFromSeed,
    Settings,
    HowToPlay,
    Help,
    Quit,
}

fn items(has_saved_game: bool) -> Vec<Item> {
    let mut items = Vec::with_capacity(7);
    if has_saved_game {
        items.push(Item::Resume);
    }
    items.extend([
        Item::NewGame,
        Item::NewGameFromSeed,
        Item::Settings,
        Item::HowToPlay,
        Item::Help,
        Item::Quit,
    ]);
    items
}

const FOOTER: [Action; 4] = [Action::Select, Action::Back, Action::Quit, Action::Help];

fn label_key(item: Item) -> &'static str {
    match item {
        Item::Resume => "menu.resume",
        Item::NewGame => "common.menu.new_game",
        Item::NewGameFromSeed => "menu.new_game_from_seed",
        Item::Settings => "common.menu.settings",
        Item::HowToPlay => "menu.how_to_play.label",
        Item::Help => "menu.help",
        Item::Quit => "common.menu.quit",
    }
}

fn effect_of(item: Item) -> Effect {
    match item {
        Item::Resume => Effect::Resume,
        Item::NewGame => Effect::Goto(Screen::GameSelect),
        Item::NewGameFromSeed => Effect::None,
        Item::Settings => Effect::Goto(Screen::Settings),
        Item::HowToPlay => Effect::HowToPlay,
        Item::Help => Effect::Open(Modal::Help),
        Item::Quit => Effect::Quit,
    }
}

/// Where the menu is: its items, the seed field, or the games of a chosen seed.
#[derive(Debug, Default)]
enum Stage {
    #[default]
    Items,
    Seed(SeedField),
    Variants {
        seed: u64,
        picker: GameSelect,
    },
}

/// State of the main menu.
#[derive(Debug, Default)]
pub struct MainMenu {
    list: MenuList,
    stage: Stage,
}

impl MainMenu {
    /// A menu focused on its first item.
    pub fn new() -> Self {
        Self::default()
    }

    /// A menu that opens on the games dealt from `seed`, as if the seed had just been submitted.
    pub fn for_seed(seed: u64) -> Self {
        Self {
            list: MenuList::new(),
            stage: Stage::Variants {
                seed,
                picker: GameSelect::for_seed(seed),
            },
        }
    }

    /// Whether the seed field is open, so that typed characters are text.
    pub fn captures_text(&self) -> bool {
        matches!(self.stage, Stage::Seed(_))
    }

    /// Handles a key press mapped to `action`.
    pub fn handle(&mut self, key: &KeyEvent, action: Option<Action>, ctx: &ViewCtx<'_>) -> Effect {
        if action == Some(Action::Back) && !matches!(self.stage, Stage::Items) {
            return self.back();
        }
        match &mut self.stage {
            Stage::Items => self.handle_items(action, ctx),
            Stage::Seed(field) => {
                let step = field.handle_key(key);
                self.after_seed(step)
            }
            Stage::Variants { picker, .. } => {
                let effect = picker.handle(key, action, ctx);
                self.leave_variants_on_start(effect)
            }
        }
    }

    /// Handles a mouse event: the wheel moves focus, a click activates an item or a button.
    pub fn handle_mouse(&mut self, event: &MouseEvent, ctx: &ViewCtx<'_>) -> Effect {
        match &mut self.stage {
            Stage::Items => {
                let items = items(ctx.has_saved_game);
                let clicked = self.list.on_mouse(event, items.len());
                match clicked.and_then(|index| items.get(index).copied()) {
                    Some(item) => self.activate(item),
                    None => Effect::None,
                }
            }
            Stage::Seed(field) => {
                let step = field.handle_mouse(event);
                self.after_seed(step)
            }
            Stage::Variants { picker, .. } => {
                let effect = picker.handle_mouse(event, ctx);
                self.leave_variants_on_start(effect)
            }
        }
    }

    fn leave_variants_on_start(&mut self, effect: Effect) -> Effect {
        if matches!(effect, Effect::Start { .. }) {
            self.stage = Stage::Items;
        }
        effect
    }

    fn handle_items(&mut self, action: Option<Action>, ctx: &ViewCtx<'_>) -> Effect {
        let items = items(ctx.has_saved_game);
        match action {
            Some(Action::Quit) => Effect::Quit,
            Some(action) => match self.list.on_action(action, items.len()) {
                Some(index) => items
                    .get(index)
                    .map_or(Effect::None, |item| self.activate(*item)),
                None => Effect::None,
            },
            None => Effect::None,
        }
    }

    fn activate(&mut self, item: Item) -> Effect {
        if item == Item::NewGameFromSeed {
            self.stage = Stage::Seed(SeedField::default());
            return Effect::None;
        }
        effect_of(item)
    }

    fn after_seed(&mut self, step: Step) -> Effect {
        match step {
            Step::Stay => {}
            Step::Back => self.stage = Stage::Items,
            Step::Submit(seed) => {
                self.stage = Stage::Variants {
                    seed,
                    picker: GameSelect::for_seed(seed),
                };
            }
        }
        Effect::None
    }

    fn back(&mut self) -> Effect {
        if let Stage::Variants { seed, .. } = self.stage {
            self.stage = Stage::Seed(SeedField::with_seed(seed));
        } else {
            self.stage = Stage::Items;
        }
        Effect::None
    }

    /// Draws the current stage: the logo and items, the seed field, or the games of the seed.
    pub fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
        match &mut self.stage {
            Stage::Items => self.draw_items(frame, area, ctx),
            Stage::Seed(field) => field.draw(frame, area, ctx),
            Stage::Variants { picker, .. } => picker.draw(frame, area, ctx),
        }
    }

    fn draw_items(&mut self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
        let rows = logo(ctx.glyphs.unicode);
        let items = items(ctx.has_saved_game);
        let logo_height = u16::try_from(rows.len()).unwrap_or(0);
        let [_, logo_area, _, list_area, _, footer_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(logo_height),
            Constraint::Length(2),
            Constraint::Length(u16::try_from(items.len()).unwrap_or(0)),
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
        let labels: Vec<String> = items
            .iter()
            .map(|item| t(label_key(*item)).into_owned())
            .collect();
        self.list.draw(frame, list_area, &labels, ctx);
        footer::draw(frame, footer_area, &FOOTER, ctx);
    }
}

#[cfg(test)]
mod tests;
