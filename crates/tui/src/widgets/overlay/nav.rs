//! Moving between the top tabs of the help window: which key or click leads to which tab, and with
//! what focus the Rules tab opens.

use ratatui::crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::layout::Rect;

use super::how_to_play::{Focus, HowToPlay, Rules};
use super::tabs::{self, HelpTab};
use super::{CommandLine, help};
use crate::action::Action;
use crate::app::Modal;
use crate::mouse;

pub fn tab_of(modal: &Modal) -> Option<HelpTab> {
    match modal {
        Modal::Help => Some(HelpTab::Keys),
        Modal::HowToPlay(_) => Some(HelpTab::Rules),
        Modal::CommandLine(_) => Some(HelpTab::CommandLine),
        _ => None,
    }
}

fn rules_focus(modal: &Modal) -> Option<Focus> {
    match modal {
        Modal::HowToPlay(state) => Some(state.focus),
        _ => None,
    }
}

/// The window on `tab`. The Rules tab opens with the focus on the games unless `entry` says where
/// the key that led there left it.
pub fn modal_for(tab: HelpTab, rules: Rules, entry: Option<Focus>) -> Modal {
    match tab {
        HelpTab::Keys => Modal::Help,
        HelpTab::Rules => {
            Modal::HowToPlay(HowToPlay::new(rules).focused(entry.unwrap_or(Focus::Games)))
        }
        HelpTab::CommandLine => Modal::CommandLine(CommandLine::new()),
    }
}

/// The tab a key switches to, and the focus the Rules tab takes when that is where it goes. Tab
/// and Shift+Tab walk the whole window in reading order: on the Rules tab through its tabs row,
/// its games row and its text, and from either end on to the next or the previous top tab. Left
/// and Right step the top tabs where the tabs row has the focus; the Help and Rules keys jump.
pub fn target(
    modal: &Modal,
    key: &KeyEvent,
    action: Option<Action>,
) -> Option<(HelpTab, Option<Focus>)> {
    let tab = tab_of(modal)?;
    let focus = rules_focus(modal);
    let on_tabs_row = focus.is_none_or(|focus| focus == Focus::Tabs);
    match key.code {
        KeyCode::Tab if focus.is_none_or(|focus| focus == Focus::Text) => {
            return Some((tab.next(), Some(Focus::Tabs)));
        }
        KeyCode::BackTab if on_tabs_row => return Some((tab.previous(), Some(Focus::Text))),
        _ => {}
    }
    match (tab, action?) {
        (HelpTab::Rules | HelpTab::CommandLine, Action::Help) => Some((HelpTab::Keys, None)),
        (HelpTab::Keys | HelpTab::CommandLine, Action::Rules) => Some((HelpTab::Rules, None)),
        (_, Action::Right) if on_tabs_row => Some((tab.next(), Some(Focus::Tabs))),
        (_, Action::Left) if on_tabs_row => Some((tab.previous(), Some(Focus::Tabs))),
        _ => None,
    }
}

pub fn close_help(key: &KeyEvent, action: Option<Action>) -> Option<crate::screens::Effect> {
    let closes = matches!(key.code, KeyCode::Esc | KeyCode::Enter)
        || matches!(action, Some(Action::Select | Action::Back));
    closes.then_some(crate::screens::Effect::None)
}

/// The tab a click on the top row leads to: a tab, or an arrow while the row shows its arrows.
pub fn clicked_tab(modal: &Modal, area: Rect, event: &MouseEvent) -> Option<HelpTab> {
    let current = tab_of(modal)?;
    let cell = mouse::left_press(event)?;
    let row = tabs::top_row(help::geometry(area).body);
    let placed = tabs::top_tabs(row);
    let arrows_shown = rules_focus(modal).is_none_or(|focus| focus == Focus::Tabs);
    placed
        .tab_at(cell)
        .or_else(|| placed.arrow_target(current, cell).filter(|_| arrows_shown))
        .filter(|tab| *tab != current)
}
