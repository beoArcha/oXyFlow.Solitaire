//! Raw terminal input and the key table — the single source for key mapping, help and footer.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent};

use crate::action::{Action, HelpGroup};
use crate::i18n::t;

/// One terminal event, already translated from the backend's event type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Tick,
}

/// A key together with the modifiers that must be held. Shift is ignored for character keys and
/// for Shift+Tab, which terminals report with or without the Shift flag.
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

    /// Plain-text name of the key for help and footer, for example `Enter`, `Shift+R` or
    /// `Ctrl+Z`. The names of the keys are localized through `help.key.*`.
    pub fn display(&self) -> String {
        let name = match self.code {
            KeyCode::Char(' ') => t("help.key.space").into_owned(),
            KeyCode::Char(c) if c.is_uppercase() => format!("{}+{c}", t("help.key.shift")),
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Enter => t("help.key.enter").into_owned(),
            KeyCode::Esc => t("help.key.esc").into_owned(),
            KeyCode::Up => t("help.key.up").into_owned(),
            KeyCode::Down => t("help.key.down").into_owned(),
            KeyCode::Left => t("help.key.left").into_owned(),
            KeyCode::Right => t("help.key.right").into_owned(),
            KeyCode::Tab => t("help.key.tab").into_owned(),
            KeyCode::BackTab => format!("{}+{}", t("help.key.shift"), t("help.key.tab")),
            KeyCode::F(n) => format!("F{n}"),
            other => format!("{other:?}"),
        };
        if self.modifiers.contains(KeyModifiers::CONTROL) {
            format!("{}+{}", t("help.key.ctrl"), name.to_uppercase())
        } else {
            name
        }
    }

    fn matches(&self, key: &KeyEvent) -> bool {
        let modifiers = match key.code {
            KeyCode::Char(_) | KeyCode::BackTab => key.modifiers - KeyModifiers::SHIFT,
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
pub const KEY_TABLE: [Binding; 28] = [
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
        Action::FocusNext,
        HelpGroup::Navigation,
        &[KeySpec::plain(KeyCode::Tab)],
    ),
    bind(
        Action::FocusPrevious,
        HelpGroup::Navigation,
        &[KeySpec::plain(KeyCode::BackTab)],
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
    bind(Action::Finish, HelpGroup::Board, &[KeySpec::ch('f')]),
    bind(Action::Guide, HelpGroup::Board, &[KeySpec::ch('g')]),
    bind(Action::Moves, HelpGroup::Board, &[KeySpec::ch('v')]),
    bind(Action::Options, HelpGroup::Board, &[KeySpec::ch('o')]),
    bind(Action::Restart, HelpGroup::Game, &[KeySpec::ch('R')]),
    bind(Action::NewGame, HelpGroup::Game, &[KeySpec::ch('n')]),
    bind(Action::Pause, HelpGroup::Game, &[KeySpec::ch('p')]),
    bind(Action::Menu, HelpGroup::Game, &[KeySpec::ch('m')]),
    bind(Action::CopySeed, HelpGroup::Game, &[KeySpec::ch('c')]),
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

/// The key press that performs `action` through its primary key: what a click on its footer
/// button sends through the same path as the key.
pub fn primary_key_event(action: Action) -> Option<KeyEvent> {
    binding_for(action)
        .and_then(|binding| binding.keys.first())
        .map(|spec| KeyEvent::new(spec.code, spec.modifiers))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    use crate::action::HelpGroup;
    use crate::i18n::{Locale, set_locale};

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
            (
                KeyCode::Char('c'),
                KeyModifiers::NONE,
                Some(Action::CopySeed),
            ),
            (KeyCode::Char('C'), KeyModifiers::SHIFT, None),
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
    fn the_moves_key_is_v_and_is_listed_with_the_board_moves() {
        let key = press(KeyCode::Char('v'), KeyModifiers::NONE);
        assert_eq!(action_for_key(&key), Some(Action::Moves));
        assert_eq!(primary_key(Action::Moves).as_deref(), Some("v"));
        assert_eq!(
            binding_for(Action::Moves).map(|binding| binding.group),
            Some(HelpGroup::Board)
        );
    }

    #[test]
    fn the_copy_seed_key_is_c_and_is_listed_with_the_game_keys() {
        assert_eq!(primary_key(Action::CopySeed).as_deref(), Some("c"));
        assert_eq!(
            binding_for(Action::CopySeed).map(|binding| binding.group),
            Some(HelpGroup::Game)
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
    fn tab_and_shift_tab_move_the_focus_with_or_without_the_shift_flag() {
        let tab = press(KeyCode::Tab, KeyModifiers::NONE);
        let back = press(KeyCode::BackTab, KeyModifiers::SHIFT);
        assert_eq!(action_for_key(&tab), Some(Action::FocusNext));
        assert_eq!(action_for_key(&back), Some(Action::FocusPrevious));
        let plain_back = press(KeyCode::BackTab, KeyModifiers::NONE);
        assert_eq!(action_for_key(&plain_back), Some(Action::FocusPrevious));
        assert!(!is_text_key(&tab));
        assert!(!is_text_key(&back));
    }

    #[test]
    fn tab_keys_are_named_for_help() {
        assert_eq!(primary_key(Action::FocusNext).as_deref(), Some("Tab"));
        assert_eq!(
            primary_key(Action::FocusPrevious).as_deref(),
            Some("Shift+Tab")
        );
        assert_eq!(
            binding_for(Action::FocusPrevious).map(|binding| binding.group),
            Some(HelpGroup::Navigation)
        );
    }

    #[test]
    fn the_primary_key_event_maps_back_to_its_action() {
        for action in Action::ALL {
            if let Some(key) = primary_key_event(action) {
                assert_eq!(action_for_key(&key), Some(action), "{action:?}");
            }
        }
    }

    #[test]
    fn display_names_are_plain_text() {
        assert_eq!(primary_key(Action::Select).as_deref(), Some("Enter"));
        assert_eq!(primary_key(Action::ForceQuit).as_deref(), Some("Ctrl+C"));
        assert_eq!(KeySpec::ch(' ').display(), "Space");
    }

    #[test]
    fn restart_is_shown_as_shift_r() {
        assert_eq!(primary_key(Action::Restart).as_deref(), Some("Shift+R"));
        assert_eq!(KeySpec::ctrl('z').display(), "Ctrl+Z");
    }

    #[test]
    fn key_names_follow_the_locale() {
        set_locale(Locale::PlPl);
        assert_eq!(KeySpec::plain(KeyCode::Up).display(), "Góra");
        assert_eq!(KeySpec::plain(KeyCode::Down).display(), "Dół");
        assert_eq!(KeySpec::ch(' ').display(), "Spacja");
        assert_eq!(KeySpec::plain(KeyCode::BackTab).display(), "Shift+Tab");
        set_locale(Locale::EnUs);
    }
}
