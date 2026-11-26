use oxysol_core::Variant;

use super::*;
use crate::screens::game_select::catalog::Family;
use crate::screens::game_select::rows::items;

fn long_rows(groups: usize, per_group: usize) -> Vec<Row> {
    let families = Family::ALL;
    let mut rows = Vec::new();
    for g in 0..groups {
        let family = families[g % families.len()];
        rows.push(Row::Heading {
            heading: Heading::Family(family),
            folded: false,
            count: per_group,
        });
        rows.extend(
            Variant::ALL
                .iter()
                .filter(|v| Family::of(**v) == family)
                .cycle()
                .take(per_group)
                .copied()
                .map(Row::Item),
        );
    }
    rows
}

#[test]
fn the_focused_row_is_always_inside_the_visible_rows() {
    let rows = long_rows(5, 3);
    let height = 6;
    let focusable_count = rows.iter().filter(|row| row.is_focusable()).count();
    let mut view = ListView {
        view_height: height,
        ..ListView::default()
    };
    for _ in 0..focusable_count {
        let pos = view.focused_row(&rows, &[]).unwrap_or(0);
        assert!(
            pos >= view.scroll && pos < view.scroll + height,
            "pos {pos}"
        );
        view.step_action(Action::Down, &rows, &[]);
        view.keep_resolved_in_view(&rows, &[]);
    }
    for _ in 0..focusable_count {
        view.step_action(Action::Up, &rows, &[]);
        view.keep_resolved_in_view(&rows, &[]);
        let pos = view.focused_row(&rows, &[]).unwrap_or(0);
        assert!(
            pos >= view.scroll && pos < view.scroll + height,
            "pos {pos}"
        );
    }
}

#[test]
fn the_heading_stays_in_view_when_its_first_variant_is_focused() {
    let rows = long_rows(5, 2);
    let mut view = ListView::default();
    let target = items(&rows)[4];
    view.set_focus_item(FocusItem::Variant(target));
    let pos = view.focused_row(&rows, &[]).unwrap_or(0);
    view.keep_position_in_view(&rows, pos, 6);
    assert!(view.scroll < pos);
}

#[test]
fn a_list_that_fits_does_not_scroll() {
    let rows = long_rows(2, 2);
    let mut view = ListView::default();
    view.focus_last(&rows);
    let pos = view.focused_row(&rows, &[]).unwrap_or(0);
    view.keep_position_in_view(&rows, pos, rows.len());
    assert_eq!(view.scroll, 0);
}

#[test]
fn wheel_scrolling_keeps_visible_focus_and_moves_when_focus_leaves_view() {
    let rows = long_rows(5, 2);
    let mut view = ListView {
        view_height: 6,
        ..ListView::default()
    };
    let start_pos = view.focused_row(&rows, &[]).unwrap_or(0);
    assert_eq!(start_pos, 1);

    view.scroll_by(1, &rows, &[]);
    assert_eq!(view.scroll, 1);
    assert_eq!(view.focused_row(&rows, &[]), Some(1));

    view.scroll_by(3, &rows, &[]);
    assert_eq!(view.scroll, 4);
    let moved = view.focused_row(&rows, &[]).unwrap_or(0);
    assert!((4..10).contains(&moved));
    assert!(matches!(rows[moved], Row::Item(_)));
}

#[test]
fn page_keys_and_home_end_move_within_bounds() {
    let rows = long_rows(5, 2);
    let mut view = ListView {
        view_height: 5,
        ..ListView::default()
    };
    view.page_by(false, &rows, &[]);
    assert_eq!(view.focused_row(&rows, &[]), Some(6));
    view.end(&rows);
    assert_eq!(
        view.focused_row(&rows, &[]),
        rows.iter().rposition(|row| matches!(row, Row::Item(_)))
    );
    view.page_by(true, &rows, &[]);
    let after_up = view.focused_row(&rows, &[]).unwrap_or(0);
    assert!(after_up < rows.len() - 1);
    view.home(&rows);
    assert_eq!(
        view.focused_row(&rows, &[]),
        rows.iter().position(|row| matches!(row, Row::Item(_)))
    );
}
