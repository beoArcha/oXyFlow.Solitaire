//! Selection and scrolling of the history list. Pure state: the screen tells it how many rows
//! exist and how many fit.

use std::ops::Range;

/// The selected row and the first visible row.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ListState {
    selected: usize,
    offset: usize,
    page: usize,
}

impl ListState {
    /// The selected row.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Rows that fit, as of the last draw; at least one so that paging always moves.
    pub fn page(&self) -> usize {
        self.page.max(1)
    }

    /// The first visible row as of the last draw.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Scrolls the window to `new_offset`, keeping the selection inside the visible range.
    pub fn scroll_to_offset(&mut self, new_offset: usize, len: usize) {
        let max_offset = len.saturating_sub(self.page());
        self.offset = new_offset.min(max_offset);
        let min_selected = self.offset;
        let max_selected = (self.offset + self.page())
            .saturating_sub(1)
            .min(len.saturating_sub(1));
        self.selected = self.selected.clamp(min_selected, max_selected);
    }

    /// Selects `index`, or the last row when it is beyond the end.
    pub fn select(&mut self, index: usize, len: usize) {
        self.selected = index.min(len.saturating_sub(1));
    }

    /// Moves the selection by `delta` rows and stops at both ends.
    pub fn step(&mut self, delta: isize, len: usize) {
        let target = self.selected.saturating_add_signed(delta);
        self.select(target, len);
    }

    /// Moves the selection a page up (`up`) or down.
    pub fn page_by(&mut self, up: bool, len: usize) {
        let page = isize::try_from(self.page()).unwrap_or(isize::MAX);
        self.step(if up { -page } else { page }, len);
    }

    /// Scrolls so that the selected row is visible in a window of `height` rows and returns the
    /// range of rows to draw.
    pub fn visible(&mut self, height: usize, len: usize) -> Range<usize> {
        self.page = height;
        self.selected = self.selected.min(len.saturating_sub(1));
        if height == 0 {
            return 0..0;
        }
        if self.selected < self.offset {
            self.offset = self.selected;
        } else if self.selected >= self.offset + height {
            self.offset = self.selected + 1 - height;
        }
        self.offset = self.offset.min(len.saturating_sub(height));
        self.offset..(self.offset + height).min(len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_selection_stops_at_both_ends() {
        let mut list = ListState::default();
        list.step(-1, 5);
        assert_eq!(list.selected(), 0);
        list.step(10, 5);
        assert_eq!(list.selected(), 4);
        list.step(1, 5);
        assert_eq!(list.selected(), 4);
    }

    #[test]
    fn an_empty_list_selects_row_zero() {
        let mut list = ListState::default();
        list.step(3, 0);
        assert_eq!(list.selected(), 0);
        assert_eq!(list.visible(5, 0), 0..0);
    }

    #[test]
    fn the_window_follows_the_selection() {
        let mut list = ListState::default();
        assert_eq!(list.visible(4, 10), 0..4);
        list.select(5, 10);
        assert_eq!(list.visible(4, 10), 2..6);
        list.select(0, 10);
        assert_eq!(list.visible(4, 10), 0..4);
        list.select(9, 10);
        assert_eq!(list.visible(4, 10), 6..10);
    }

    #[test]
    fn a_page_moves_by_the_visible_height() {
        let mut list = ListState::default();
        list.visible(4, 20);
        list.page_by(false, 20);
        assert_eq!(list.selected(), 4);
        list.page_by(true, 20);
        assert_eq!(list.selected(), 0);
    }

    #[test]
    fn a_taller_window_than_the_list_shows_everything_from_the_top() {
        let mut list = ListState::default();
        list.select(2, 3);
        assert_eq!(list.visible(10, 3), 0..3);
    }

    #[test]
    fn scroll_to_offset_clamps_offset_and_keeps_selection_inside() {
        let mut list = ListState::default();
        list.visible(5, 60);
        list.scroll_to_offset(20, 60);
        assert_eq!(list.offset(), 20);
        assert_eq!(list.selected(), 20);
        list.scroll_to_offset(10, 60);
        assert_eq!(list.offset(), 10);
        assert_eq!(list.selected(), 14);
        list.scroll_to_offset(100, 60);
        assert_eq!(list.offset(), 55);
        assert_eq!(list.selected(), 55);
    }
}
