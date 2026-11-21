//! The text of the `Now:` row: what is focused, carried or hinted, in the colon form
//! `{pile}: {what}`. A pile name is never the object of a verb, so a language can decline the rest
//! of the sentence freely. The row is also what a screen reader reads.

use oxysol_core::{Move, PileRef, PileSnapshot};

use super::status::look_at;
use crate::announce::{Look, look_text, pile_name, targets_text};
use crate::i18n::{t, tf};

fn snapshot_of(piles: &[PileSnapshot], pile: PileRef) -> Option<&PileSnapshot> {
    piles.iter().find(|snapshot| snapshot.pile == pile)
}

/// The cards `count` deep in `pile` in words: the card, or `3 cards from seven of hearts`.
fn what_of(piles: &[PileSnapshot], pile: PileRef, count: usize) -> String {
    let card = look_text(snapshot_of(piles, pile).map_or(Look::Empty, |s| look_at(s, count)));
    if count > 1 {
        return tf(
            "board.action.cards",
            &[("count", &count.to_string()), ("what", &card)],
        );
    }
    card
}

/// The row while the cursor browses `pile`, `depth` cards from its top.
pub(super) fn browsing(piles: &[PileSnapshot], pile: PileRef, depth: usize) -> String {
    let what = look_text(snapshot_of(piles, pile).map_or(Look::Empty, |s| look_at(s, depth)));
    let name = pile_name(pile);
    if depth > 1 {
        return tf(
            "board.action.browsing_run",
            &[
                ("pile", &name),
                ("count", &depth.to_string()),
                ("what", &what),
            ],
        );
    }
    tf("board.action.browsing", &[("pile", &name), ("what", &what)])
}

/// The row while `count` cards of `source` are picked up and `targets` piles accept them.
pub(super) fn carrying(
    piles: &[PileSnapshot],
    source: PileRef,
    count: u8,
    targets: usize,
) -> String {
    tf(
        "board.action.carrying",
        &[
            ("what", &what_of(piles, source, usize::from(count))),
            ("pile", &pile_name(source)),
            ("targets", &targets_text(targets)),
        ],
    )
}

/// The compact name of a pile, such as `T5`; the piles that are never crowded keep their name.
fn short_pile_name(pile: PileRef) -> String {
    let ordinal = |index: u8| (u16::from(index) + 1).to_string();
    match pile {
        PileRef::Tableau(index) => tf("board.name.tableau_short", &[("number", &ordinal(index))]),
        PileRef::Foundation(index) => tf(
            "board.name.foundation_short",
            &[("number", &ordinal(index))],
        ),
        PileRef::Cell(index) => tf("board.name.free_cell_short", &[("number", &ordinal(index))]),
        PileRef::Pyramid { row, index } => tf(
            "board.name.pyramid_short",
            &[("row", &ordinal(row)), ("position", &ordinal(index))],
        ),
        PileRef::Stock | PileRef::Waste => pile_name(pile),
    }
}

/// The wordings of [`carrying`] for a narrow row, longest first: the compact column name with the
/// targets, then without them. The source column stays in every one of them.
pub(super) fn carrying_shorter(
    piles: &[PileSnapshot],
    source: PileRef,
    count: u8,
    targets: usize,
) -> Vec<String> {
    let what = what_of(piles, source, usize::from(count));
    let pile = short_pile_name(source);
    vec![
        tf(
            "board.action.carrying",
            &[
                ("what", &what),
                ("pile", &pile),
                ("targets", &targets_text(targets)),
            ],
        ),
        tf(
            "board.action.carrying_short",
            &[("what", &what), ("pile", &pile)],
        ),
    ]
}

/// The row after a hint: the move it suggests, named in words.
pub(super) fn hint(piles: &[PileSnapshot], mv: Move) -> String {
    match mv {
        Move::Stock => t("board.action.hint_stock").into_owned(),
        Move::Transfer { from, to, count } => tf(
            "board.action.hint_move",
            &[
                ("what", &what_of(piles, from, usize::from(count))),
                ("pile", &pile_name(to)),
            ],
        ),
        Move::Remove {
            first,
            second: Some(second),
        } => tf(
            "board.action.hint_pair",
            &[
                ("what", &what_of(piles, first, 1)),
                ("other", &what_of(piles, second, 1)),
            ],
        ),
        Move::Remove {
            first,
            second: None,
        } => tf(
            "board.action.hint_single",
            &[("what", &what_of(piles, first, 1))],
        ),
    }
}
