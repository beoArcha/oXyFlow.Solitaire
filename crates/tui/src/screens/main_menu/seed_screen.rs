//! The seed screen of "New game from seed", shown once the game is chosen. Its options: type a seed
//! (the typing page, see `seed.rs`), replay one of the games already played in that game, a random
//! replay of one of them, or pick one of the curated seeds of the five difficulty levels. Esc or
//! Back on the options goes back to the game list; on a page of its own it goes back to the options.
//! Starting a seed returns `Outcome::Start`, which the menu turns into `Effect::Start`.

mod draw;

use ratatui::crossterm::event::{KeyEvent, MouseEvent};

use super::seed::{SeedField, Step};
use super::seed_levels::{self, LevelSeed, Lookup};
use super::seed_played::Played;
use crate::action::Action;
use crate::i18n::t;
use crate::storage::GameRecord;
use crate::widgets::footer::{FooterMap, KeyHint};
use crate::widgets::menu_list::MenuList;
use oxysol_core::curated;
use oxysol_core::{GameOptions, Variant};

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
    /// Deal a played game again, with the options and the undo mode it was played with.
    Replay {
        variant: Variant,
        seed: u64,
        options: GameOptions,
        undo_allowed: bool,
    },
}

/// The entries of the options page, in the order they are listed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Choice {
    Type,
    Played,
    Random,
    Levels,
}

const CHOICES: [Choice; 4] = [Choice::Type, Choice::Played, Choice::Random, Choice::Levels];

const OPTIONS_FOOTER: [KeyHint; 2] = [
    (Action::Select, "menu.footer.open"),
    (Action::Back, "help.action.back"),
];
const LIST_FOOTER: [KeyHint; 2] = [
    (Action::Select, "menu.footer.play"),
    (Action::Back, "help.action.back"),
];

const NO_PLAYED_KEY: &str = "menu.seed.played_none";
const LEVELS_STANDARD_KEY: &str = "menu.seed.levels_standard_only";
const UNREADABLE_KEY: &str = "menu.seed.history_unreadable";

/// The replay of a played game, as the statistics replay deals it.
const fn replay_of(game: &GameRecord) -> Outcome {
    Outcome::Replay {
        variant: game.variant,
        seed: game.seed,
        options: game.options,
        undo_allowed: game.undo_allowed,
    }
}

/// Whether the curated levels, rated for the standard rules, apply to `variant` under `options`.
pub(super) fn levels_offered(variant: Variant, options: GameOptions) -> bool {
    options.is_default_for(variant)
}

impl Choice {
    const fn label_key(self) -> &'static str {
        match self {
            Choice::Type => "menu.seed.type_seed",
            Choice::Played => "menu.seed.played",
            Choice::Random => "menu.seed.random_replay",
            Choice::Levels => "menu.seed.levels",
        }
    }
}

/// The page on screen. The list state of the options and the lists is the one `SeedScreen` holds.
#[derive(Debug)]
enum Page {
    Options,
    Typing(SeedField),
    Played,
    Levels(Vec<LevelSeed>),
}

/// The seed screen of one game variant.
#[derive(Debug)]
pub struct SeedScreen {
    variant: Variant,
    lookup: Lookup,
    page: Page,
    played: Played,
    /// Localization key of the message under the list, shown until the next key or click.
    notice: Option<&'static str>,
    list: MenuList,
    footer: FooterMap,
}

impl SeedScreen {
    /// The options for `variant`, with the curated seeds of the shipped table and the games in
    /// `played` (already those of `variant`).
    pub fn new(variant: Variant, played: Played) -> Self {
        Self::with_lookup(variant, curated::seeds, played)
    }

    /// The options for `variant`, whose levels read their seeds through `lookup`.
    pub fn with_lookup(variant: Variant, lookup: Lookup, played: Played) -> Self {
        Self {
            variant,
            lookup,
            page: Page::Options,
            played,
            notice: None,
            list: MenuList::new(),
            footer: FooterMap::default(),
        }
    }

    /// Whether the typing page has the focus, so that typed characters are the seed.
    pub fn captures_text(&self) -> bool {
        matches!(&self.page, Page::Typing(field) if field.captures_text())
    }

    #[cfg(feature = "dev")]
    pub(super) fn dev_state(
        &self,
    ) -> (
        &'static str,
        &'static str,
        Option<usize>,
        Option<&'static str>,
        Option<&'static str>,
    ) {
        match &self.page {
            Page::Options => (
                "seed_options",
                "seed_choice",
                Some(self.list.focus()),
                self.notice,
                None,
            ),
            Page::Typing(field) => {
                let (focus, err) = field.dev_focus();
                ("seed_typing", focus, None, self.notice, err)
            }
            Page::Played => (
                "seed_played",
                "played_seed",
                Some(self.list.focus()),
                self.notice,
                None,
            ),
            Page::Levels(_) => (
                "seed_levels",
                "level_seed",
                Some(self.list.focus()),
                self.notice,
                None,
            ),
        }
    }

    fn show(&mut self, page: Page) {
        self.page = page;
        self.list = MenuList::new();
        self.notice = None;
    }

    /// Esc or the Back button: the options leave the screen, a page goes back to the options.
    pub fn back(&mut self) -> Outcome {
        match self.page {
            Page::Options => Outcome::Leave,
            Page::Typing(_) | Page::Played | Page::Levels(_) => {
                self.show(Page::Options);
                Outcome::Stay
            }
        }
    }

    /// Handles a key press mapped to `action` on the current page. `options` are the game options of
    /// the settings, which decide whether the levels are offered.
    pub fn handle(
        &mut self,
        key: &KeyEvent,
        action: Option<Action>,
        options: GameOptions,
    ) -> Outcome {
        self.notice = None;
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
                    Some(index) => self.choose(index, options),
                    None => Outcome::Stay,
                }
            }
            Page::Played => {
                let Some(action) = action else {
                    return Outcome::Stay;
                };
                match self.list.on_action(action, self.played.len()) {
                    Some(index) => self.start_played(index),
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
    pub fn handle_mouse(&mut self, event: &MouseEvent, options: GameOptions) -> Outcome {
        self.notice = None;
        match &mut self.page {
            Page::Typing(field) => {
                let step = field.handle_mouse(event);
                self.after_field(step)
            }
            Page::Options => match self.list.on_mouse(event, CHOICES.len()) {
                Some(index) => self.choose(index, options),
                None => Outcome::Stay,
            },
            Page::Played => match self.list.on_mouse(event, self.played.len()) {
                Some(index) => self.start_played(index),
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
            Page::Options | Page::Played | Page::Levels(_) => self.footer.action_at(cell),
        }
    }

    fn choose(&mut self, index: usize, options: GameOptions) -> Outcome {
        match CHOICES.get(index) {
            Some(Choice::Type) => self.show(Page::Typing(SeedField::default())),
            Some(Choice::Played) => self.show_played(),
            Some(Choice::Random) => return self.random_replay(),
            Some(Choice::Levels) if levels_offered(self.variant, options) => {
                let entries = seed_levels::entries(self.variant, self.lookup);
                self.show(Page::Levels(entries));
            }
            Some(Choice::Levels) | None => {}
        }
        Outcome::Stay
    }

    /// The message under the options: a notice of the last key, or the reason why the levels are
    /// not offered under the game options of the settings.
    fn options_notice(&self, options: GameOptions) -> String {
        match self.notice {
            Some(key) => t(key).into_owned(),
            None if !levels_offered(self.variant, options) => t(LEVELS_STANDARD_KEY).into_owned(),
            None => String::new(),
        }
    }

    /// The message for a game list or random replay that has no game to offer.
    fn empty_notice(&self) -> &'static str {
        if self.played.is_unreadable() {
            UNREADABLE_KEY
        } else {
            NO_PLAYED_KEY
        }
    }

    fn show_played(&mut self) {
        if self.played.is_empty() {
            self.notice = Some(self.empty_notice());
        } else {
            self.show(Page::Played);
        }
    }

    fn random_replay(&mut self) -> Outcome {
        match self.played.random_game() {
            Some(game) => replay_of(game),
            None => {
                self.notice = Some(self.empty_notice());
                Outcome::Stay
            }
        }
    }

    fn start_played(&self, index: usize) -> Outcome {
        self.played.game_at(index).map_or(Outcome::Stay, replay_of)
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
            Page::Options | Page::Typing(_) | Page::Played => Outcome::Stay,
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
}
