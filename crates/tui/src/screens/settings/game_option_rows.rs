//! The rows of the game options (P6-06): the value each one shows and one step of change. An option
//! applies to the next game of its variant; every default keeps the rule the game always had.

use oxysol_core::{GameOptions, StockPasses};

use super::change::cycle;
use super::options::{Field, Step};
use crate::i18n::t;

const PASSES: [StockPasses; 3] = [StockPasses::Unlimited, StockPasses::Three, StockPasses::One];

const fn passes_key(passes: StockPasses) -> &'static str {
    match passes {
        StockPasses::Unlimited => "settings.game_option.passes_unlimited",
        StockPasses::Three => "settings.game_option.passes_three",
        StockPasses::One => "settings.game_option.passes_one",
    }
}

const fn choice_key(field: Field, on: bool) -> &'static str {
    match (field, on) {
        (Field::KlondikeEmptyColumn, false) => "settings.game_option.king_only",
        (Field::KlondikeEmptyColumn, true) => "settings.game_option.any_card",
        (Field::SpiderDeal, false) => "settings.game_option.columns_filled",
        (Field::SpiderDeal, true) => "settings.game_option.any_time",
        (Field::GolfWrap, false) => "settings.game_option.not_adjacent",
        (Field::GolfWrap, true) => "settings.game_option.adjacent",
        (Field::GolfKingStops, false) => "settings.game_option.queen_follows",
        (_, _) => "settings.game_option.nothing_follows",
    }
}

fn switch_of(field: Field, options: GameOptions) -> bool {
    match field {
        Field::KlondikeEmptyColumn => options.klondike_any_empty,
        Field::SpiderDeal => options.spider_deal_empty,
        Field::GolfWrap => options.golf_wrap,
        _ => options.golf_king_stops,
    }
}

/// The value shown for a game option row.
pub(super) fn value(field: Field, options: GameOptions) -> String {
    if field == Field::KlondikePasses {
        return t(passes_key(options.klondike_passes)).into_owned();
    }
    t(choice_key(field, switch_of(field, options))).into_owned()
}

/// `options` after one step on the row `field`.
pub(super) fn changed(field: Field, options: GameOptions, step: Step) -> GameOptions {
    let mut next = options;
    match field {
        Field::KlondikePasses => {
            next.klondike_passes = cycle(&PASSES, options.klondike_passes, step);
        }
        Field::KlondikeEmptyColumn => next.klondike_any_empty = !options.klondike_any_empty,
        Field::SpiderDeal => next.spider_deal_empty = !options.spider_deal_empty,
        Field::GolfWrap => next.golf_wrap = !options.golf_wrap,
        Field::GolfKingStops => next.golf_king_stops = !options.golf_king_stops,
        _ => {}
    }
    next
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROWS: [Field; 5] = [
        Field::KlondikePasses,
        Field::KlondikeEmptyColumn,
        Field::SpiderDeal,
        Field::GolfWrap,
        Field::GolfKingStops,
    ];

    #[test]
    fn passes_cycle_both_ways_through_unlimited_three_and_one() {
        let mut options = GameOptions::DEFAULT;
        let mut seen = Vec::new();
        for _ in 0..3 {
            options = changed(Field::KlondikePasses, options, Step::Next);
            seen.push(options.klondike_passes);
        }
        assert_eq!(
            seen,
            [StockPasses::Three, StockPasses::One, StockPasses::Unlimited]
        );
        let back = changed(Field::KlondikePasses, GameOptions::DEFAULT, Step::Previous);
        assert_eq!(back.klondike_passes, StockPasses::One);
    }

    #[test]
    fn every_switch_row_changes_only_its_own_option_and_back() {
        for field in ROWS.into_iter().skip(1) {
            let once = changed(field, GameOptions::DEFAULT, Step::Next);
            assert!(switch_of(field, once), "{field:?}");
            let others_default = ROWS
                .into_iter()
                .skip(1)
                .filter(|other| *other != field)
                .all(|other| !switch_of(other, once));
            assert!(others_default, "{field:?}");
            assert_eq!(
                changed(field, once, Step::Previous),
                GameOptions::DEFAULT,
                "{field:?}"
            );
        }
    }

    #[test]
    fn the_default_and_the_changed_value_read_differently() {
        for field in ROWS {
            let changed_value = value(field, changed(field, GameOptions::DEFAULT, Step::Next));
            assert_ne!(
                value(field, GameOptions::DEFAULT),
                changed_value,
                "{field:?}"
            );
        }
    }
}
