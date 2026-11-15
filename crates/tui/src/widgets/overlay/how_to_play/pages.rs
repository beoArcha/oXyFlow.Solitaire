//! What each game's page is made of: the localization keys of its text and the controls it lists.
//! The words live in `locales/*/rules.toml`; the keys come from the key table at draw time.

use super::state::Rules;
use crate::action::Action;

/// One line of the controls section: the actions whose keys are shown and what they do.
#[derive(Debug, Clone, Copy)]
pub struct Control {
    pub actions: &'static [Action],
    pub text: &'static str,
}

/// The content of one game's page.
#[derive(Debug, Clone, Copy)]
pub struct Page {
    pub goal: &'static [&'static str],
    pub layout: &'static [&'static str],
    pub moves: &'static [&'static str],
    pub controls: &'static [Control],
    pub controls_note: &'static str,
    pub tips: &'static [&'static str],
}

const NAVIGATE: Control = Control {
    actions: &[Action::Left, Action::Right, Action::Up, Action::Down],
    text: "rules.control.navigate",
};
const SELECT: Control = Control {
    actions: &[Action::Select],
    text: "rules.control.select",
};
const SELECT_PAIR: Control = Control {
    actions: &[Action::Select],
    text: "rules.control.select_pair",
};
const BACK: Control = Control {
    actions: &[Action::Back],
    text: "rules.control.back",
};
const STOCK: Control = Control {
    actions: &[Action::Stock],
    text: "rules.control.stock",
};
const DEAL: Control = Control {
    actions: &[Action::Stock],
    text: "rules.control.deal",
};
const RUN: Control = Control {
    actions: &[Action::RunMore, Action::RunLess],
    text: "rules.control.run",
};
const QUICK_MOVE: Control = Control {
    actions: &[Action::QuickMove],
    text: "rules.control.quick_move",
};
const UNDO: Control = Control {
    actions: &[Action::Undo],
    text: "rules.control.undo",
};
const HINT: Control = Control {
    actions: &[Action::Hint],
    text: "rules.control.hint",
};

const KLONDIKE: Page = Page {
    goal: &["rules.klondike.goal"],
    layout: &[
        "rules.klondike.layout_1",
        "rules.klondike.layout_2",
        "rules.klondike.layout_3",
    ],
    moves: &[
        "rules.klondike.moves_1",
        "rules.klondike.moves_2",
        "rules.klondike.moves_3",
        "rules.klondike.moves_4",
        "rules.klondike.moves_5",
    ],
    controls: &[NAVIGATE, SELECT, BACK, STOCK, RUN, QUICK_MOVE, UNDO, HINT],
    controls_note: "rules.control.mouse",
    tips: &[
        "rules.klondike.tips_1",
        "rules.klondike.tips_2",
        "rules.klondike.tips_3",
        "rules.klondike.tips_4",
    ],
};

const SPIDER: Page = Page {
    goal: &["rules.spider.goal"],
    layout: &[
        "rules.spider.layout_1",
        "rules.spider.layout_2",
        "rules.spider.layout_3",
    ],
    moves: &[
        "rules.spider.moves_1",
        "rules.spider.moves_2",
        "rules.spider.moves_3",
        "rules.spider.moves_4",
        "rules.spider.moves_5",
    ],
    controls: &[NAVIGATE, SELECT, BACK, DEAL, RUN, QUICK_MOVE, UNDO, HINT],
    controls_note: "rules.control.mouse",
    tips: &[
        "rules.spider.tips_1",
        "rules.spider.tips_2",
        "rules.spider.tips_3",
        "rules.spider.tips_4",
    ],
};

const FREECELL: Page = Page {
    goal: &["rules.freecell.goal"],
    layout: &["rules.freecell.layout_1", "rules.freecell.layout_2"],
    moves: &[
        "rules.freecell.moves_1",
        "rules.freecell.moves_2",
        "rules.freecell.moves_3",
        "rules.freecell.moves_4",
        "rules.freecell.moves_5",
    ],
    controls: &[NAVIGATE, SELECT, BACK, RUN, QUICK_MOVE, UNDO, HINT],
    controls_note: "rules.control.mouse",
    tips: &[
        "rules.freecell.tips_1",
        "rules.freecell.tips_2",
        "rules.freecell.tips_3",
        "rules.freecell.tips_4",
    ],
};

const PYRAMID: Page = Page {
    goal: &["rules.pyramid.goal"],
    layout: &[
        "rules.pyramid.layout_1",
        "rules.pyramid.layout_2",
        "rules.pyramid.layout_3",
        "rules.pyramid.layout_4",
    ],
    moves: &[
        "rules.pyramid.moves_1",
        "rules.pyramid.moves_2",
        "rules.pyramid.moves_3",
        "rules.pyramid.moves_4",
    ],
    controls: &[NAVIGATE, SELECT_PAIR, BACK, STOCK, QUICK_MOVE, UNDO, HINT],
    controls_note: "rules.control.mouse",
    tips: &[
        "rules.pyramid.tips_1",
        "rules.pyramid.tips_2",
        "rules.pyramid.tips_3",
    ],
};

/// The page of `rules`.
pub const fn page(rules: Rules) -> Page {
    match rules {
        Rules::Klondike => KLONDIKE,
        Rules::Spider => SPIDER,
        Rules::FreeCell => FREECELL,
        Rules::Pyramid => PYRAMID,
    }
}
