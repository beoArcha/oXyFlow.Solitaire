//! The main menu: Resume (when a save exists), the new-game entries, Statistics, Settings, How to play, Help and Quit,
//! and the language selector in the top-left corner (first in the Tab order, before the items).
//! "New game from seed" asks for a seed, then lists the games dealt from it; the menu keeps that
//! flow to itself, so the seed travels in the `Effect::Start` it returns.

mod banner;
mod resume;
mod seed;

use ratatui::Frame;
use ratatui::crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::{Alignment, Constraint, Layout, Margin, Rect};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use self::banner::{Banner, BannerButton};
use self::seed::{SeedField, Step};
use super::focus::tab_forward;
use super::game_select::GameSelect;
use super::{Effect, ViewCtx};
use crate::action::Action;
use crate::app::{Modal, Screen};
use crate::i18n::{Locale, t, tf};
use crate::mouse::wheel_action;
use crate::settings::Settings;
use crate::theme::Token;
use crate::widgets::footer::{self, FooterMap, KeyHint};

/// Empty columns kept on each side of the footer row, so that the version label and the key line
/// do not touch a docked help window beside the menu.
const FOOTER_MARGIN: u16 = 1;
use crate::widgets::language_picker::{LanguagePicker, Pointed};
use crate::widgets::logo::logo;
use crate::widgets::menu_list::{Highlight, MenuList, Row};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Item {
    Resume,
    NewGame,
    NewGameFromSeed,
    Statistics,
    Settings,
    HowToPlay,
    Help,
    Quit,
}

fn items(has_saved_game: bool) -> Vec<Item> {
    let mut items = Vec::with_capacity(8);
    if has_saved_game {
        items.push(Item::Resume);
    }
    items.extend([
        Item::NewGame,
        Item::NewGameFromSeed,
        Item::Statistics,
        Item::Settings,
        Item::HowToPlay,
        Item::Help,
        Item::Quit,
    ]);
    items
}

/// The key line of the item list. Esc is not shown: in the list it does nothing (it leaves the seed
/// entry and the games of a seed, which have their own key line).
pub(crate) const FOOTER: [KeyHint; 4] = [
    (Action::Select, "menu.footer.open"),
    (Action::FocusNext, "menu.footer.language"),
    (Action::Quit, "help.action.quit"),
    (Action::Help, "help.action.help"),
];

fn label_key(item: Item) -> &'static str {
    match item {
        Item::Resume => "menu.resume",
        Item::NewGame => "common.menu.new_game",
        Item::NewGameFromSeed => "menu.new_game_from_seed",
        Item::Statistics => "stats.label",
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
        Item::Statistics => Effect::Goto(Screen::Stats),
        Item::Settings => Effect::Goto(Screen::Settings),
        Item::HowToPlay => Effect::HowToPlay,
        Item::Help => Effect::Open(Modal::help()),
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
    footer: FooterMap,
    picker: LanguagePicker,
    banner: Banner,
    banner_focus: Option<BannerButton>,
    selector_focus: bool,
    other_pane_focused: bool,
    version: &'static str,
}

impl MainMenu {
    /// A menu focused on its first item, or one that opens on the games dealt from `seed`, as if
    /// the seed had just been submitted. `version` is the effective version shown in the footer.
    pub fn start(seed: Option<u64>, version: &'static str) -> Self {
        let menu = Self {
            version,
            ..Self::default()
        };
        match seed {
            Some(seed) => menu.for_seed(seed),
            None => menu,
        }
    }

    fn for_seed(self, seed: u64) -> Self {
        Self {
            version: self.version,
            list: MenuList::new(),
            stage: Stage::Variants {
                seed,
                picker: GameSelect::for_seed(seed),
            },
            footer: FooterMap::default(),
            picker: LanguagePicker::new(),
            banner: Banner::default(),
            banner_focus: None,
            selector_focus: false,
            other_pane_focused: self.other_pane_focused,
        }
    }

    /// Records whether the focus of the application is in another pane, such as a docked help
    /// window, so that the menu mutes its selection while it keeps its marker.
    pub fn set_other_pane_focused(&mut self, focused: bool) {
        self.other_pane_focused = focused;
    }

    fn highlight(&self) -> Highlight {
        if self.selector_focus || self.banner_focus.is_some() {
            Highlight::Off
        } else if self.other_pane_focused {
            Highlight::Muted
        } else {
            Highlight::Focused
        }
    }

    /// Clears the selector's tooltip, hover and keyboard focus when a window opens over the menu.
    pub fn window_opened(&mut self) {
        self.picker.forget_hover();
        self.selector_focus = false;
        self.banner_focus = None;
    }

    /// Whether the seed field has the focus, so that typed characters are text.
    pub fn captures_text(&self) -> bool {
        match &self.stage {
            Stage::Seed(field) => field.captures_text(),
            Stage::Variants { picker, .. } => picker.captures_text(),
            Stage::Items => false,
        }
    }

    fn searching_games(&self) -> bool {
        matches!(&self.stage, Stage::Variants { picker, .. } if picker.searching())
    }

    /// Handles a key press mapped to `action`.
    pub fn handle(&mut self, key: &KeyEvent, action: Option<Action>, ctx: &ViewCtx<'_>) -> Effect {
        let leaves = !matches!(self.stage, Stage::Items) && !self.searching_games();
        if action == Some(Action::Back) && leaves {
            return self.back();
        }
        match &mut self.stage {
            Stage::Items => self.handle_items(action, ctx),
            Stage::Seed(field) => {
                let step = field.handle_key(key, action);
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
            Stage::Items => self.handle_items_mouse(event, ctx),
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

    /// The footer button under `cell`: the menu's own key line, or the one of the games of a seed.
    pub fn footer_action_at(&self, cell: (u16, u16)) -> Option<Action> {
        match &self.stage {
            Stage::Items => self.footer.action_at(cell),
            Stage::Seed(_) => None,
            Stage::Variants { picker, .. } => picker.footer_action_at(cell),
        }
    }

    fn leave_variants_on_start(&mut self, effect: Effect) -> Effect {
        if matches!(effect, Effect::Start { .. }) {
            self.stage = Stage::Items;
        }
        effect
    }

    fn handle_items(&mut self, action: Option<Action>, ctx: &ViewCtx<'_>) -> Effect {
        let items = items(ctx.saved.is_some());
        let Some(action) = action else {
            return Effect::None;
        };
        if self.picker.is_open() {
            let chosen = self.picker.on_action(action, ctx.settings.locale);
            return language_effect(chosen, ctx.settings);
        }
        if action == Action::Quit {
            return Effect::Quit;
        }
        if ctx.settings.low_vision_offered {
            self.banner_focus = None;
        }
        if let Some(effect) = banner::handle_action(&mut self.banner_focus, action, ctx) {
            return effect;
        }
        if self.selector_focus {
            return self.handle_selector(action, items.len(), ctx);
        }
        if !ctx.settings.low_vision_offered && matches!(action, Action::Left | Action::Right) {
            self.banner_focus = Some(BannerButton::Yes);
            return Effect::None;
        }
        if action == Action::Up && self.list.focus() == 0 {
            self.selector_focus = true;
            return Effect::None;
        }
        if let Some(forward) = tab_forward(action)
            && self.leaves_list(forward, items.len())
        {
            self.selector_focus = true;
            return Effect::None;
        }
        match self.list.on_action(action, items.len()) {
            Some(index) => items
                .get(index)
                .map_or(Effect::None, |item| self.activate(*item)),
            None => Effect::None,
        }
    }

    fn leaves_list(&self, forward: bool, len: usize) -> bool {
        if forward {
            self.list.focus() + 1 >= len
        } else {
            self.list.focus() == 0
        }
    }

    fn handle_selector(&mut self, action: Action, len: usize, ctx: &ViewCtx<'_>) -> Effect {
        if let Some(forward) = tab_forward(action) {
            self.selector_focus = false;
            self.list = MenuList::new();
            if !forward {
                self.list.on_action(Action::Up, len);
            }
            return Effect::None;
        }
        if action == Action::Down {
            self.selector_focus = false;
            self.list = MenuList::new();
            return Effect::None;
        }
        if action == Action::Up {
            self.selector_focus = false;
            self.list = MenuList::new();
            self.list.on_action(Action::Up, len);
            return Effect::None;
        }
        let chosen = self.picker.on_action(action, ctx.settings.locale);
        language_effect(chosen, ctx.settings)
    }

    fn handle_items_mouse(&mut self, event: &MouseEvent, ctx: &ViewCtx<'_>) -> Effect {
        match self.picker.on_mouse(event, ctx.settings.locale) {
            Pointed::Handled => return Effect::None,
            Pointed::Focus => {
                self.selector_focus = true;
                self.banner_focus = None;
                return Effect::None;
            }
            Pointed::Choose(locale) => {
                self.selector_focus = true;
                self.banner_focus = None;
                return language_effect(Some(locale), ctx.settings);
            }
            Pointed::Ignored => {}
        }
        if !ctx.settings.low_vision_offered
            && let Some(button) = self.banner.hit(event)
        {
            self.selector_focus = false;
            self.banner_focus = None;
            return banner::effect(button == BannerButton::Yes, ctx);
        }
        let items = items(ctx.saved.is_some());
        let clicked = self.list.on_mouse(event, items.len());
        if clicked.is_some() || wheel_action(event).is_some() {
            self.selector_focus = false;
            self.banner_focus = None;
        }
        match clicked.and_then(|index| items.get(index).copied()) {
            Some(item) => self.activate(item),
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
        let items = items(ctx.saved.is_some());
        let list_rows: Vec<Row> = items
            .iter()
            .map(|item| match (item, ctx.saved) {
                (Item::Resume, Some(saved)) => resume::row(saved, area.width),
                _ => Row::new(t(label_key(*item)).into_owned()),
            })
            .collect();
        let logo_height = u16::try_from(rows.len()).unwrap_or(0);
        let list_height = MenuList::height(&list_rows);
        let (logo_area, tagline_area, list_area, banner_area, footer_row) =
            if !ctx.settings.low_vision_offered {
                let [
                    _,
                    logo_area,
                    tagline_area,
                    _,
                    list_area,
                    _,
                    banner_area,
                    _,
                    footer_row,
                ] = Layout::vertical([
                    Constraint::Fill(1),
                    Constraint::Length(logo_height),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(list_height),
                    Constraint::Length(1),
                    Constraint::Length(banner::HEIGHT),
                    Constraint::Fill(1),
                    Constraint::Length(1),
                ])
                .areas(area);
                (
                    logo_area,
                    tagline_area,
                    list_area,
                    Some(banner_area),
                    footer_row,
                )
            } else {
                let [_, logo_area, tagline_area, _, list_area, _, footer_row] = Layout::vertical([
                    Constraint::Fill(1),
                    Constraint::Length(logo_height),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(list_height),
                    Constraint::Fill(2),
                    Constraint::Length(1),
                ])
                .areas(area);
                (logo_area, tagline_area, list_area, None, footer_row)
            };
        let footer_area = footer_row.inner(Margin::new(FOOTER_MARGIN, 0));
        let lines: Vec<Line> = rows
            .iter()
            .map(|row| Line::styled(*row, ctx.theme.style(Token::Accent, Token::Background)))
            .collect();
        frame.render_widget(
            Paragraph::new(lines).alignment(Alignment::Center),
            logo_area,
        );
        let tagline = Line::styled(
            t("common.app.tagline").into_owned(),
            ctx.theme.style(Token::TextMuted, Token::Background),
        );
        frame.render_widget(
            Paragraph::new(tagline).alignment(Alignment::Center),
            tagline_area,
        );
        self.list
            .draw(frame, list_area, &list_rows, ctx, self.highlight());
        if let Some(banner_area) = banner_area {
            self.banner.draw(frame, banner_area, self.banner_focus, ctx);
        } else {
            self.banner.clear();
        }
        self.footer = footer::draw(
            frame,
            footer_area,
            &footer::key_segments(&FOOTER),
            Alignment::Center,
            footer::key_style(ctx),
            ctx,
        );
        let version = tf("menu.version", &[("version", self.version)]);
        frame.render_widget(
            Paragraph::new(Line::styled(
                version,
                ctx.theme.style(Token::TextMuted, Token::Background),
            ))
            .alignment(Alignment::Right),
            footer_area,
        );
        self.picker
            .draw(frame, area, ctx.settings.locale, self.selector_focus, ctx);
    }
}

/// The effect of choosing `locale` in the selector: the same settings with the new language, which
/// the application applies and saves at once.
fn language_effect(locale: Option<Locale>, settings: &Settings) -> Effect {
    match locale {
        Some(locale) if locale != settings.locale => Effect::Apply(Settings {
            locale,
            ..settings.clone()
        }),
        _ => Effect::None,
    }
}

#[cfg(test)]
mod banner_tests;
#[cfg(test)]
mod dock_focus_tests;
#[cfg(test)]
mod focus_tests;
#[cfg(test)]
mod footer_margin_tests;
#[cfg(test)]
mod language_tests;
#[cfg(test)]
mod resume_tests;
#[cfg(test)]
mod tests;
