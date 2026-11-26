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
pub mod guide_window;
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
mod no_moves;
#[cfg(test)]
mod no_moves_tests;
mod pause;
mod quick_options;
#[cfg(test)]
mod render_tests;
mod search;
#[cfg(test)]
mod separator_tests;
#[cfg(test)]
mod tab_click_tests;
#[cfg(test)]
mod tab_focus_tests;
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
pub use self::guide_window::GuideWindow;
pub use self::help_keys::HelpKeys;
pub use self::how_to_play::{HowToPlay, Rules};
pub use self::moves_window::MovesWindow;
pub use self::quick_options::QuickOptions;
pub use self::tabs::{HelpTab, TopTabs};
pub use self::win::PersonalBest;
use crate::action::Action;
use crate::app::{Modal, Screen, SearchCommand};
use crate::i18n::t;
use crate::layout::{InterfaceSize, chrome};
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
        Modal::NoMoves { can_undo, focus } => Some(no_moves::text(*can_undo, *focus)),
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

#[cfg(test)]
fn geometry(area: Rect, modal: &Modal, board: &BoardScreen, now: Instant) -> Geometry {
    geometry_for(area, modal, board, now, InterfaceSize::default())
}

fn geometry_for(
    area: Rect,
    modal: &Modal,
    board: &BoardScreen,
    now: Instant,
    size: InterfaceSize,
) -> Geometry {
    match (modal, text_of(modal, board, now)) {
        (_, Some(text)) => text.geometry_for(area, size),
        (Modal::HowToPlay(_), None) => how_to_play::geometry_for(area, size),
        (Modal::Guide(_), None) => guide_window::geometry_for(area, size),
        (Modal::Moves(state), None) => moves_window::geometry_for(area, state, size),
        (Modal::Quick(state), None) => quick_options::geometry_for(area, state, size),
        (_, None) => help::geometry_for(area, size),
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
            let geometry = text.geometry_for(area, ctx.settings.interface_size);
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
        Modal::NoMoves { can_undo, focus } => {
            *focus = step(*focus, no_moves::button_count(*can_undo), forward);
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
        Modal::NoMoves { focus, .. } => *focus,
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
        Modal::NoMoves { can_undo, .. } => no_moves::effect_at(*can_undo, index),
        Modal::Search(progress) => Effect::SearchStep(search::command_at(progress, index)),
        _ => Effect::None,
    }
}

fn click_help_button(area: Rect, event: &MouseEvent, size: InterfaceSize) -> Option<Effect> {
    let cell = mouse::left_press(event)?;
    dialog::button_at(&help::geometry_for(area, size), cell).map(|_| Effect::None)
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

/// Handles a key while `modal` is open at the default scale.
#[cfg(test)]
pub fn handle_for(
    modal: &mut Modal,
    key: &KeyEvent,
    action: Option<Action>,
    rules: Rules,
    area: Rect,
) -> Option<Effect> {
    handle_for_size(modal, key, action, rules, area, InterfaceSize::default())
}

/// Handles a key while `modal` is open. `None` keeps the modal open; `Some(effect)` closes it and
/// then applies `effect` (`Effect::None` just closes). `rules` is the game the Rules tab opens on
/// and `area` is the whole screen, the same area [`draw`] received: the Help tabs scroll by the
/// text rows that the box shows at this size and scale.
pub fn handle_for_size(
    modal: &mut Modal,
    key: &KeyEvent,
    action: Option<Action>,
    rules: Rules,
    area: Rect,
    size: InterfaceSize,
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
            Modal::HowToPlay(state) => how_to_play::handle_for(state, key, action, area, size),
            Modal::CommandLine(state) => command_line::handle_for(state, key, action, area, size),
            Modal::Help(state) => help_keys::handle_key_for(state, key, action, area, size),
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

/// Handles a mouse event at the default interface scale.
#[cfg(test)]
pub fn handle_mouse(
    modal: &mut Modal,
    area: Rect,
    board: &BoardScreen,
    now: Instant,
    event: &MouseEvent,
) -> Option<Effect> {
    handle_mouse_for(modal, area, board, now, event, InterfaceSize::default())
}

/// Handles a mouse event while `modal` is open: a click on a button presses it, any other event
/// keeps the modal open. `area` is the whole screen, the same area [`draw`] received.
pub fn handle_mouse_for(
    modal: &mut Modal,
    area: Rect,
    board: &BoardScreen,
    now: Instant,
    event: &MouseEvent,
    size: InterfaceSize,
) -> Option<Effect> {
    if let Modal::Guide(state) = modal {
        return guide_window::handle_mouse_for(state, area, event, size);
    }
    if let Modal::Moves(state) = modal {
        return moves_window::handle_mouse_for(state, area, event, size);
    }
    if let Modal::Quick(state) = modal {
        return quick_options::handle_mouse_for(state, area, event, size);
    }
    if nav::tab_of(modal).is_some() {
        let body = geometry_for(area, modal, board, now, size).body;
        if let Some(target) = nav::clicked_tab(modal, body, event) {
            *modal = nav::modal_for(target, rules_of(board));
            return None;
        }
        return match modal {
            Modal::HowToPlay(state) => how_to_play::handle_mouse_for(state, area, event, size),
            Modal::CommandLine(state) => command_line::handle_mouse_for(state, area, event, size),
            Modal::Help(state) => help_keys::handle_mouse_for(state, area, event, size),
            _ => click_help_button(area, event, size),
        };
    }
    let cell = mouse::left_press(event)?;
    let index = dialog::button_at(&geometry_for(area, modal, board, now, size), cell)?;
    Some(press(modal, index))
}

/// Focused element target and optional index inside `modal`.
#[cfg(feature = "dev")]
pub(crate) fn dev_focus(modal: &Modal) -> (&'static str, Option<usize>) {
    match modal {
        Modal::Help(_) => ("help_keys", None),
        Modal::HowToPlay(_) => ("help_rules", None),
        Modal::CommandLine(_) => ("help_command_line", None),
        Modal::Guide(_) => ("close", None),
        Modal::Moves(s) => ("moves_entry", Some(s.focus)),
        Modal::Quick(s) => ("quick_option", Some(s.focus)),
        Modal::Pause {
            restart_focused, ..
        } => (
            if *restart_focused {
                "restart"
            } else {
                "continue"
            },
            None,
        ),
        Modal::Confirm { yes_focused, .. } => (if *yes_focused { "yes" } else { "no" }, None),
        Modal::GraphicsCheck(c) => (if c.yes_focused { "yes" } else { "no" }, None),
        Modal::Alert { .. } => ("ok", None),
        Modal::Search(p) => ("search_button", Some(p.focus)),
        Modal::Win { focus, .. } => (
            ["new_game", "same_seed", "menu"]
                .get(*focus)
                .copied()
                .unwrap_or("menu"),
            Some(*focus),
        ),
        Modal::NoMoves {
            can_undo, focus, ..
        } => {
            let names: &[&str] = if *can_undo {
                &["undo", "restart", "new_game", "menu"]
            } else {
                &["restart", "new_game", "menu"]
            };
            (names.get(*focus).copied().unwrap_or("menu"), Some(*focus))
        }
    }
}

/// Outer frame rectangle of `modal` in `area` using the exact layout functions drawing uses.
#[cfg(feature = "dev")]
pub(crate) fn rect_for(
    area: Rect,
    modal: &Modal,
    board: &BoardScreen,
    now: Instant,
    dock_side: crate::settings::DockSide,
    size: InterfaceSize,
) -> Option<Rect> {
    if crate::widgets::too_small::is_too_small(area) {
        return None;
    }
    if dock::dock_view::is_dockable(modal)
        && let Some((rect, _)) = dock::split_for(area, dock_side, size)
    {
        return Some(rect);
    }
    Some(geometry_for(area, modal, board, now, size).frame)
}
