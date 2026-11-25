//! The modal layer: help, how to play, pause, win, confirmation and alert dialogs. [`draw`] paints the open
//! dialog over the screen; [`handle`] and [`handle_mouse`] turn input into the [`Effect`] that
//! closes it. Every dialog sizes itself to its content and is centered in the area it is given.

mod command_line;
#[cfg(test)]
mod command_line_tests;
mod dialog;
pub mod dock;
#[cfg(test)]
mod focus_tests;
#[cfg(test)]
mod footer_tests;
mod graphics_check;
mod guide_window;
mod help;
#[cfg(test)]
mod help_height_tests;
mod help_keys;
#[cfg(test)]
mod help_rules_tests;
mod how_to_play;
#[cfg(test)]
mod modal_fixes_tests;
mod moves_window;
mod nav;
mod pause;
mod quick_options;
#[cfg(test)]
mod render_tests;
mod search;
#[cfg(test)]
mod separator_tests;
#[cfg(test)]
mod tab_click_tests;
mod tabs;
#[cfg(test)]
mod tests;
mod win;

use std::time::Instant;

use ratatui::Frame;
use ratatui::crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Clear};

pub use self::command_line::CommandLine;
use self::dialog::{Button, Geometry, Text};
pub use self::graphics_check::GraphicsCheck;
pub use self::guide_window::{GuideWindow, TEXT_WIDTH};
pub use self::help_keys::HelpKeys;
pub use self::how_to_play::{HowToPlay, Rules};
pub use self::moves_window::MovesWindow;
pub use self::quick_options::QuickOptions;
pub use self::tabs::{HelpTab, TopTabs};
pub use self::win::PersonalBest;
use crate::action::Action;
use crate::app::{Modal, Screen, SearchCommand};
use crate::i18n::t;
use crate::layout::chrome;
use crate::mouse;
use crate::screens::board::BoardScreen;
use crate::screens::focus::{step, tab_forward};
use crate::screens::{Effect, ViewCtx};
use crate::widgets::footer::FooterMap;

fn text_of(modal: &Modal, board: &BoardScreen, now: Instant) -> Option<Text> {
    match modal {
        Modal::Help(_)
        | Modal::HowToPlay(_)
        | Modal::CommandLine(_)
        | Modal::Guide(_)
        | Modal::Moves(_)
        | Modal::Quick(_) => None,
        Modal::Pause {
            restart_focused, ..
        } => Some(pause::text(*restart_focused)),
        Modal::Search(progress) => Some(search::text(progress)),
        Modal::GraphicsCheck(check) => Some(check.text()),
        Modal::Win { focus, best } => {
            let (moves, variant) = board.session().map_or((0, None), |session| {
                (session.moves_made(), Some(session.variant()))
            });
            Some(win::text(board.elapsed(now), moves, variant, *focus, *best))
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
        (Modal::Guide(_), None) => guide_window::geometry(area),
        (Modal::Moves(state), None) => moves_window::geometry(area, state),
        (Modal::Quick(state), None) => quick_options::geometry(area, state),
        (_, None) => help::geometry(area),
    }
}

/// Draws `modal` over the screen and returns the footer buttons it drew. The pause dialog first
/// clears the board, so no card shows; the header with the clock stays visible.
pub fn draw(
    frame: &mut Frame,
    area: Rect,
    modal: &Modal,
    board: &BoardScreen,
    ctx: &ViewCtx<'_>,
) -> FooterMap {
    if matches!(modal, Modal::Pause { .. }) {
        let body = chrome(area).body;
        frame.render_widget(Clear, body);
        frame.render_widget(Block::new().style(ctx.theme.base()), body);
    }
    match (modal, text_of(modal, board, ctx.now)) {
        (_, Some(text)) => {
            let geometry = text.geometry(area);
            text.draw(frame, &geometry, ctx);
            FooterMap::default()
        }
        (Modal::HowToPlay(state), None) => how_to_play::draw(frame, area, state, ctx),
        (Modal::CommandLine(state), None) => command_line::draw(frame, area, state, ctx),
        (Modal::Guide(state), None) => guide_window::draw(frame, area, state, board, ctx),
        (Modal::Moves(state), None) => moves_window::draw(frame, area, state, ctx),
        (Modal::Quick(state), None) => {
            quick_options::draw(frame, area, state, ctx);
            FooterMap::default()
        }
        (Modal::Help(state), None) => help::draw(frame, area, state, ctx),
        (_, None) => FooterMap::default(),
    }
}

fn step_focus(modal: &mut Modal, forward: bool) -> bool {
    match modal {
        Modal::Search(progress) => {
            progress.step_focus(forward);
            true
        }
        Modal::Pause {
            restart_focused, ..
        } => {
            *restart_focused = !*restart_focused;
            true
        }
        Modal::Confirm { yes_focused, .. } => {
            *yes_focused = !*yes_focused;
            true
        }
        Modal::GraphicsCheck(check) => {
            check.yes_focused = !check.yes_focused;
            true
        }
        Modal::Win { focus, .. } => {
            *focus = step(*focus, win::BUTTONS, forward);
            true
        }
        _ => false,
    }
}

fn focused_index(modal: &Modal) -> usize {
    match modal {
        Modal::Pause {
            restart_focused, ..
        } => usize::from(*restart_focused),
        Modal::Confirm { yes_focused, .. } => usize::from(!yes_focused),
        Modal::GraphicsCheck(check) => usize::from(!check.yes_focused),
        Modal::Win { focus, .. } => *focus,
        Modal::Search(progress) => progress.focus,
        _ => 0,
    }
}

/// What pressing button `index` of `modal` does. Every button closes the dialog.
fn press(modal: &Modal, index: usize) -> Effect {
    match modal {
        Modal::Confirm { on_yes, .. } if index == 0 => (**on_yes).clone(),
        Modal::Confirm {
            on_no: Some(on_no), ..
        } => (**on_no).clone(),
        Modal::GraphicsCheck(check) => check.press(index),
        Modal::Pause { undo_allowed, .. } if index == pause::RESTART => {
            let prompt = if *undo_allowed {
                "board.confirm.restart"
            } else {
                "board.confirm.restart_no_undo"
            };
            Effect::Open(Modal::confirm(prompt, Effect::Restart))
        }
        Modal::Win { .. } if index == win::NEW_GAME => Effect::Goto(Screen::GameSelect),
        Modal::Win { .. } if index == win::SAME_SEED => Effect::SameSeed,
        Modal::Win { .. } if index == win::MENU => Effect::Goto(Screen::MainMenu),
        Modal::Search(progress) => Effect::SearchStep(search::command_at(progress, index)),
        _ => Effect::None,
    }
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

/// Handles a key while `modal` is open at the 100 by 30 size, with Klondike as the game of the
/// Rules tab.
#[cfg(test)]
pub fn handle(modal: &mut Modal, key: &KeyEvent, action: Option<Action>) -> Option<Effect> {
    handle_for(
        modal,
        key,
        action,
        Rules::Klondike,
        Rect::new(0, 0, 100, 30),
    )
}

/// Handles a key while `modal` is open. `None` keeps the modal open; `Some(effect)` closes it and
/// then applies `effect` (`Effect::None` just closes). `rules` is the game the Rules tab opens on
/// and `area` is the whole screen, the same area [`draw`] received: the Help tabs scroll by the
/// text rows that the box shows at this size.
pub fn handle_for(
    modal: &mut Modal,
    key: &KeyEvent,
    action: Option<Action>,
    rules: Rules,
    area: Rect,
) -> Option<Effect> {
    if let Modal::Guide(state) = modal {
        return guide_window::handle(state, key, action);
    }
    if let Modal::Moves(state) = modal {
        return moves_window::handle(state, key, action);
    }
    if let Modal::Quick(state) = modal {
        return quick_options::handle(state, key, action);
    }
    if nav::tab_of(modal).is_some() {
        if let Some(target) = nav::target(modal, key, action) {
            *modal = nav::modal_for(target, rules);
            return None;
        }
        return match modal {
            Modal::HowToPlay(state) => how_to_play::handle(state, key, action, area),
            Modal::CommandLine(state) => command_line::handle(state, key, action, area),
            Modal::Help(state) => help_keys::handle_key(state, key, action, area),
            _ => nav::close_help(key, action),
        };
    }
    let action = action?;
    let moves_focus = matches!(
        action,
        Action::Left | Action::Right | Action::Up | Action::Down
    ) || tab_forward(action).is_some();
    let forward = tab_forward(action).unwrap_or(!matches!(action, Action::Left | Action::Up));
    if moves_focus && step_focus(modal, forward) {
        return None;
    }
    match action {
        Action::Back if matches!(modal, Modal::Search(_)) => {
            Some(Effect::SearchStep(SearchCommand::Cancel))
        }
        Action::Select => Some(press(modal, focused_index(modal))),
        Action::Back if matches!(modal, Modal::GraphicsCheck(_) | Modal::Confirm { .. }) => {
            Some(press(modal, 1))
        }
        Action::Back => Some(Effect::None),
        Action::Pause if matches!(modal, Modal::Pause { .. }) => Some(Effect::None),
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
    if let Modal::Guide(state) = modal {
        return guide_window::handle_mouse(state, area, event);
    }
    if let Modal::Moves(state) = modal {
        return moves_window::handle_mouse(state, area, event);
    }
    if let Modal::Quick(state) = modal {
        return quick_options::handle_mouse(state, area, event);
    }
    if nav::tab_of(modal).is_some() {
        let body = geometry(area, modal, board, now).body;
        if let Some(target) = nav::clicked_tab(modal, body, event) {
            *modal = nav::modal_for(target, rules_of(board));
            return None;
        }
        return match modal {
            Modal::HowToPlay(state) => how_to_play::handle_mouse(state, area, event),
            Modal::CommandLine(state) => command_line::handle_mouse(state, area, event),
            Modal::Help(state) => help_keys::handle_mouse(state, area, event),
            _ => click_help_button(area, event),
        };
    }
    let cell = mouse::left_press(event)?;
    let index = dialog::button_at(&geometry(area, modal, board, now), cell)?;
    Some(press(modal, index))
}
