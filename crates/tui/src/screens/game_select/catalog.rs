//! The structure of the game menu: the families the games belong to, in a fixed order, and which
//! games carry the popularity star. A new game is placed by adding its family (or a variant to an
//! existing one) here; the compiler points at every `match` that has to learn it.

use oxysol_core::Variant;

/// A group of related games. The order of [`Family::ALL`] is the order of the menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Family {
    Klondike,
    Spider,
    FreeCell,
    Pyramid,
    Yukon,
}

impl Family {
    pub(super) const ALL: [Family; 5] = [
        Family::Klondike,
        Family::Spider,
        Family::FreeCell,
        Family::Pyramid,
        Family::Yukon,
    ];

    pub(super) const fn of(variant: Variant) -> Family {
        match variant {
            Variant::Klondike(_) => Family::Klondike,
            Variant::Spider(_) => Family::Spider,
            Variant::FreeCell => Family::FreeCell,
            Variant::Pyramid(_) => Family::Pyramid,
            Variant::Yukon(_) => Family::Yukon,
        }
    }

    pub(super) const fn title_key(self) -> &'static str {
        match self {
            Family::Klondike => "menu.family.klondike",
            Family::Spider => "menu.family.spider",
            Family::FreeCell => "menu.family.free_cell",
            Family::Pyramid => "menu.family.pyramid",
            Family::Yukon => "menu.family.yukon",
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
