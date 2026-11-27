//! The window of rows shown when the screen is too low for all rows of the active tab.

/// The first row index to show so that `focus` is inside a window of `height` rows out of
/// `total`. `current` is where the window was; it moves only as far as it must.
pub(super) fn first_shown(total: usize, focus: usize, height: usize, current: usize) -> usize {
    if height == 0 {
        return 0;
    }
    let last_start = total.saturating_sub(height);
    let start = if focus < current {
        focus
    } else if focus >= current + height {
        focus + 1 - height
    } else {
        current
    };
    start.min(last_start)
}

/// The row index inside the visible window at `offset` nearest to `current`.
pub(super) fn nearest_visible(total: usize, offset: usize, height: usize, current: usize) -> usize {
    if total == 0 || height == 0 {
        return 0;
    }
    let last_start = total.saturating_sub(height);
    let start = offset.min(last_start);
    let end = (start + height).min(total);
    current.clamp(start, end.saturating_sub(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_window_that_holds_everything_starts_at_the_top() {
        assert_eq!(first_shown(24, 23, 24, 0), 0);
    }

    #[test]
    fn the_window_follows_the_focus_down_and_back_up() {
        let total = 24;
        let height = 10;
        let mut first = 0;
        for focus in 0..total {
            first = first_shown(total, focus, height, first);
            assert!(focus >= first && focus < first + height);
        }
        assert_eq!(first, total - height);
        for focus in (0..total).rev() {
            first = first_shown(total, focus, height, first);
            assert!(focus >= first && focus < first + height);
        }
        assert_eq!(first, 0);
    }
}
