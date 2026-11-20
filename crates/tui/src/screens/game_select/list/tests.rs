use oxysol_core::Variant;

use super::*;
use crate::screens::game_select::catalog::Family;

fn long_rows(groups: usize, per_group: usize) -> Vec<Row> {
    let mut rows = Vec::new();
    for _ in 0..groups {
        rows.push(Row::Heading(Heading::Family(Family::Klondike)));
        rows.extend((0..per_group).map(|_| Row::Item(Variant::FreeCell)));
    }
    rows
}

fn position_of_focus(rows: &[Row], focus: usize) -> usize {
    rows.iter()
        .enumerate()
        .filter(|(_, row)| matches!(row, Row::Item(_)))
        .nth(focus)
        .map_or(0, |(position, _)| position)
}

#[test]
fn the_focused_variant_is_always_inside_the_visible_rows() {
    let rows = long_rows(5, 4);
    let height = 6;
    let count = items(&rows).len();
    let mut view = ListView::default();
    for focus in (0..count).chain((0..count).rev()) {
        view.set_focus(focus);
        view.keep_in_view(&rows, height);
        let at = position_of_focus(&rows, focus);
        assert!(
            at >= view.scroll && at < view.scroll + height,
            "focus {focus}"
        );
    }
}

#[test]
fn the_heading_stays_in_view_when_its_first_variant_is_focused() {
    let rows = long_rows(5, 4);
    let mut view = ListView::default();
    view.set_focus(0);
    view.keep_in_view(&rows, 6);
    assert_eq!(view.scroll, 0);
    view.set_focus(8);
    view.keep_in_view(&rows, 6);
    let top = position_of_focus(&rows, 8) - 1;
    assert!(view.scroll <= top);
}

#[test]
fn a_list_that_fits_does_not_scroll() {
    let rows = long_rows(2, 3);
    let mut view = ListView::default();
    view.set_focus(items(&rows).len() - 1);
    view.keep_in_view(&rows, rows.len());
    assert_eq!(view.scroll, 0);
}
