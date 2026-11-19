//! State of the "How to play" window: which game it explains and how far it is scrolled.

use oxysol_core::Variant;

use super::content;
use crate::widgets::scrollbar::Metrics;

/// The four games the window explains. Variants of one game share a page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rules {
    Klondike,
    Spider,
    FreeCell,
    Pyramid,
}

impl Rules {
    /// Every game, in tab order.
    pub const ALL: [Rules; 4] = [
        Rules::Klondike,
        Rules::Spider,
        Rules::FreeCell,
        Rules::Pyramid,
    ];

    /// The game a variant belongs to.
    pub const fn of(variant: Variant) -> Self {
        match variant {
            Variant::Klondike(_) => Rules::Klondike,
            Variant::Spider(_) => Rules::Spider,
            Variant::FreeCell => Rules::FreeCell,
            Variant::Pyramid(_) => Rules::Pyramid,
        }
    }

    /// Localization key of the name shown on the tab.
    pub const fn name_key(self) -> &'static str {
        match self {
            Rules::Klondike => "rules.game.klondike",
            Rules::Spider => "rules.game.spider",
            Rules::FreeCell => "rules.game.freecell",
            Rules::Pyramid => "rules.game.pyramid",
        }
    }

    fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|rules| *rules == self)
            .unwrap_or(0)
    }
}

/// The part of the window that has the focus, top to bottom in reading order. Left and Right act
/// on the tabs and on the games, Up and Down scroll the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Tabs,
    Games,
    Text,
}

impl Focus {
    const ORDER: [Focus; 3] = [Focus::Tabs, Focus::Games, Focus::Text];

    fn index(self) -> usize {
        Self::ORDER
            .iter()
            .position(|focus| *focus == self)
            .unwrap_or(0)
    }
}

/// The open window: the game on show, the first visible line of its text and the focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HowToPlay {
    pub rules: Rules,
    pub scroll: u16,
    pub focus: Focus,
    pub(super) drag: Option<u16>,
}

impl HowToPlay {
    /// The window opened on `rules`, at the top, with the focus on the games.
    pub const fn new(rules: Rules) -> Self {
        Self {
            rules,
            scroll: 0,
            focus: Focus::Games,
            drag: None,
        }
    }

    /// The same window with the focus on `focus`.
    pub const fn focused(mut self, focus: Focus) -> Self {
        self.focus = focus;
        self
    }

    /// The window opened on the game of `variant`.
    pub const fn for_variant(variant: Variant) -> Self {
        Self::new(Rules::of(variant))
    }

    /// Shows the next game (`forward`) or the previous one, wrapping around, from the top.
    pub fn switch(&mut self, forward: bool) {
        let count = Rules::ALL.len();
        let index = self.rules.index();
        let next = if forward {
            (index + 1) % count
        } else {
            (index + count - 1) % count
        };
        self.show(Rules::ALL[next]);
    }

    /// Shows `rules` from the top; the focus stays where it is.
    pub fn show(&mut self, rules: Rules) {
        *self = Self::new(rules).focused(self.focus);
    }

    /// Up: the text scrolls a line, and at its top the focus goes up to the games; on the games
    /// it goes up to the tabs.
    pub fn up(&mut self) {
        match self.focus {
            Focus::Text if self.scroll > 0 => self.scroll_by(-1),
            Focus::Text => self.focus = Focus::Games,
            Focus::Games => self.focus = Focus::Tabs,
            Focus::Tabs => {}
        }
    }

    /// Down: the focus goes down from the tabs to the games and on to the text, where it scrolls.
    pub fn down(&mut self) {
        match self.focus {
            Focus::Tabs => self.focus = Focus::Games,
            Focus::Games => self.focus = Focus::Text,
            Focus::Text => self.scroll_by(1),
        }
    }

    /// Tab (`forward`) or Shift+Tab moves the focus one part on. `false` when the focus is
    /// already at that end, so that the key leaves for the next or the previous tab.
    pub fn step_focus(&mut self, forward: bool) -> bool {
        let index = self.focus.index();
        let next = if forward {
            index + 1
        } else {
            index.wrapping_sub(1)
        };
        match Focus::ORDER.get(next) {
            Some(focus) => {
                self.focus = *focus;
                true
            }
            None => false,
        }
    }

    /// The largest value of `scroll` for the current game: the last line sits on the last row.
    pub fn max_scroll(&self) -> u16 {
        let lines = content::rows(self.rules).len();
        u16::try_from(lines.saturating_sub(content::VIEW_ROWS)).unwrap_or(u16::MAX)
    }

    /// Scrolls by `lines` (negative goes up), staying inside the text.
    pub fn scroll_by(&mut self, lines: i32) {
        let target = i32::from(self.scroll).saturating_add(lines);
        let clamped = target.clamp(0, i32::from(self.max_scroll()));
        self.scroll = u16::try_from(clamped).unwrap_or(0);
    }

    /// Scrolls to the first (`false`) or the last line.
    pub fn scroll_to(&mut self, end: bool) {
        self.scroll = if end { self.max_scroll() } else { 0 };
    }

    /// The scroll bar's view of the text: its lines, the `view` rows that show and the position.
    pub fn metrics(&self, view: u16) -> Metrics {
        Metrics {
            total: content::rows(self.rules).len(),
            view: usize::from(view),
            offset: usize::from(self.scroll),
        }
    }

    /// Puts the first visible line on `line`, staying inside the text.
    pub fn scroll_to_line(&mut self, line: usize) {
        let clamped = line.min(usize::from(self.max_scroll()));
        self.scroll = u16::try_from(clamped).unwrap_or(0);
    }
}
