//! The modal layer: drawing and input of the dialogs in [`Modal`]. Stub: P3-07 replaces the
//! contents with the real help, pause, win, confirmation and alert overlays. The signatures of
//! [`draw`] and [`handle`] are the contract with `App`.

pub mod dialog;

use ratatui::Frame;
use ratatui::crossterm::event::KeyEvent;
use ratatui::layout::Rect;

use self::dialog::Button;
use crate::action::Action;
use crate::app::Modal;
use crate::i18n::t;
use crate::screens::{Effect, ViewCtx};

fn ok_button() -> Vec<Button> {
    vec![Button {
        label: t("common.dialog.ok").into_owned(),
        focused: true,
    }]
}

fn yes_no(yes_focused: bool) -> Vec<Button> {
    vec![
        Button {
            label: t("common.dialog.yes").into_owned(),
            focused: yes_focused,
        },
        Button {
            label: t("common.dialog.no").into_owned(),
            focused: !yes_focused,
        },
    ]
}

/// Draws `modal` over the screen.
pub fn draw(frame: &mut Frame, area: Rect, modal: &Modal, ctx: &ViewCtx<'_>) {
    let (title, body, buttons) = match modal {
        Modal::Help => (
            "common.dialog.help_title",
            "common.dialog.help_body",
            ok_button(),
        ),
        Modal::Pause => (
            "common.dialog.pause_title",
            "common.dialog.pause_body",
            Vec::new(),
        ),
        Modal::Win => (
            "common.dialog.win_title",
            "common.dialog.win_body",
            ok_button(),
        ),
        Modal::Alert { message } => ("common.dialog.alert_title", *message, ok_button()),
        Modal::Confirm {
            prompt,
            yes_focused,
            ..
        } => ("common.dialog.confirm_title", *prompt, yes_no(*yes_focused)),
    };
    dialog::draw(
        frame,
        area,
        &t(title),
        &[t(body).into_owned()],
        &buttons,
        ctx,
    );
}

/// Handles a key while `modal` is open. `None` keeps the modal open; `Some(effect)` closes it and
/// then applies `effect` (`Effect::None` just closes).
pub fn handle(modal: &mut Modal, _key: &KeyEvent, action: Option<Action>) -> Option<Effect> {
    let action = action?;
    match modal {
        Modal::Confirm {
            on_yes,
            yes_focused,
            ..
        } => match action {
            Action::Left | Action::Right | Action::Up | Action::Down => {
                *yes_focused = !*yes_focused;
                None
            }
            Action::Select if *yes_focused => Some((**on_yes).clone()),
            Action::Select | Action::Back => Some(Effect::None),
            _ => None,
        },
        _ => {
            matches!(action, Action::Select | Action::Back | Action::Pause).then_some(Effect::None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::{KeyCode, KeyModifiers};

    fn key() -> KeyEvent {
        KeyEvent::new(KeyCode::Null, KeyModifiers::NONE)
    }

    #[test]
    fn simple_modals_close_on_select_and_back() {
        for action in [Action::Select, Action::Back] {
            let mut modal = Modal::Help;
            assert_eq!(handle(&mut modal, &key(), Some(action)), Some(Effect::None));
        }
        let mut modal = Modal::Help;
        assert_eq!(handle(&mut modal, &key(), Some(Action::Hint)), None);
        assert_eq!(handle(&mut modal, &key(), None), None);
    }

    #[test]
    fn confirm_defaults_to_no_and_runs_the_effect_only_on_yes() {
        let mut modal = Modal::confirm("common.dialog.win_body", Effect::Quit);
        assert_eq!(
            handle(&mut modal, &key(), Some(Action::Select)),
            Some(Effect::None)
        );

        let mut modal = Modal::confirm("common.dialog.win_body", Effect::Quit);
        assert_eq!(handle(&mut modal, &key(), Some(Action::Left)), None);
        assert_eq!(
            handle(&mut modal, &key(), Some(Action::Select)),
            Some(Effect::Quit)
        );
    }

    #[test]
    fn confirm_back_cancels() {
        let mut modal = Modal::confirm("common.dialog.win_body", Effect::Quit);
        assert_eq!(
            handle(&mut modal, &key(), Some(Action::Back)),
            Some(Effect::None)
        );
    }
}
