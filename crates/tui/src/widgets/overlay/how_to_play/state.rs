//! State of the "How to play" window: which game it explains and how far it is scrolled.

use oxysol_core::Variant;

use super::content;
use crate::widgets::scrollbar::Metrics;

/// The games the window explains. Variants of one game share a page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rules {
    Klondike,
    Spider,
    FreeCell,
    Pyramid,
    Yukon,
    Golf,
}

impl Rules {
    /// Every game, in tab order.
    pub const ALL: [Rules; 6] = [
        Rules::Klondike,
        Rules::Spider,
        Rules::FreeCell,
        Rules::Pyramid,
        Rules::Yukon,
        Rules::Golf,
    ];

    /// The game a variant belongs to.
    pub const fn of(variant: Variant) -> Self {
        match variant {
            Variant::Klondike(_) => Rules::Klondike,
            Variant::Spider(_) => Rules::Spider,
            Variant::FreeCell => Rules::FreeCell,
            Variant::Pyramid(_) => Rules::Pyramid,
            Variant::Yukon(_) => Rules::Yukon,
            Variant::Golf => Rules::Golf,
        }
    }

    /// Localization key of the name shown on the tab.
    pub const fn name_key(self) -> &'static str {
        match self {
            Rules::Klondike => "rules.game.klondike",
            Rules::Spider => "rules.game.spider",
            Rules::FreeCell => "rules.game.freecell",
            Rules::Pyramid => "rules.game.pyramid",
            Rules::Yukon => "rules.game.yukon",
            Rules::Golf => "rules.game.golf",
        }
    }

    fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|rules| *rules == self)
            .unwrap_or(0)
    }
}

/// The open window: the game on show and the first visible line of its text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HowToPlay {
    pub rules: Rules,
    pub scroll: u16,
    pub from_help: bool,
    pub(super) drag: Option<u16>,
    view: u16,
}

impl HowToPlay {
    /// The window opened on `rules`, at the top.
    pub const fn new(rules: Rules) -> Self {
        Self {
            rules,
            scroll: 0,
            from_help: false,
            drag: None,
            view: content::VIEW_ROWS as u16,
        }
    }

    /// The same window, reached from the Help window through its tabs.
    pub const fn reached_from_help(mut self) -> Self {
        self.from_help = true;
        self
    }

    /// The window's title: "How to play" when it was opened as the rules, "Help · Rules" when the
    /// Rules tab was reached from the Help window.
    pub const fn title_key(self) -> &'static str {
        if self.from_help {
            "help.rules.title_from_help"
        } else {
            "help.rules.title"
        }
    }

    /// Takes the number of text rows the box shows at the current size.
    pub fn set_view(&mut self, rows: u16) {
        self.view = rows;
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

    /// Shows `rules` from the top.
    pub fn show(&mut self, rules: Rules) {
        let from_help = self.from_help;
        *self = Self::new(rules);
        self.from_help = from_help;
    }

    /// The largest value of `scroll` for the current game: the last line sits on the last row.
    pub fn max_scroll(&self) -> u16 {
        let lines = content::rows(self.rules).len();
        u16::try_from(lines.saturating_sub(usize::from(self.view))).unwrap_or(u16::MAX)
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
