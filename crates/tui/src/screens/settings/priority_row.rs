//! The process priority row of the Interface group: the level, and why the system did not take it.

use crate::app::FontFacts;
use crate::i18n::t;
use crate::settings::Settings;

/// The value of the row: the level, followed by the system's refusal when there was one.
pub(super) fn value(settings: &Settings, facts: FontFacts) -> String {
    let level = t(settings.process_priority.label_key());
    match facts.priority.note_key() {
        Some(note) => format!("{level} ({})", t(note)),
        None => level.into_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::PriorityOutcome;
    use crate::i18n::{Locale, set_locale};
    use crate::screens::settings::options::{Field, Group, Step};
    use crate::screens::settings::values::{changed, line, value_text};
    use crate::settings::ProcessPriority;

    #[test]
    fn the_row_is_in_the_interface_group_so_a_change_is_previewed() {
        assert_eq!(Field::Priority.group(), Group::Performance);
        assert!(Field::Priority.is_interface());
    }

    #[test]
    fn a_step_walks_the_five_levels_and_wraps() {
        let mut settings = Settings::default();
        let mut seen = Vec::new();
        for _ in 0..5 {
            settings = changed(Field::Priority, &settings, Step::Next, &[]);
            seen.push(settings.process_priority);
        }
        assert_eq!(
            seen,
            [
                ProcessPriority::Low,
                ProcessPriority::Lowest,
                ProcessPriority::Highest,
                ProcessPriority::High,
                ProcessPriority::Normal
            ]
        );
        let back = changed(Field::Priority, &Settings::default(), Step::Previous, &[]);
        assert_eq!(back.process_priority, ProcessPriority::High);
    }

    #[test]
    fn the_value_names_the_level_and_a_refusal() {
        set_locale(Locale::EnUs);
        let settings = Settings {
            process_priority: ProcessPriority::Highest,
            ..Settings::default()
        };
        assert_eq!(value_text(Field::Priority, &settings, &[]), "Highest");
        let facts = FontFacts {
            priority: PriorityOutcome::NeedsPrivileges,
            ..FontFacts::default()
        };
        assert_eq!(value(&settings, facts), "Highest (needs privileges)");
        let row = line(Field::Priority, &settings, true, &[], (200, facts), None);
        assert!(row.starts_with("Process priority"), "{row}");
    }
}
