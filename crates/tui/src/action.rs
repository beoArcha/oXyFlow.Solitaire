//! The semantic vocabulary between input and application: what the player wants, not which key.

/// A semantic command. Keys and mouse gestures are both translated into these.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
    FocusNext,
    FocusPrevious,
    Select,
    Back,
    Quit,
    ForceQuit,
    Help,
    Rules,
    Guide,
    Moves,
    Options,
    Stock,
    QuickMove,
    RunMore,
    RunLess,
    Undo,
    Redo,
    Hint,
    Finish,
    Restart,
    NewGame,
    Pause,
    Menu,
    CopySeed,
}

impl Action {
    /// Every action, in the order the help overlay lists them.
    pub const ALL: [Action; 28] = [
        Action::Up,
        Action::Down,
        Action::Left,
        Action::Right,
        Action::FocusNext,
        Action::FocusPrevious,
        Action::Select,
        Action::Back,
        Action::Stock,
        Action::QuickMove,
        Action::RunMore,
        Action::RunLess,
        Action::Undo,
        Action::Redo,
        Action::Hint,
        Action::Finish,
        Action::Guide,
        Action::Moves,
        Action::Options,
        Action::Restart,
        Action::NewGame,
        Action::Pause,
        Action::Menu,
        Action::CopySeed,
        Action::Quit,
        Action::ForceQuit,
        Action::Help,
        Action::Rules,
    ];

    /// Localization key of the short name shown in help and in the footer.
    pub const fn label_key(self) -> &'static str {
        match self {
            Action::Up => "help.action.up",
            Action::Down => "help.action.down",
            Action::Left => "help.action.left",
            Action::Right => "help.action.right",
            Action::FocusNext => "help.action.focus_next",
            Action::FocusPrevious => "help.action.focus_previous",
            Action::Select => "help.action.select",
            Action::Back => "help.action.back",
            Action::Quit => "help.action.quit",
            Action::ForceQuit => "help.action.force_quit",
            Action::Help => "help.action.help",
            Action::Rules => "help.action.rules",
            Action::Guide => "help.action.guide",
            Action::Moves => "help.action.moves",
            Action::Options => "help.action.options",
            Action::Stock => "help.action.stock",
            Action::QuickMove => "help.action.quick_move",
            Action::RunMore => "help.action.run_more",
            Action::RunLess => "help.action.run_less",
            Action::Undo => "help.action.undo",
            Action::Redo => "help.action.redo",
            Action::Hint => "help.action.hint",
            Action::Finish => "help.action.finish",
            Action::Restart => "help.action.restart",
            Action::NewGame => "help.action.new_game",
            Action::Pause => "help.action.pause",
            Action::Menu => "help.action.menu",
            Action::CopySeed => "help.action.copy_seed",
        }
    }
}

/// Topic under which the help overlay lists an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HelpGroup {
    Navigation,
    Board,
    Game,
    General,
}

impl HelpGroup {
    /// Groups in display order.
    pub const ALL: [HelpGroup; 4] = [
        HelpGroup::Navigation,
        HelpGroup::Board,
        HelpGroup::Game,
        HelpGroup::General,
    ];

    /// Localization key of the group heading.
    pub const fn title_key(self) -> &'static str {
        match self {
            HelpGroup::Navigation => "help.group.navigation",
            HelpGroup::Board => "help.group.board",
            HelpGroup::Game => "help.group.game",
            HelpGroup::General => "help.group.general",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn all_lists_every_action_once() {
        let unique: HashSet<_> = Action::ALL.iter().collect();
        assert_eq!(unique.len(), Action::ALL.len());
    }
}
