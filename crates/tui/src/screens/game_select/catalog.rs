//! The structure of the game menu: the families the games belong to, in a fixed order, and which
//! games carry the popularity star. A new game is placed by adding its family (or a variant to an
//! existing one) here; the compiler points at every `match` that has to learn it.

use crate::settings::GameGroup;
use oxysol_core::Variant;

/// A group of related games. The order of [`Family::ALL`] is the order of the menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Family {
    Klondike,
    Spider,
    FreeCell,
    Pyramid,
    Yukon,
    Golf,
    TriPeaks,
}

impl Family {
    pub(super) const ALL: [Family; 7] = [
        Family::Klondike,
        Family::Spider,
        Family::FreeCell,
        Family::Pyramid,
        Family::Yukon,
        Family::Golf,
        Family::TriPeaks,
    ];

    pub(super) const fn of(variant: Variant) -> Family {
        match variant {
            Variant::Klondike(_) => Family::Klondike,
            Variant::Spider(_) => Family::Spider,
            Variant::FreeCell => Family::FreeCell,
            Variant::Pyramid(_) => Family::Pyramid,
            Variant::Yukon(_) => Family::Yukon,
            Variant::Golf => Family::Golf,
            Variant::TriPeaks => Family::TriPeaks,
        }
    }

    pub(super) const fn title_key(self) -> &'static str {
        match self {
            Family::Klondike => "menu.family.klondike",
            Family::Spider => "menu.family.spider",
            Family::FreeCell => "menu.family.free_cell",
            Family::Pyramid => "menu.family.pyramid",
            Family::Yukon => "menu.family.yukon",
            Family::Golf => "menu.family.golf",
            Family::TriPeaks => "menu.family.tri_peaks",
        }
    }

    pub(super) const fn group(self) -> GameGroup {
        match self {
            Family::Klondike => GameGroup::Klondike,
            Family::Spider => GameGroup::Spider,
            Family::FreeCell => GameGroup::FreeCell,
            Family::Pyramid => GameGroup::Pyramid,
            Family::Yukon => GameGroup::Yukon,
            Family::Golf => GameGroup::Golf,
            Family::TriPeaks => GameGroup::TriPeaks,
        }
    }
}

/// The families whose games are marked as popular.
const POPULAR: [Family; 4] = [
    Family::Klondike,
    Family::Spider,
    Family::FreeCell,
    Family::Pyramid,
];

/// Whether the game menu marks `variant` with the popularity star.
pub(super) fn is_popular(variant: Variant) -> bool {
    POPULAR.contains(&Family::of(variant))
}

/// The variants of `family` in the order of [`Variant::ALL`].
pub(super) fn variants_of(family: Family) -> impl Iterator<Item = Variant> {
    Variant::ALL
        .into_iter()
        .filter(move |variant| Family::of(*variant) == family)
}
