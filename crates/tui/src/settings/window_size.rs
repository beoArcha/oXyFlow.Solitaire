//! The requested size of a console window that the game owns, when windowed.

use serde::{Deserialize, Serialize};

/// The window size requested for an owned classic console window when it is not in full screen.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowSize {
    /// 130 columns by 40 rows.
    #[default]
    Small,
    /// 195 columns by 60 rows.
    Medium,
    /// 260 columns by 80 rows.
    Large,
}

impl WindowSize {
    /// Every size, in the order of the settings screen.
    pub const ALL: [WindowSize; 3] = [WindowSize::Small, WindowSize::Medium, WindowSize::Large];

    /// Requested `(columns, rows)` for this window size.
    pub const fn dimensions(self) -> (u16, u16) {
        match self {
            WindowSize::Small => (130, 40),
            WindowSize::Medium => (195, 60),
            WindowSize::Large => (260, 80),
        }
    }

    /// Localization key of the size label.
    pub const fn label_key(self) -> &'static str {
        match self {
            WindowSize::Small => "settings.window_size.small",
            WindowSize::Medium => "settings.window_size.medium",
            WindowSize::Large => "settings.window_size.large",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;

    #[test]
    fn small_is_the_default_and_old_values_without_it_keep_small() {
        assert_eq!(WindowSize::default(), WindowSize::Small);
        assert_eq!(Settings::default().window_size, WindowSize::Small);
        let parsed = toml::from_str::<Settings>("volume = 10");
        assert!(matches!(parsed, Ok(ref s) if s.window_size == WindowSize::Small));
    }

    #[test]
    fn each_size_gives_its_requested_dimensions() {
        assert_eq!(WindowSize::Small.dimensions(), (130, 40));
        assert_eq!(WindowSize::Medium.dimensions(), (195, 60));
        assert_eq!(WindowSize::Large.dimensions(), (260, 80));
    }

    #[test]
    fn every_value_round_trips() {
        for size in WindowSize::ALL {
            let settings = Settings {
                window_size: size,
                ..Settings::default()
            };
            let text = toml::to_string(&settings).unwrap_or_default();
            let back = toml::from_str::<Settings>(&text);
            assert!(matches!(back, Ok(ref s) if *s == settings));
        }
    }
}
