//! The full-screen setting and the pure decision about what to do with it. The decision depends only
//! on the setting, the command-line flag, whether the game owns its console window and the kind of
//! terminal; the console calls themselves are made by the application through `ConsoleDisplay`.

use serde::{Deserialize, Serialize};

/// Whether the game fills the whole screen without a frame or title bar.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FullScreen {
    /// On only when the game owns its console window, which a double-click or `--new-window` gives.
    #[default]
    Auto,
    On,
    Off,
}

/// The kinds of terminal that differ in how full screen is reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerminalKind {
    /// The classic Windows console, which `SetConsoleDisplayMode` can switch.
    ClassicConsole,
    /// Windows Terminal: no API, the player presses F11 or Alt+Enter.
    WindowsTerminal,
    /// A terminal on Linux: no safe API, the player presses F11.
    Linux,
    /// A terminal on macOS: no safe API, the player presses Ctrl+Cmd+F.
    MacOs,
}

impl TerminalKind {
    /// Localization key of the one-line hint that names this terminal's own key.
    pub const fn hint_key(self) -> &'static str {
        match self {
            Self::ClassicConsole => "board.status.full_screen_classic",
            Self::WindowsTerminal => "board.status.full_screen_windows_terminal",
            Self::Linux => "board.status.full_screen_linux",
            Self::MacOs => "board.status.full_screen_macos",
        }
    }
}

impl TerminalKind {
    /// Localization key of the zoom shortcut of this terminal, which the font size row of
    /// Settings shows instead of a value: only the classic console lets the game change the font.
    pub const fn font_keys_key(self) -> Option<&'static str> {
        match self {
            Self::ClassicConsole => None,
            Self::WindowsTerminal => Some("settings.value.font_keys_windows_terminal"),
            Self::Linux => Some("settings.value.font_keys_linux"),
            Self::MacOs => Some("settings.value.font_keys_macos"),
        }
    }
}

/// What the game does about full screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Full screen is not wanted: the console is left as it is, or leaves full screen.
    Windowed,
    /// Full screen is wanted and the terminal can be switched by the game.
    Enter,
    /// Full screen is wanted but only the player can switch the terminal: name its key.
    Hint,
}

/// Decides what to do. `flag` is `--fullscreen` (`Some(true)`) or `--no-fullscreen`
/// (`Some(false)`) and wins over the setting; `Auto` wants full screen exactly when `owns_window`.
pub const fn decide(
    setting: FullScreen,
    flag: Option<bool>,
    owns_window: bool,
    kind: TerminalKind,
) -> Decision {
    let wanted = match flag {
        Some(forced) => forced,
        None => match setting {
            FullScreen::On => true,
            FullScreen::Off => false,
            FullScreen::Auto => owns_window,
        },
    };
    match (wanted, kind) {
        (false, _) => Decision::Windowed,
        (true, TerminalKind::ClassicConsole) => Decision::Enter,
        (true, _) => Decision::Hint,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KINDS: [TerminalKind; 4] = [
        TerminalKind::ClassicConsole,
        TerminalKind::WindowsTerminal,
        TerminalKind::Linux,
        TerminalKind::MacOs,
    ];
    const SETTINGS: [FullScreen; 3] = [FullScreen::Auto, FullScreen::On, FullScreen::Off];
    const FLAGS: [Option<bool>; 3] = [None, Some(true), Some(false)];

    fn wanted(setting: FullScreen, flag: Option<bool>, owns_window: bool) -> bool {
        match flag {
            Some(forced) => forced,
            None => match setting {
                FullScreen::On => true,
                FullScreen::Off => false,
                FullScreen::Auto => owns_window,
            },
        }
    }

    fn expected(
        setting: FullScreen,
        flag: Option<bool>,
        owns_window: bool,
        kind: TerminalKind,
    ) -> Decision {
        match (wanted(setting, flag, owns_window), kind) {
            (false, _) => Decision::Windowed,
            (true, TerminalKind::ClassicConsole) => Decision::Enter,
            (true, _) => Decision::Hint,
        }
    }

    fn combinations() -> Vec<(FullScreen, Option<bool>, bool, TerminalKind)> {
        let mut all = Vec::new();
        for setting in SETTINGS {
            for flag in FLAGS {
                all.extend(KINDS.into_iter().flat_map(|kind| {
                    [false, true]
                        .into_iter()
                        .map(move |owns| (setting, flag, owns, kind))
                }));
            }
        }
        all
    }

    #[test]
    fn every_combination_follows_the_rule() {
        let all = combinations();
        assert_eq!(all.len(), 72);
        for (setting, flag, owns_window, kind) in all {
            assert_eq!(
                decide(setting, flag, owns_window, kind),
                expected(setting, flag, owns_window, kind),
                "{setting:?} {flag:?} owns={owns_window} {kind:?}"
            );
        }
    }

    #[test]
    fn auto_goes_full_screen_only_in_a_window_the_game_owns() {
        let classic = TerminalKind::ClassicConsole;
        assert_eq!(
            decide(FullScreen::Auto, None, true, classic),
            Decision::Enter
        );
        assert_eq!(
            decide(FullScreen::Auto, None, false, classic),
            Decision::Windowed
        );
    }

    #[test]
    fn the_flags_win_over_the_setting() {
        let classic = TerminalKind::ClassicConsole;
        assert_eq!(
            decide(FullScreen::Off, Some(true), false, classic),
            Decision::Enter
        );
        assert_eq!(
            decide(FullScreen::On, Some(false), true, classic),
            Decision::Windowed
        );
    }

    #[test]
    fn terminals_the_game_cannot_switch_get_a_hint_when_full_screen_is_wanted() {
        for kind in [
            TerminalKind::WindowsTerminal,
            TerminalKind::Linux,
            TerminalKind::MacOs,
        ] {
            assert_eq!(decide(FullScreen::On, None, false, kind), Decision::Hint);
            assert_eq!(
                decide(FullScreen::Off, None, true, kind),
                Decision::Windowed
            );
        }
    }

    #[test]
    fn the_setting_is_auto_by_default_and_old_values_without_it_keep_auto() {
        assert_eq!(FullScreen::default(), FullScreen::Auto);
        let parsed = toml::from_str::<crate::settings::Settings>("volume = 10");
        assert!(matches!(parsed, Ok(ref s) if s.full_screen == FullScreen::Auto));
    }

    #[test]
    fn every_value_round_trips() {
        for setting in SETTINGS {
            let settings = crate::settings::Settings {
                full_screen: setting,
                ..crate::settings::Settings::default()
            };
            let text = toml::to_string(&settings).unwrap_or_default();
            let back = toml::from_str::<crate::settings::Settings>(&text);
            assert!(matches!(back, Ok(ref s) if *s == settings));
        }
    }
}
