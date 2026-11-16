//! The modal layer: help, how to play, pause, win, confirmation and alert dialogs. [`draw`] paints the open
//! dialog over the screen; [`handle`] and [`handle_mouse`] turn input into the [`Effect`] that
//! closes it. Every dialog sizes itself to its content and is centered in the area it is given.

mod command_line;
#[cfg(test)]
mod command_line_tests;
mod dialog;
mod help;
#[cfg(test)]
mod help_rules_tests;
mod how_to_play;
mod pause;
#[cfg(test)]
mod render_tests;
mod tabs;
#[cfg(test)]
mod tests;
mod win;

use std::time::Instant;

use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Clear};

pub use self::command_line::CommandLine;
use self::dialog::{Button, Geometry, Text};
pub use self::how_to_play::{HowToPlay, Rules};
pub use self::tabs::{HelpTab, TopTabs};
use crate::action::Action;
use crate::app::{Modal, Screen};
use crate::i18n::t;
use crate::layout::chrome;
use crate::mouse;
use crate::screens::board::BoardScreen;
use crate::screens::{Effect, ViewCtx};

fn text_of(modal: &Modal, board: &BoardScreen, now: Instant) -> Option<Text> {
    match modal {
        Modal::Help | Modal::HowToPlay(_) | Modal::CommandLine(_) => None,
        Modal::Pause => Some(pause::text()),
        Modal::Win { menu_focused } => {
            let moves = board.session().map_or(0, |session| session.moves_made());
            Some(win::text(board.elapsed(now), moves, *menu_focused))
        }
        Modal::Confirm {
            prompt,
            yes_focused,
            ..
        } => Some(Text {
            title: t("common.dialog.confirm_title").into_owned(),
            lines: vec![t(prompt).into_owned()],
            buttons: vec![
                Button {
                    label: t("common.dialog.yes").into_owned(),
                    focused: *yes_focused,
                },
                Button {
                    label: t("common.dialog.no").into_owned(),
                    focused: !*yes_focused,
                },
            ],
        }),
        Modal::Alert { message } => Some(Text {
            title: t("common.dialog.alert_title").into_owned(),
            lines: vec![t(message).into_owned()],
            buttons: vec![Button {
                label: t("common.dialog.ok").into_owned(),
                focused: true,
            }],
        }),
    }
}

fn geometry(area: Rect, modal: &Modal, board: &BoardScreen, now: Instant) -> Geometry {
    match (modal, text_of(modal, board, now)) {
        (_, Some(text)) => text.geometry(area),
        (Modal::HowToPlay(_), None) => how_to_play::geometry(area),
        (_, None) => help::geometry(area),
    }
}

/// Draws `modal` over the screen. The pause dialog first clears the board, so no card shows; the
/// header with the clock stays visible.
pub fn draw(frame: &mut Frame, area: Rect, modal: &Modal, board: &BoardScreen, ctx: &ViewCtx<'_>) {
    if *modal == Modal::Pause {
        let body = chrome(area).body;
        frame.render_widget(Clear, body);
        frame.render_widget(Block::new().style(ctx.theme.base()), body);
    }
    match (modal, text_of(modal, board, ctx.now)) {
        (_, Some(text)) => {
            let geometry = text.geometry(area);
            text.draw(frame, &geometry, ctx);
        }
        (Modal::HowToPlay(state), None) => how_to_play::draw(frame, area, state, ctx),
        (Modal::CommandLine(state), None) => command_line::draw(frame, area, state, ctx),
        (_, None) => help::draw(frame, area, ctx),
    }
}

fn toggle_focus(modal: &mut Modal) -> bool {
    match modal {
        Modal::Confirm { yes_focused, .. } => {
            *yes_focused = !*yes_focused;
            true
        }
        Modal::Win { menu_focused } => {
            *menu_focused = !*menu_focused;
            true
        }
        _ => false,
    }
}

fn focused_index(modal: &Modal) -> usize {
    match modal {
        Modal::Confirm { yes_focused, .. } => usize::from(!yes_focused),
        Modal::Win { menu_focused } => usize::from(*menu_focused),
        _ => 0,
    }
}

/// What pressing button `index` of `modal` does. Every button closes the dialog.
fn press(modal: &Modal, index: usize) -> Effect {
    match modal {
        Modal::Confirm { on_yes, .. } if index == 0 => (**on_yes).clone(),
        Modal::Win { .. } if index == 0 => Effect::Goto(Screen::GameSelect),
        Modal::Win { .. } => Effect::Goto(Screen::MainMenu),
        _ => Effect::None,
    }
}

fn tab_of(modal: &Modal) -> Option<HelpTab> {
    match modal {
        Modal::Help => Some(HelpTab::Keys),
        Modal::HowToPlay(_) => Some(HelpTab::Rules),
        Modal::CommandLine(_) => Some(HelpTab::CommandLine),
        _ => None,
    }
}

fn modal_for(tab: HelpTab, rules: Rules) -> Modal {
    match tab {
        HelpTab::Keys => Modal::Help,
        HelpTab::Rules => Modal::HowToPlay(HowToPlay::new(rules)),
        HelpTab::CommandLine => Modal::CommandLine(CommandLine::new()),
    }
}

/// The tab a key switches to from `tab`: Tab and Shift+Tab step through the tabs. Left and Right
/// step too, except on Rules where they pick the game. The Help and Rules keys jump to their tabs.
fn tab_target(tab: HelpTab, key: &KeyEvent, action: Option<Action>) -> Option<HelpTab> {
    match key.code {
        KeyCode::Tab => return Some(tab.next()),
        KeyCode::BackTab => return Some(tab.previous()),
        _ => {}
    }
    match (tab, action?) {
        (HelpTab::Rules, Action::Help) | (HelpTab::CommandLine, Action::Help) => {
            Some(HelpTab::Keys)
        }
        (HelpTab::Keys | HelpTab::CommandLine, Action::Rules) => Some(HelpTab::Rules),
        (HelpTab::Keys | HelpTab::CommandLine, Action::Right) => Some(tab.next()),
        (HelpTab::Keys | HelpTab::CommandLine, Action::Left) => Some(tab.previous()),
        _ => None,
    }
}

fn close_help(key: &KeyEvent, action: Option<Action>) -> Option<Effect> {
    let closes = matches!(key.code, KeyCode::Esc | KeyCode::Enter)
        || matches!(action, Some(Action::Select | Action::Back));
    closes.then_some(Effect::None)
}

fn clicked_tab(current: HelpTab, area: Rect, event: &MouseEvent) -> Option<HelpTab> {
    let cell = mouse::left_press(event)?;
    let row = tabs::top_row(help::geometry(area).body);
    tabs::top_tabs(row)
        .tab_at(cell)
        .filter(|tab| *tab != current)
}

fn click_help_button(area: Rect, event: &MouseEvent) -> Option<Effect> {
    let cell = mouse::left_press(event)?;
    dialog::button_at(&help::geometry(area), cell).map(|_| Effect::None)
}

/// The game the Rules tab opens on: the one on the board, Klondike without a game.
pub fn rules_of(board: &BoardScreen) -> Rules {
    board
        .session()
        .map_or(Rules::Klondike, |session| Rules::of(session.variant()))
}

/// Handles a key while `modal` is open, with Klondike as the game of the Rules tab.
#[cfg(test)]
pub fn handle(modal: &mut Modal, key: &KeyEvent, action: Option<Action>) -> Option<Effect> {
    handle_for(modal, key, action, Rules::Klondike)
}

/// Handles a key while `modal` is open. `None` keeps the modal open; `Some(effect)` closes it and
/// then applies `effect` (`Effect::None` just closes). `rules` is the game the Rules tab opens on.
pub fn handle_for(
    modal: &mut Modal,
    key: &KeyEvent,
    action: Option<Action>,
    rules: Rules,
) -> Option<Effect> {
    if let Some(tab) = tab_of(modal) {
        if let Some(target) = tab_target(tab, key, action) {
            *modal = modal_for(target, rules);
            return None;
        }
        return match modal {
            Modal::HowToPlay(state) => how_to_play::handle(state, key, action),
            Modal::CommandLine(state) => command_line::handle(state, key, action),
            _ => close_help(key, action),
        };
    }
    let action = action?;
    if matches!(
        action,
        Action::Left | Action::Right | Action::Up | Action::Down
    ) && toggle_focus(modal)
    {
        return None;
    }
    match action {
        Action::Select => Some(press(modal, focused_index(modal))),
        Action::Back => Some(Effect::None),
        Action::Pause if *modal == Modal::Pause => Some(Effect::None),
        _ => None,
    }
}

/// Handles a mouse event while `modal` is open: a click on a button presses it, any other event
/// keeps the modal open. `area` is the whole screen, the same area [`draw`] received.
pub fn handle_mouse(
    modal: &mut Modal,
    area: Rect,
    board: &BoardScreen,
    now: Instant,
    event: &MouseEvent,
) -> Option<Effect> {
    if let Some(tab) = tab_of(modal) {
        if let Some(target) = clicked_tab(tab, area, event) {
            *modal = modal_for(target, rules_of(board));
            return None;
        }
        return match modal {
            Modal::HowToPlay(state) => how_to_play::handle_mouse(state, area, event),
            Modal::CommandLine(state) => command_line::handle_mouse(state, area, event),
            _ => click_help_button(area, event),
        };
    }
    let cell = mouse::left_press(event)?;
    let index = dialog::button_at(&geometry(area, modal, board, now), cell)?;
    Some(press(modal, index))
}
