//! The lines of the settings list and the part of them that is shown when the screen is too low for
//! all of them.

use super::options::{FIELDS, Group};

/// One line of the list: a group heading, a blank line between groups, or a row (its index in
/// [`FIELDS`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Entry {
    Heading(Group),
    Gap,
    Row(usize),
}

/// The lines of the list, groups in order, a gap before every group but the first.
pub(super) fn entries() -> Vec<Entry> {
    let mut entries = Vec::with_capacity(FIELDS.len() + 2 * Group::ALL.len());
    for (position, group) in Group::ALL.into_iter().enumerate() {
        if position > 0 {
            entries.push(Entry::Gap);
        }
        entries.push(Entry::Heading(group));
        for (index, field) in FIELDS.iter().enumerate() {
            if field.group() == group {
                entries.push(Entry::Row(index));
            }
        }
    }
    entries
}

/// The first line to show so that the line of row `focus` is inside a window of `height` lines,
/// with its group heading when the row opens the group. `current` is where the window was; it only
/// moves as far as it must.
pub(super) fn first_shown(entries: &[Entry], focus: usize, height: usize, current: usize) -> usize {
    let last_start = entries.len().saturating_sub(height);
    let Some(at) = entries.iter().position(|entry| *entry == Entry::Row(focus)) else {
        return current.min(last_start);
    };
    let top = match at.checked_sub(1).and_then(|before| entries.get(before)) {
        Some(Entry::Heading(_)) => at - 1,
        _ => at,
    };
    let start = if top < current {
        top
    } else if at >= current + height {
        at + 1 - height
    } else {
        current
    };
    start.min(last_start)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_window_that_holds_everything_starts_at_the_top() {
        let all = entries();
        assert_eq!(first_shown(&all, FIELDS.len() - 1, all.len(), 0), 0);
    }

    #[test]
    fn the_window_follows_the_focus_down_and_back_up() {
        let all = entries();
        let height = 10;
        let mut first = 0;
        for focus in 0..FIELDS.len() {
            first = first_shown(&all, focus, height, first);
            let at = all.iter().position(|entry| *entry == Entry::Row(focus));
            assert!(matches!(at, Some(at) if at >= first && at < first + height));
        }
        assert_eq!(first, all.len() - height);
        for focus in (0..FIELDS.len()).rev() {
            first = first_shown(&all, focus, height, first);
            let at = all.iter().position(|entry| *entry == Entry::Row(focus));
            assert!(matches!(at, Some(at) if at >= first && at < first + height));
        }
        assert_eq!(first, 0);
    }

    #[test]
    fn the_heading_is_shown_with_the_first_row_of_its_group() {
        let all = entries();
        let first_interface = FIELDS
            .iter()
            .position(|field| field.group() == Group::Interface)
            .unwrap_or_default();
        let shown = first_shown(&all, first_interface, 8, all.len());
        assert!(matches!(
            all.get(shown),
            Some(Entry::Heading(Group::Interface))
        ));
    }
}
