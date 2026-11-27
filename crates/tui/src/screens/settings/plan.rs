//! Where the preview pane goes: beside the list when the window is wide enough, under it when it is
//! high enough, otherwise nowhere (the screen then says so). The plan depends only on the window and
//! the list, never on the settings, so the list does not move while a setting is changed.

use ratatui::layout::Rect;

use crate::layout::{CardSize, SIZES};

/// Columns between the list and a pane beside it.
pub(super) const GAP: u16 = 2;

/// Columns reserved at the right edge of a scrolling list: one gap column and the scroll bar.
pub(super) const BAR_COLUMNS: u16 = 2;

/// Width of the list block for `content_width` row columns, adding room for the scroll bar when
/// the list scrolls, never wider than `screen_width`.
pub(super) fn list_box_width(content_width: u16, screen_width: u16, scrolls: bool) -> u16 {
    if scrolls {
        content_width.saturating_add(BAR_COLUMNS).min(screen_width)
    } else {
        content_width.min(screen_width)
    }
}

/// The fewest list rows kept when the pane goes under a long list.
const MIN_LIST_ROWS: u16 = 12;

/// Cards in the pane: a back, then the focused, the carried and the target card.
pub(super) const CARDS: u16 = 4;

/// The widest aura, in rings of cells around a card.
pub(super) const WIDEST_AURA: u16 = 2;

/// Columns the cards of the pane take with an aura of `rings` rings: the cards, the gaps between
/// them and the rings at both ends.
pub(super) const fn content_width(card: CardSize, rings: u16) -> u16 {
    CARDS * card.width + (CARDS - 1) * card_gap(rings) + 2 * rings
}

/// Rows the pane takes with an aura of `rings` rings: its title, the rings above and under the
/// cards, the cards, and the row of labels.
pub(super) const fn content_rows(card: CardSize, rings: u16) -> u16 {
    card.height + 2 * rings + 2
}

/// Columns between two cards: room for both rings and one free column.
pub(super) const fn card_gap(rings: u16) -> u16 {
    2 * rings + 1
}

/// Where the preview pane is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Pane {
    /// Right of the list, `width` columns wide, `rows` rows high (at least the list height).
    Side { width: u16, rows: u16 },
    /// Under the list, `rows` rows high.
    Below { rows: u16, width: u16 },
    /// Not drawn: the window is too small.
    Hidden,
}

/// The rows of the list that are shown and the place of the pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Plan {
    pub list_rows: u16,
    pub pane: Pane,
}

/// Plans `entries` list lines of `list_width` columns in a screen `screen_width` columns wide that
/// has `body_rows` rows for the list (and the pane, when it goes under it).
pub(super) fn plan(screen_width: u16, body_rows: u16, list_width: u16, entries: usize) -> Plan {
    let smallest = CardSize::smallest();
    let (min_width, min_rows) = (
        content_width(smallest, WIDEST_AURA),
        content_rows(smallest, WIDEST_AURA),
    );
    let (max_width, max_rows) = (
        content_width(SIZES[0], WIDEST_AURA),
        content_rows(SIZES[0], WIDEST_AURA),
    );
    let entries = u16::try_from(entries).unwrap_or(u16::MAX);
    let shown = entries.min(body_rows);
    let side_list = list_box_width(list_width, screen_width, entries > body_rows);
    let beside = screen_width.saturating_sub(side_list + GAP);
    if beside >= min_width && body_rows >= min_rows {
        return Plan {
            list_rows: shown,
            pane: Pane::Side {
                width: beside.min(max_width),
                rows: body_rows.min(max_rows),
            },
        };
    }
    let min_list = MIN_LIST_ROWS.min(entries);
    let rows = body_rows.saturating_sub(entries).clamp(min_rows, max_rows);
    let list_rows = entries.min(body_rows.saturating_sub(rows));
    if screen_width >= min_width && list_rows > 0 && list_rows >= min_list {
        return Plan {
            list_rows,
            pane: Pane::Below {
                rows,
                width: screen_width.min(max_width),
            },
        };
    }
    Plan {
        list_rows: shown,
        pane: Pane::Hidden,
    }
}

/// Where the list and the preview pane are drawn, the block of both centred in the screen.
pub(super) struct Places {
    pub list: Rect,
    pub pane: Option<Rect>,
}

impl Places {
    pub(super) fn new(pane: Pane, list_width: u16, (list_area, below_area): (Rect, Rect)) -> Self {
        let centred = |area: Rect, width: u16| area.x + area.width.saturating_sub(width) / 2;
        let beside = match pane {
            Pane::Side { width, .. } => GAP + width,
            Pane::Below { .. } | Pane::Hidden => 0,
        };
        let x = centred(list_area, list_width + beside);
        let list = Rect::new(x, list_area.y, list_width, list_area.height);
        let pane = match pane {
            Pane::Side { width, rows } => Some(Rect::new(
                x + list_width + GAP,
                list_area.y,
                width,
                list_area.height.max(rows),
            )),
            Pane::Below { width, .. } => Some(Rect::new(
                centred(below_area, width),
                below_area.y,
                width,
                below_area.height,
            )),
            Pane::Hidden => None,
        };
        Self { list, pane }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wide_window_puts_the_pane_beside_the_list() {
        let plan = plan(130, 33, 49, 24);
        assert_eq!(plan.list_rows, 24);
        assert!(matches!(plan.pane, Pane::Side { width, .. } if width <= 130 - 49 - GAP));
    }

    #[test]
    fn a_wide_window_also_puts_the_pane_beside_a_short_tab() {
        let plan = plan(130, 30, 49, 7);
        assert_eq!(plan.list_rows, 7);
        assert!(matches!(plan.pane, Pane::Side { rows, .. } if rows >= 11));
    }

    #[test]
    fn a_narrow_high_window_puts_the_pane_under_the_list() {
        let plan = plan(60, 50, 49, 24);
        assert_eq!(plan.list_rows, 24);
        assert!(matches!(plan.pane, Pane::Below { rows, .. } if rows >= 11));
    }

    #[test]
    fn a_short_tab_shows_the_pane_below_when_height_allows() {
        let plan = plan(100, 42, 60, 7);
        assert_eq!(plan.list_rows, 7);
        assert!(matches!(plan.pane, Pane::Below { rows, .. } if rows >= 11));
    }

    #[test]
    fn a_narrow_low_window_hides_the_pane_and_keeps_the_whole_height_for_the_list() {
        let plan = plan(60, 20, 49, 24);
        assert_eq!(plan.list_rows, 20);
        assert_eq!(plan.pane, Pane::Hidden);
    }

    #[test]
    fn a_wide_window_with_scrolling_reserves_bar_columns_beside_the_list() {
        let p = plan(130, 20, 49, 27);
        assert_eq!(p.list_rows, 20);
        assert!(matches!(
            p.pane,
            Pane::Side { width, .. } if 49 + BAR_COLUMNS + GAP + width <= 130
        ));
    }

    #[test]
    fn the_pane_never_takes_more_than_the_window_has() {
        for entries in [7, 24] {
            for width in 20..160 {
                for rows in 5..60 {
                    let plan = plan(width, rows, 49, entries);
                    let pane_rows = match plan.pane {
                        Pane::Below { rows, .. } => rows,
                        Pane::Side { rows, .. } => rows,
                        Pane::Hidden => 0,
                    };
                    if let Pane::Below { .. } = plan.pane {
                        assert!(plan.list_rows + pane_rows <= rows);
                    } else {
                        assert!(plan.list_rows.max(pane_rows) <= rows);
                    }
                    match plan.pane {
                        Pane::Side { width: pane, .. } => assert!(49 + GAP + pane <= width),
                        Pane::Below { width: pane, .. } => assert!(pane <= width),
                        Pane::Hidden => {}
                    }
                }
            }
        }
    }
}
