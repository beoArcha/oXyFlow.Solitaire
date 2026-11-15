//! The modal layer: help, pause, win, confirmation and alert dialogs. [`draw`] paints the open
//! dialog over the screen; [`handle`] and [`handle_mouse`] turn input into the [`Effect`] that
//! closes it. Every dialog sizes itself to its content and is centered in the area it is given.

mod dialog;
mod help;
mod pause;
#[cfg(test)]
mod render_tests;
#[cfg(test)]
mod tests;
mod win;

use std::time::Instant;

use ratatui::Frame;
use ratatui::crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Clear};

use self::dialog::{Button, Geometry, Text};
use crate::action::Action;
use crate::app::{Modal, Screen};
use crate::i18n::t;
use crate::layout::chrome;
use crate::mouse;
use crate::screens::board::BoardScreen;
use crate::screens::{Effect, ViewCtx};

fn text_of(modal: &Modal, board: &BoardScreen, now: Instant) -> Option<Text> {
    match modal {
        Modal::Help => None,
        Modal::Pause => Some(pause::text()),
        Modal::Win { menu_focused } => Some(win::text(board.elapsed(now), *menu_focused)),
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
    match text_of(modal, board, now) {
        Some(text) => text.geometry(area),
        None => help::geometry(area),
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
    match text_of(modal, board, ctx.now) {
        Some(text) => {
            let geometry = text.geometry(area);
            text.draw(frame, &geometry, ctx);
        }
        None => help::draw(frame, area, ctx),
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

/// Handles a key while `modal` is open. `None` keeps the modal open; `Some(effect)` closes it and
/// then applies `effect` (`Effect::None` just closes).
pub fn handle(modal: &mut Modal, _key: &KeyEvent, action: Option<Action>) -> Option<Effect> {
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
    let cell = mouse::left_press(event)?;
    let index = dialog::button_at(&geometry(area, modal, board, now), cell)?;
    Some(press(modal, index))
}
