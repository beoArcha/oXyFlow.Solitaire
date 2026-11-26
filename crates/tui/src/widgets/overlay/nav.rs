//! Moving between the top tabs of the help window: which key or click leads to which tab.

use std::borrow::Cow;

use ratatui::crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::layout::Rect;

use super::CommandLine;
use super::how_to_play::{HowToPlay, RulesContext};
use super::tabs::{self, HelpTab};
use crate::action::Action;
use crate::app::Modal;
use crate::i18n::t;
use crate::mouse;

pub fn tab_of(modal: &Modal) -> Option<HelpTab> {
    match modal {
        Modal::Help(_) => Some(HelpTab::Keys),
        Modal::HowToPlay(_) => Some(HelpTab::Rules),
        Modal::CommandLine(_) => Some(HelpTab::CommandLine),
        _ => None,
    }
}

/// The window on `tab`.
pub fn modal_for(tab: HelpTab, rules: impl Into<RulesContext>) -> Modal {
    let context = rules.into();
    match tab {
        HelpTab::Keys => Modal::help(),
        HelpTab::Rules => Modal::HowToPlay(
            HowToPlay::with_options(context.rules, context.options).reached_from_help(),
        ),
        HelpTab::CommandLine => Modal::CommandLine(CommandLine::new()),
    }
}

/// The title of the window: the Rules window names its own title, the other windows of the help
/// window are titled "Help".
pub fn window_title(modal: &Modal) -> Cow<'static, str> {
    match modal {
        Modal::HowToPlay(state) => t(state.title_key()),
        _ => t("common.dialog.help_title"),
    }
}

/// The tab a key switches to. Tab and Shift+Tab step between the main Help tabs from every tab;
/// Left and Right step the top tabs on Keys and Command line (on Rules they switch the game). The
/// Help and Rules keys jump.
pub fn target(modal: &Modal, key: &KeyEvent, action: Option<Action>) -> Option<HelpTab> {
    let tab = tab_of(modal)?;
    match key.code {
        KeyCode::Tab => return Some(tab.next()),
        KeyCode::BackTab => return Some(tab.previous()),
        _ => {}
    }
    match (tab, action?) {
        (HelpTab::Rules | HelpTab::CommandLine, Action::Help) => Some(HelpTab::Keys),
        (HelpTab::Keys | HelpTab::CommandLine, Action::Rules) => Some(HelpTab::Rules),
        (HelpTab::Keys | HelpTab::CommandLine, Action::Right) => Some(tab.next()),
        (HelpTab::Keys | HelpTab::CommandLine, Action::Left) => Some(tab.previous()),
        _ => None,
    }
}

pub fn close_help(key: &KeyEvent, action: Option<Action>) -> Option<crate::screens::Effect> {
    let closes = matches!(key.code, KeyCode::Esc | KeyCode::Enter)
        || matches!(action, Some(Action::Select | Action::Back));
    closes.then_some(crate::screens::Effect::None)
}

/// The tab a click on the top row leads to: a tab, or an arrow while the row shows its arrows.
/// `body` is the box body of the window `modal` draws, the same rectangle its drawing uses.
pub fn clicked_tab(modal: &Modal, body: Rect, event: &MouseEvent) -> Option<HelpTab> {
    let current = tab_of(modal)?;
    let cell = mouse::left_press(event)?;
    let row = tabs::top_row(body);
    let placed = tabs::top_tabs(row);
    let arrows_shown = current != HelpTab::Rules;
    placed
        .tab_at(cell)
        .or_else(|| placed.arrow_target(current, cell).filter(|_| arrows_shown))
        .filter(|tab| *tab != current)
}
