use oxysol_core::{Rank, Suit};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::*;
use crate::i18n::Locale;

fn middle(lines: &[String]) -> String {
    lines[lines.len() / 2]
        .trim_matches([' ', '│', '|'])
        .to_owned()
}

fn slot(mode: GlyphMode, size: (u16, u16), placeholder: Placeholder) -> Vec<String> {
    render_face(mode, size, Face::Slot(placeholder), (Mark::default(), None))
}

#[test]
fn placeholders_name_what_an_empty_slot_is_for() {
    let expected = [
        (Placeholder::None, "", ""),
        (Placeholder::Ace, "A", "A"),
        (Placeholder::Recycle, "↑", "^"),
        (Placeholder::Spent, "x", "x"),
    ];
    each_locale(|locale| {
        for (placeholder, unicode, ascii) in expected {
            for size in SIZES {
                let found = [GlyphMode::Unicode, GlyphMode::Ascii]
                    .map(|mode| middle(&slot(mode, size, placeholder)));
                assert_eq!(
                    found,
                    [unicode, ascii],
                    "{locale:?} {placeholder:?} {size:?}"
                );
            }
        }
    });
}

#[test]
fn the_foundation_placeholder_comes_from_the_locale() {
    set_locale(Locale::PlPl);
    let polish = middle(&slot(GlyphMode::Unicode, (9, 7), Placeholder::Ace));
    set_locale(Locale::EnUs);
    assert_eq!(polish, "A");
}

#[test]
fn an_empty_slot_with_a_placeholder_differs_from_a_plain_one_and_from_a_face_down_card() {
    for mode in MODES {
        for size in SIZES {
            let plain = slot(mode, size, Placeholder::None);
            let ace = slot(mode, size, Placeholder::Ace);
            let down = render_face(mode, size, Face::Down, (Mark::default(), None));
            assert_ne!(plain, ace);
            assert_ne!(plain, down);
        }
    }
}

#[test]
fn the_stock_badge_is_a_count_with_a_prefix_so_it_is_not_read_as_a_rank() {
    for (mode, expected) in [(GlyphMode::Unicode, "┘"), (GlyphMode::Ascii, "+")] {
        for size in SIZES {
            let lines = render_face(mode, size, Face::Down, (Mark::default(), Some(18)));
            let bottom = &lines[lines.len() - 1];
            assert!(
                bottom.ends_with(&format!("x18{expected}")),
                "{mode:?} {bottom}"
            );
        }
    }
}

fn back_row(mode: GlyphMode, mark: Mark) -> String {
    let fixture = Fixture::new(mode);
    let area = Rect::new(0, 0, 7, 1);
    let mut buf = Buffer::empty(area);
    draw_back_row(&mut buf, area, mark, &fixture.ctx());
    rows(&buf, area).remove(0)
}

#[test]
fn a_covered_face_down_card_draws_the_back_pattern_in_its_visible_row() {
    assert_eq!(back_row(GlyphMode::Unicode, Mark::default()), "┌▒▒▒▒▒┐");
    assert_eq!(back_row(GlyphMode::Ascii, Mark::default()), "+#####+");
    for mode in MODES {
        let empty = render_face(
            mode,
            (7, 5),
            Face::Slot(Placeholder::None),
            (Mark::default(), None),
        );
        assert_ne!(back_row(mode, Mark::default()), empty[0]);
    }
}

#[test]
fn a_fanned_card_shows_only_its_border_and_label_in_the_visible_strip() {
    for mode in MODES {
        for size in SIZES {
            let fixture = Fixture::new(mode);
            let area = Rect::new(0, 0, size.0, size.1);
            let mut buf = Buffer::empty(area);
            let face = Face::Up(card(Rank::Queen, Suit::Hearts));
            draw_fanned(&mut buf, area, 3, (face, Mark::default()), &fixture.ctx());
            let lines = rows(&buf, area);
            let strip = |row: usize| lines[row].chars().take(3).collect::<String>();
            assert!(strip(0).contains('Q'), "{mode:?} {size:?}: the label stays");
            for row in 1..lines.len() - 1 {
                let inner: String = strip(row).chars().skip(1).collect();
                assert_eq!(inner, "  ", "{mode:?} {size:?} row {row}");
            }
            assert_eq!(strip(lines.len() - 1), "   ", "{mode:?} {size:?} bottom");
            let beyond: String = lines[lines.len() - 1].chars().skip(3).collect();
            assert_ne!(
                beyond.trim(),
                "",
                "{mode:?} {size:?}: the rest of the card stays"
            );
        }
    }
}
