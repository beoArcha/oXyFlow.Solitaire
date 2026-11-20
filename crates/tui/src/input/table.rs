//! The key table and bindings: mapping keys to actions, help display and footer labels.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::action::{Action, HelpGroup};
use crate::i18n::t;

/// A key together with the modifiers that must be held. Shift is ignored for character keys and
/// for Shift+Tab, which terminals report with or without the Shift flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeySpec {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeySpec {
    pub const fn plain(code: KeyCode) -> Self {
        Self {
            code,
            modifiers: KeyModifiers::NONE,
        }
    }

    pub const fn ch(c: char) -> Self {
        Self::plain(KeyCode::Char(c))
    }

    pub const fn ctrl(c: char) -> Self {
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

    pub(crate) fn matches(&self, key: &KeyEvent) -> bool {
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
pub const KEY_TABLE: [Binding; 33] = [
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
    bind(Action::ZoomIn, HelpGroup::Display, &[KeySpec::ch(']')]),
    bind(Action::ZoomOut, HelpGroup::Display, &[KeySpec::ch('[')]),
    bind(Action::ZoomAuto, HelpGroup::Display, &[KeySpec::ch('0')]),
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
    bind(Action::Dock, HelpGroup::General, &[KeySpec::ch('d')]),
    bind(
        Action::DockFocus,
        HelpGroup::General,
        &[KeySpec::plain(KeyCode::F(6))],
    ),
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
