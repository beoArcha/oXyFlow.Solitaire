//! The one-line hint under the game list: the difficulty word and what the game asks for.

use oxysol_core::{DrawCount, PyramidRules, SuitCount, Variant};

use crate::i18n::{t, tf};

fn difficulty_key(variant: Variant) -> &'static str {
    match variant {
        Variant::Klondike(DrawCount::One)
        | Variant::Spider(SuitCount::One)
        | Variant::Pyramid(PyramidRules::Relaxed) => "menu.difficulty.easy",
        Variant::Klondike(DrawCount::Three)
        | Variant::Spider(SuitCount::Two)
        | Variant::FreeCell
        | Variant::Yukon(_)
        | Variant::Golf => "menu.difficulty.medium",
        Variant::Spider(SuitCount::Four) | Variant::Pyramid(PyramidRules::Classic) => {
            "menu.difficulty.hard"
        }
    }
}

const fn game_hint_key(variant: Variant) -> &'static str {
    match variant {
        Variant::Klondike(_) => "menu.game_hint.klondike",
        Variant::Spider(_) => "menu.game_hint.spider",
        Variant::FreeCell => "menu.game_hint.freecell",
        Variant::Pyramid(_) => "menu.game_hint.pyramid",
        Variant::Yukon(_) => "menu.game_hint.yukon",
        Variant::Golf => "menu.game_hint.golf",
    }
}

pub(super) fn hint_text(variant: Variant) -> String {
    let level = t(difficulty_key(variant)).into_owned();
    let description = t(game_hint_key(variant)).into_owned();
    tf(
        "menu.variant_hint",
        &[
            ("level", level.as_str()),
            ("description", description.as_str()),
        ],
    )
}
