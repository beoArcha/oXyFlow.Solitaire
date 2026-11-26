//! The board aids: visual hints and auto-finish. The settings keep their defaults for a new game;
//! the running game keeps its own copy (the quick options), which starts as the defaults.

use super::{AuraMode, Settings};
use crate::i18n::{t, tf};
use crate::layout::InterfaceSize;

/// The aids as one value: what the player may switch on the board and set as default in Settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Aids {
    pub aura: AuraMode,
    pub target_marks: bool,
    pub guide_panel: bool,
    pub moves_panel: bool,
    pub auto_finish: bool,
    pub quick_move: bool,
}

/// One aid, as a row of the "Hints and help" group and of the quick options dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aid {
    Aura,
    TargetMarks,
    GuidePanel,
    MovesPanel,
    AutoFinish,
    QuickMove,
}

impl Aid {
    /// Every aid, top to bottom.
    pub const ALL: [Aid; 6] = [
        Aid::Aura,
        Aid::TargetMarks,
        Aid::GuidePanel,
        Aid::MovesPanel,
        Aid::AutoFinish,
        Aid::QuickMove,
    ];

    /// Localization key of the row label.
    pub const fn label_key(self) -> &'static str {
        match self {
            Aid::Aura => "settings.row.aura",
            Aid::TargetMarks => "settings.row.target_marks",
            Aid::GuidePanel => "settings.row.guide_panel",
            Aid::MovesPanel => "settings.row.moves_panel",
            Aid::AutoFinish => "settings.row.auto_finish",
            Aid::QuickMove => "settings.row.quick_move",
        }
    }

    /// Localization key of the sentence that explains the aid.
    pub const fn description_key(self) -> &'static str {
        match self {
            Aid::Aura => "settings.desc.aura",
            Aid::TargetMarks => "settings.desc.target_marks",
            Aid::GuidePanel => "settings.desc.guide_panel",
            Aid::MovesPanel => "settings.desc.moves_panel",
            Aid::AutoFinish => "settings.desc.auto_finish",
            Aid::QuickMove => "settings.desc.quick_move",
        }
    }

    /// Whether the aid is a side panel that needs a wide screen.
    pub const fn is_panel(self) -> bool {
        matches!(self, Aid::GuidePanel | Aid::MovesPanel)
    }

    /// Whether the aid is on; the aura counts as on when it is full.
    pub const fn is_on(self, aids: &Aids) -> bool {
        match self {
            Aid::Aura => matches!(aids.aura, AuraMode::Full),
            Aid::TargetMarks => aids.target_marks,
            Aid::GuidePanel => aids.guide_panel,
            Aid::MovesPanel => aids.moves_panel,
            Aid::AutoFinish => aids.auto_finish,
            Aid::QuickMove => aids.quick_move,
        }
    }

    /// The value as the player reads it.
    pub fn value_text(self, aids: &Aids) -> String {
        let key = match (self, self.is_on(aids)) {
            (Aid::Aura, true) => "settings.aura.full",
            (Aid::Aura, false) => "settings.aura.focus_only",
            (_, true) => "settings.value.on",
            (_, false) => "settings.value.off",
        };
        t(key).into_owned()
    }

    /// The value for a screen of `columns` at the default interface scale: a side panel that is
    /// on says which width it needs when the screen is too narrow for it.
    pub fn value_for(self, aids: &Aids, columns: u16) -> String {
        self.value_for_size(aids, columns, InterfaceSize::default())
    }

    /// The value for a screen of `columns` at `interface_size`: a side panel that is on says which
    /// width it needs when the screen is too narrow for it.
    pub fn value_for_size(
        self,
        aids: &Aids,
        columns: u16,
        interface_size: InterfaceSize,
    ) -> String {
        let value = self.value_text(aids);
        let needed = interface_size.scale().chrome.standard_min_width;
        if self.is_panel() && self.is_on(aids) && columns < needed {
            return tf(
                "settings.value.needs_columns",
                &[
                    ("value", value.as_str()),
                    ("needed", &needed.to_string()),
                    ("now", &columns.to_string()),
                ],
            );
        }
        value
    }

    /// The aids after this one was switched; the others stay as they were.
    pub fn toggled(self, aids: &Aids) -> Aids {
        let mut next = *aids;
        match self {
            Aid::Aura => {
                next.aura = match aids.aura {
                    AuraMode::Full => AuraMode::FocusOnly,
                    AuraMode::FocusOnly => AuraMode::Full,
                };
            }
            Aid::TargetMarks => next.target_marks = !aids.target_marks,
            Aid::GuidePanel => next.guide_panel = !aids.guide_panel,
            Aid::MovesPanel => next.moves_panel = !aids.moves_panel,
            Aid::AutoFinish => next.auto_finish = !aids.auto_finish,
            Aid::QuickMove => next.quick_move = !aids.quick_move,
        }
        next
    }
}

impl Settings {
    /// The defaults of the board aids.
    pub fn aids(&self) -> Aids {
        Aids {
            aura: self.aura,
            target_marks: self.target_marks,
            guide_panel: self.guide_panel,
            moves_panel: self.moves_panel,
            auto_finish: self.auto_finish,
            quick_move: self.quick_move,
        }
    }

    /// These settings with the defaults of the board aids replaced by `aids`.
    pub fn with_aids(&self, aids: Aids) -> Self {
        Self {
            aura: aids.aura,
            target_marks: aids.target_marks,
            guide_panel: aids.guide_panel,
            moves_panel: aids.moves_panel,
            auto_finish: aids.auto_finish,
            quick_move: aids.quick_move,
            ..self.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_aids_round_trip_through_the_settings() {
        let aids = Aids {
            aura: AuraMode::FocusOnly,
            target_marks: false,
            guide_panel: false,
            moves_panel: false,
            auto_finish: false,
            quick_move: false,
        };
        let settings = Settings::default().with_aids(aids);
        assert_eq!(settings.aids(), aids);
        assert_eq!(settings.volume, Settings::default().volume);
    }

    #[test]
    fn every_aid_switches_only_itself_and_back() {
        let base = Settings::default().aids();
        for aid in Aid::ALL {
            let once = aid.toggled(&base);
            assert_ne!(once, base, "{aid:?}");
            assert_eq!(aid.is_on(&once), !aid.is_on(&base), "{aid:?}");
            assert_eq!(aid.toggled(&once), base, "{aid:?}");
        }
    }
}
