//! One-line plain-text descriptions of the focus and the last event for the status area.
//! Stub: P3-08 builds the real announcer from localized card and pile names.

/// Joins a description of the focus with a description of the last event, skipping empty parts.
pub fn announce(focus: &str, last_event: &str) -> String {
    match (focus.is_empty(), last_event.is_empty()) {
        (true, true) => String::new(),
        (false, true) => focus.to_owned(),
        (true, false) => last_event.to_owned(),
        (false, false) => format!("{focus}: {last_event}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_parts_are_skipped() {
        assert_eq!(announce("", ""), "");
        assert_eq!(announce("Menu", ""), "Menu");
        assert_eq!(announce("", "Moved"), "Moved");
        assert_eq!(announce("Menu", "Moved"), "Menu: Moved");
    }
}
