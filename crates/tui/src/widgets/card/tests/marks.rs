use oxysol_core::{Rank, Suit};

use super::*;

fn marks() -> [(&'static str, Mark); 6] {
    let base = Mark::default();
    [
        ("none", base),
        ("focus", Mark { focus: true, ..base }),
        ("selected", Mark { selected: true, ..base }),
        ("target", Mark { target: true, ..base }),
        ("hint source", Mark { hint: Some(Hint::Source), ..base }),
        ("hint target", Mark { hint: Some(Hint::Target), ..base }),
    ]
}

fn faces() -> [(&'static str, Face); 3] {
    [
        ("up", Face::Up(card(Rank::Queen, Suit::Hearts))),
        ("down", Face::Down),
        ("slot", Face::Slot(Placeholder::Ace)),
    ]
}

#[test]
fn every_mark_changes_the_card_by_symbol_or_border_in_both_sets() {
    each_locale(|locale| {
        for mode in MODES {
            for size in SIZES {
                for (kind, face) in faces() {
                    let looks: Vec<_> = marks()
                        .into_iter()
                        .map(|(name, mark)| (name, render_face(mode, size, face, (mark, None))))
                        .collect();
                    for (index, (first, first_look)) in looks.iter().enumerate() {
                        for (second, second_look) in &looks[index + 1..] {
                            assert_ne!(
                                first_look, second_look,
                                "{locale:?} {mode:?} {size:?} {kind}: {first} = {second}"
                            );
                        }
                    }
                }
            }
        }
    });
}

#[test]
fn a_target_shows_the_plus_in_both_top_corners_and_on_an_empty_slot_in_the_middle() {
    for mode in MODES {
        let target = Mark { target: true, ..Mark::default() };
        for size in SIZES {
            for (kind, face) in faces() {
                let lines = render_face(mode, size, face, (target, None));
                assert!(lines[0].starts_with('+') && lines[0].ends_with('+'), "{mode:?} {kind} {lines:?}");
            }
            let slot = render_face(mode, size, Face::Slot(Placeholder::None), (target, None));
            let middle = &slot[usize::from(size.1) / 2];
            assert_eq!(middle.trim_matches([' ', '│', '|', '!']), "+", "{mode:?} {size:?}");
        }
    }
}

#[test]
fn a_focused_target_keeps_the_plus_corners() {
    for mode in MODES {
        let both = Mark { target: true, focus: true, ..Mark::default() };
        let lines = render_face(mode, (7, 5), Face::Up(card(Rank::Five, Suit::Clubs)), (both, None));
        assert!(lines[0].starts_with('+') && lines[0].ends_with('+'), "{lines:?}");
        let focus_only = Mark { focus: true, ..Mark::default() };
        let plain = render_face(mode, (7, 5), Face::Up(card(Rank::Five, Suit::Clubs)), (focus_only, None));
        assert_ne!(lines, plain);
    }
}

#[test]
fn the_hint_marks_the_source_and_the_target_with_two_cells() {
    let source = Mark { hint: Some(Hint::Source), ..Mark::default() };
    let target = Mark { hint: Some(Hint::Target), ..Mark::default() };
    let expected = [
        (GlyphMode::Unicode, "?→", "→?"),
        (GlyphMode::Ascii, "?>", ">?"),
    ];
    for (mode, from, to) in expected {
        for size in SIZES {
            let face = Face::Up(card(Rank::Ten, Suit::Hearts));
            let source_top = &render_face(mode, size, face, (source, None))[0];
            let target_top = &render_face(mode, size, face, (target, None))[0];
            let corner = |top: &str| top.chars().last().unwrap_or(' ');
            let from_end = format!("{from}{}", corner(source_top));
            let to_end = format!("{to}{}", corner(target_top));
            assert!(source_top.ends_with(&from_end), "{mode:?} {size:?} {source_top}");
            assert!(target_top.ends_with(&to_end), "{mode:?} {size:?} {target_top}");
            assert!(source_top.contains("10"), "the label stays: {source_top}");
        }
    }
}

#[test]
fn the_hint_border_differs_from_the_normal_focus_and_selection_borders_in_both_sets() {
    for mode in MODES {
        let corner = |mark: Mark| render_face(mode, (9, 7), Face::Up(card(Rank::Two, Suit::Clubs)), (mark, None))[0]
            .chars()
            .next();
        let hint = corner(Mark { hint: Some(Hint::Source), ..Mark::default() });
        let others = [
            corner(Mark::default()),
            corner(Mark { focus: true, ..Mark::default() }),
            corner(Mark { selected: true, ..Mark::default() }),
        ];
        if mode == GlyphMode::Unicode {
            assert!(others.iter().all(|other| *other != hint), "{hint:?} {others:?}");
        }
        let side = |mark: Mark| render_face(mode, (9, 7), Face::Up(card(Rank::Two, Suit::Clubs)), (mark, None))[2]
            .chars()
            .next();
        assert_ne!(side(Mark::default()), side(Mark { hint: Some(Hint::Source), ..Mark::default() }));
    }
}

#[test]
fn a_mark_that_does_not_fit_beside_the_label_is_left_out_whole() {
    let hint = Mark { hint: Some(Hint::Source), focus: true, ..Mark::default() };
    let face = Face::Up(card(Rank::Ten, Suit::Spades));
    let lines = render_face(GlyphMode::Unicode, (7, 5), face, (hint, None));
    assert!(lines[0].contains('►'), "{}", lines[0]);
    assert!(!lines[0].contains('?'), "{}", lines[0]);
}
