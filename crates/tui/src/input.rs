//! Raw terminal input and the key table — the single source for key mapping, help and footer.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent};

use crate::action::{Action, HelpGroup};

/// One terminal event, already translated from the backend's event type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Tick,
}

/// A key together with the modifiers that must be held. Shift is ignored for character keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeySpec {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeySpec {
    const fn plain(code: KeyCode) -> Self {
        Self {
            code,
            modifiers: KeyModifiers::NONE,
        }
    }

    const fn ch(c: char) -> Self {
        Self::plain(KeyCode::Char(c))
    }

    const fn ctrl(c: char) -> Self {
        Self {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::CONTROL,
        }
    }

    /// Plain-text name of the key for help and footer, for example `Enter` or `Ctrl+Z`.
    pub fn display(&self) -> String {
        let name = match self.code {
            KeyCode::Char(' ') => "Space".to_owned(),
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Enter => "Enter".to_owned(),
            KeyCode::Esc => "Esc".to_owned(),
            KeyCode::Up => "Up".to_owned(),
            KeyCode::Down => "Down".to_owned(),
            KeyCode::Left => "Left".to_owned(),
            KeyCode::Right => "Right".to_owned(),
            KeyCode::F(n) => format!("F{n}"),
            other => format!("{other:?}"),
        };
        if self.modifiers.contains(KeyModifiers::CONTROL) {
            format!("Ctrl+{}", name.to_uppercase())
        } else {
            name
        }
    }

    fn matches(&self, key: &KeyEvent) -> bool {
        let modifiers = match key.code {
            KeyCode::Char(_) => key.modifiers - KeyModifiers::SHIFT,
            _ => key.modifiers,
        };
        self.code == key.code && self.modifiers == modifiers
    }
}

/// One row of the key table: an action, its keys and where help lists it.
#[derive(Debug, Clone, Copy)]
pub struct Binding {
    pub action: Action,
    pub keys: &'static [KeySpec],
    pub group: HelpGroup,
}

const fn bind(action: Action, group: HelpGroup, keys: &'static [KeySpec]) -> Binding {
    Binding {
        action,
        keys,
        group,
    }
}

/// Every key binding of the application. Order is the order of the help overlay.
pub const KEY_TABLE: [Binding; 22] = [
    bind(
        Action::Up,
        HelpGroup::Navigation,
        &[KeySpec::plain(KeyCode::Up), KeySpec::ch('k')],
    ),
    bind(
        Action::Down,
        HelpGroup::Navigation,
        &[KeySpec::plain(KeyCode::Down), KeySpec::ch('j')],
    ),
    bind(
        Action::Left,
        HelpGroup::Navigation,
        &[KeySpec::plain(KeyCode::Left)],
    ),
    bind(
        Action::Right,
        HelpGroup::Navigation,
        &[KeySpec::plain(KeyCode::Right)],
    ),
    bind(
        Action::Select,
        HelpGroup::Navigation,
        &[KeySpec::plain(KeyCode::Enter), KeySpec::ch(' ')],
    ),
    bind(
        Action::Back,
        HelpGroup::Navigation,
        &[KeySpec::plain(KeyCode::Esc)],
    ),
    bind(Action::Stock, HelpGroup::Board, &[KeySpec::ch('s')]),
    bind(Action::QuickMove, HelpGroup::Board, &[KeySpec::ch('a')]),
    bind(Action::RunMore, HelpGroup::Board, &[KeySpec::ch('+')]),
    bind(Action::RunLess, HelpGroup::Board, &[KeySpec::ch('-')]),
    bind(
        Action::Undo,
        HelpGroup::Board,
        &[KeySpec::ch('u'), KeySpec::ctrl('z')],
    ),
    bind(
        Action::Redo,
        HelpGroup::Board,
        &[KeySpec::ch('r'), KeySpec::ctrl('y')],
    ),
    bind(Action::Hint, HelpGroup::Board, &[KeySpec::ch('h')]),
    bind(Action::Guide, HelpGroup::Board, &[KeySpec::ch('g')]),
    bind(Action::Restart, HelpGroup::Game, &[KeySpec::ch('R')]),
    bind(Action::NewGame, HelpGroup::Game, &[KeySpec::ch('n')]),
    bind(Action::Pause, HelpGroup::Game, &[KeySpec::ch('p')]),
    bind(Action::Menu, HelpGroup::Game, &[KeySpec::ch('m')]),
    bind(Action::Quit, HelpGroup::General, &[KeySpec::ch('q')]),
    bind(Action::ForceQuit, HelpGroup::General, &[KeySpec::ctrl('c')]),
    bind(
        Action::Help,
        HelpGroup::General,
        &[KeySpec::ch('?'), KeySpec::plain(KeyCode::F(1))],
    ),
    bind(Action::Rules, HelpGroup::General, &[KeySpec::ch('i')]),
];

/// The action bound to a key press, if any. Key releases never map to an action.
pub fn action_for_key(key: &KeyEvent) -> Option<Action> {
    if key.kind == KeyEventKind::Release {
        return None;
    }
    KEY_TABLE
        .iter()
        .find(|binding| binding.keys.iter().any(|spec| spec.matches(key)))
        .map(|binding| binding.action)
}

/// Whether a key press types a character: a character key without Ctrl. Such a key is text for a
/// text field, so it must not reach the global shortcuts.
pub fn is_text_key(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char(_)) && !key.modifiers.contains(KeyModifiers::CONTROL)
}

/// The binding of an action in the key table.
pub fn binding_for(action: Action) -> Option<&'static Binding> {
    KEY_TABLE.iter().find(|binding| binding.action == action)
}

/// The primary key name of an action, for example `Enter` for [`Action::Select`].
pub fn primary_key(action: Action) -> Option<String> {
    binding_for(action)
        .and_then(|binding| binding.keys.first())
        .map(KeySpec::display)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    use crate::action::HelpGroup;

    fn press(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn every_action_has_exactly_one_binding_with_a_key() {
        for action in Action::ALL {
            let rows = KEY_TABLE.iter().filter(|b| b.action == action).count();
            assert_eq!(rows, 1, "{action:?}");
            assert!(binding_for(action).is_some_and(|b| !b.keys.is_empty()));
        }
    }

    #[test]
    fn no_key_is_bound_twice() {
        let mut seen = HashSet::new();
        for spec in KEY_TABLE.iter().flat_map(|b| b.keys.iter()) {
            assert!(seen.insert(spec.display()), "duplicate {}", spec.display());
        }
    }

    #[test]
    fn keys_map_to_their_actions() {
        let cases = [
            (KeyCode::Enter, KeyModifiers::NONE, Some(Action::Select)),
            (KeyCode::Char('j'), KeyModifiers::NONE, Some(Action::Down)),
            (
                KeyCode::Char('c'),
                KeyModifiers::CONTROL,
                Some(Action::ForceQuit),
            ),
            (
                KeyCode::Char('R'),
                KeyModifiers::SHIFT,
                Some(Action::Restart),
            ),
            (KeyCode::Char('?'), KeyModifiers::SHIFT, Some(Action::Help)),
            (KeyCode::Char('c'), KeyModifiers::NONE, None),
            (KeyCode::Char('q'), KeyModifiers::CONTROL, None),
        ];
        for (code, modifiers, expected) in cases {
            assert_eq!(
                action_for_key(&press(code, modifiers)),
                expected,
                "{code:?}"
            );
        }
    }

    #[test]
    fn the_rules_key_is_i_and_is_listed_with_the_general_help() {
        let key = press(KeyCode::Char('i'), KeyModifiers::NONE);
        assert_eq!(action_for_key(&key), Some(Action::Rules));
        assert_eq!(primary_key(Action::Rules).as_deref(), Some("i"));
        assert_eq!(
            binding_for(Action::Rules).map(|binding| binding.group),
            Some(HelpGroup::General)
        );
    }

    #[test]
    fn the_guide_key_is_g_and_is_listed_with_the_board_moves() {
        let key = press(KeyCode::Char('g'), KeyModifiers::NONE);
        assert_eq!(action_for_key(&key), Some(Action::Guide));
        assert_eq!(primary_key(Action::Guide).as_deref(), Some("g"));
        assert_eq!(
            binding_for(Action::Guide).map(|binding| binding.group),
            Some(HelpGroup::Board)
        );
    }

    #[test]
    fn typed_characters_are_text_and_control_keys_are_not() {
        assert!(is_text_key(&press(KeyCode::Char('?'), KeyModifiers::NONE)));
        assert!(is_text_key(&press(KeyCode::Char('I'), KeyModifiers::SHIFT)));
        assert!(!is_text_key(&press(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL
        )));
        assert!(!is_text_key(&press(KeyCode::Enter, KeyModifiers::NONE)));
        assert!(!is_text_key(&press(KeyCode::F(1), KeyModifiers::NONE)));
        assert!(!is_text_key(&press(KeyCode::Backspace, KeyModifiers::NONE)));
    }

    #[test]
    fn key_release_maps_to_nothing() {
        let mut key = press(KeyCode::Enter, KeyModifiers::NONE);
        key.kind = KeyEventKind::Release;
        assert_eq!(action_for_key(&key), None);
    }

    #[test]
    fn display_names_are_plain_text() {
        assert_eq!(primary_key(Action::Select).as_deref(), Some("Enter"));
        assert_eq!(primary_key(Action::ForceQuit).as_deref(), Some("Ctrl+C"));
        assert_eq!(KeySpec::ch(' ').display(), "Space");
    }
}
