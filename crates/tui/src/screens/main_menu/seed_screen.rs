//! The seed screen of "New game from seed", shown once the game is chosen. Its options: type a seed
//! (the typing page, see `seed.rs`) or pick one of the curated seeds of the five difficulty levels.
//! Esc or Back on the options goes back to the game list; on a page of its own it goes back to the
//! options. Starting a seed returns `Outcome::Start`, which the menu turns into `Effect::Start`.

use ratatui::Frame;
use ratatui::crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use super::seed::{SeedField, Step};
use super::seed_levels::{self, LevelSeed, Lookup};
use crate::action::Action;
use crate::i18n::{t, tf};
use crate::screens::{ViewCtx, variant_label_key};
use crate::theme::Token;
use crate::widgets::footer::{self, FooterMap, KeyHint};
use crate::widgets::menu_list::{Highlight, MenuList, Row};
use oxysol_core::Variant;
use oxysol_core::curated;

/// What the seed screen asks the menu to do after an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Stay,
    /// Back to the game list: Esc or Back on the options.
    Leave,
    /// Deal `variant` from `seed`.
    Start {
        variant: Variant,
        seed: u64,
    },
}

/// The entries of the options page, in the order they are listed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Choice {
    Type,
    Levels,
}

const CHOICES: [Choice; 2] = [Choice::Type, Choice::Levels];

const OPTIONS_FOOTER: [KeyHint; 2] = [
    (Action::Select, "menu.footer.open"),
    (Action::Back, "help.action.back"),
];
const LEVELS_FOOTER: [KeyHint; 2] = [
    (Action::Select, "menu.footer.play"),
    (Action::Back, "help.action.back"),
];

impl Choice {
    const fn label_key(self) -> &'static str {
        match self {
            Choice::Type => "menu.seed.type_seed",
            Choice::Levels => "menu.seed.levels",
        }
    }
}

/// The page on screen. The list state of the options and the levels is the one `SeedScreen` holds.
#[derive(Debug)]
enum Page {
    Options,
    Typing(SeedField),
    Levels(Vec<LevelSeed>),
}

/// The seed screen of one game variant.
#[derive(Debug)]
pub struct SeedScreen {
    variant: Variant,
    lookup: Lookup,
    page: Page,
    list: MenuList,
    footer: FooterMap,
}

impl SeedScreen {
    /// The options for `variant`, with the curated seeds of the shipped table.
    pub fn new(variant: Variant) -> Self {
        Self::with_lookup(variant, curated::seeds)
    }

    /// The options for `variant`, whose levels read their seeds through `lookup`.
    pub fn with_lookup(variant: Variant, lookup: Lookup) -> Self {
        Self {
            variant,
            lookup,
            page: Page::Options,
            list: MenuList::new(),
            footer: FooterMap::default(),
        }
    }

    /// Whether the typing page has the focus, so that typed characters are the seed.
    pub fn captures_text(&self) -> bool {
        matches!(&self.page, Page::Typing(field) if field.captures_text())
    }

    fn show(&mut self, page: Page) {
        self.page = page;
        self.list = MenuList::new();
    }

    /// Esc or the Back button: the options leave the screen, a page goes back to the options.
    pub fn back(&mut self) -> Outcome {
        match self.page {
            Page::Options => Outcome::Leave,
            Page::Typing(_) | Page::Levels(_) => {
                self.show(Page::Options);
                Outcome::Stay
            }
        }
    }

    /// Handles a key press mapped to `action` on the current page.
    pub fn handle(&mut self, key: &KeyEvent, action: Option<Action>) -> Outcome {
        match &mut self.page {
            Page::Typing(field) => {
                let step = field.handle_key(key, action);
                self.after_field(step)
            }
            Page::Options => {
                let Some(action) = action else {
                    return Outcome::Stay;
                };
                match self.list.on_action(action, CHOICES.len()) {
                    Some(index) => self.choose(index),
                    None => Outcome::Stay,
                }
            }
            Page::Levels(entries) => {
                let Some(action) = action else {
                    return Outcome::Stay;
                };
                match self.list.on_action(action, entries.len()) {
                    Some(index) => self.start_level(index),
                    None => Outcome::Stay,
                }
            }
        }
    }

    /// Handles a mouse event: a click on a row chooses it, a click on a button presses it.
    pub fn handle_mouse(&mut self, event: &MouseEvent) -> Outcome {
        match &mut self.page {
            Page::Typing(field) => {
                let step = field.handle_mouse(event);
                self.after_field(step)
            }
            Page::Options => match self.list.on_mouse(event, CHOICES.len()) {
                Some(index) => self.choose(index),
                None => Outcome::Stay,
            },
            Page::Levels(entries) => {
                let count = entries.len();
                match self.list.on_mouse(event, count) {
                    Some(index) => self.start_level(index),
                    None => Outcome::Stay,
                }
            }
        }
    }

    /// The footer button under `cell`; the typing page has its own key line instead.
    pub fn footer_action_at(&self, cell: (u16, u16)) -> Option<Action> {
        match self.page {
            Page::Typing(_) => None,
            Page::Options | Page::Levels(_) => self.footer.action_at(cell),
        }
    }

    fn choose(&mut self, index: usize) -> Outcome {
        match CHOICES.get(index) {
            Some(Choice::Type) => self.show(Page::Typing(SeedField::default())),
            Some(Choice::Levels) => {
                let entries = seed_levels::entries(self.variant, self.lookup);
                self.show(Page::Levels(entries));
            }
            None => {}
        }
        Outcome::Stay
    }

    fn start_level(&self, index: usize) -> Outcome {
        match &self.page {
            Page::Levels(entries) => {
                entries
                    .get(index)
                    .map_or(Outcome::Stay, |entry| Outcome::Start {
                        variant: self.variant,
                        seed: entry.seed,
                    })
            }
            Page::Options | Page::Typing(_) => Outcome::Stay,
        }
    }

    fn after_field(&mut self, step: Step) -> Outcome {
        match step {
            Step::Stay => Outcome::Stay,
            Step::Back => self.back(),
            Step::Submit(seed) => Outcome::Start {
                variant: self.variant,
                seed,
            },
        }
    }

    fn title(&self) -> String {
        let game = t(variant_label_key(self.variant));
        tf("menu.seed.title_game", &[("game", game.as_ref())])
    }

    /// Draws the current page.
    pub fn draw(&mut self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
        let title = self.title();
        match &mut self.page {
            Page::Typing(field) => field.draw(frame, area, ctx, &title),
            Page::Options => {
                let rows: Vec<Row> = CHOICES
                    .iter()
                    .map(|choice| Row::new(t(choice.label_key()).into_owned()))
                    .collect();
                let prompt = t("menu.seed.options_prompt").into_owned();
                let page = Heading {
                    title: &title,
                    prompt: &prompt,
                };
                self.draw_list(frame, area, ctx, page, &rows, &OPTIONS_FOOTER);
            }
            Page::Levels(entries) => {
                let rows = seed_levels::rows(entries);
                let prompt = t("menu.seed.levels_prompt").into_owned();
                let page = Heading {
                    title: &title,
                    prompt: &prompt,
                };
                self.draw_list(frame, area, ctx, page, &rows, &LEVELS_FOOTER);
            }
        }
    }

    fn draw_list(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        ctx: &ViewCtx<'_>,
        heading: Heading<'_>,
        rows: &[Row],
        keys: &[KeyHint],
    ) {
        let list_height = MenuList::height(rows).max(1);
        let [_, title_area, _, prompt_area, _, list_area, _, footer_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(list_height),
            Constraint::Fill(2),
            Constraint::Length(1),
        ])
        .areas(area);
        draw_line(
            frame,
            title_area,
            heading.title,
            ctx.theme.style(Token::Accent, Token::Background),
        );
        draw_line(frame, prompt_area, heading.prompt, ctx.theme.base());
        if rows.is_empty() {
            draw_line(
                frame,
                list_area,
                &t("menu.seed.levels_none"),
                ctx.theme.style(Token::TextMuted, Token::Background),
            );
        } else {
            self.list
                .draw(frame, list_area, rows, ctx, Highlight::Focused);
        }
        self.footer = footer::draw(
            frame,
            footer_area,
            &footer::key_segments(keys),
            Alignment::Center,
            footer::key_style(ctx),
            ctx,
        );
    }
}

/// The title and the prompt above a list.
struct Heading<'a> {
    title: &'a str,
    prompt: &'a str,
}

fn draw_line(frame: &mut Frame, area: Rect, text: &str, style: Style) {
    let line = Line::styled(text.to_owned(), style);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}
