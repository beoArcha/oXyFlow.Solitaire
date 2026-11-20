//! The process priority the player asks of the operating system: five levels, never real time.

use serde::{Deserialize, Serialize};

/// How much processor time the game asks for against other programs. `Normal` leaves the system's
/// own choice.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessPriority {
    Highest,
    High,
    #[default]
    Normal,
    Low,
    Lowest,
}

impl ProcessPriority {
    /// Every level, in the order of the settings screen.
    pub const ALL: [ProcessPriority; 5] = [
        ProcessPriority::Highest,
        ProcessPriority::High,
        ProcessPriority::Normal,
        ProcessPriority::Low,
        ProcessPriority::Lowest,
    ];

    /// The name the `--priority` flag takes.
    pub const fn flag_name(self) -> &'static str {
        match self {
            ProcessPriority::Highest => "highest",
            ProcessPriority::High => "high",
            ProcessPriority::Normal => "normal",
            ProcessPriority::Low => "low",
            ProcessPriority::Lowest => "lowest",
        }
    }

    /// The level a `--priority` value names, `None` for anything else.
    pub fn from_flag_name(name: &str) -> Option<ProcessPriority> {
        Self::ALL
            .into_iter()
            .find(|level| level.flag_name() == name)
    }

    /// Localization key of the level name.
    pub const fn label_key(self) -> &'static str {
        match self {
            ProcessPriority::Highest => "settings.priority.highest",
            ProcessPriority::High => "settings.priority.high",
            ProcessPriority::Normal => "settings.priority.normal",
            ProcessPriority::Low => "settings.priority.low",
            ProcessPriority::Lowest => "settings.priority.lowest",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_flag_name_round_trips_and_nothing_else_parses() {
        for level in ProcessPriority::ALL {
            assert_eq!(
                ProcessPriority::from_flag_name(level.flag_name()),
                Some(level)
            );
        }
        for bad in ["", "realtime", "Normal", "HIGH", "5"] {
            assert_eq!(ProcessPriority::from_flag_name(bad), None, "{bad}");
        }
    }

    #[test]
    fn the_default_is_normal() {
        assert_eq!(ProcessPriority::default(), ProcessPriority::Normal);
    }
}
