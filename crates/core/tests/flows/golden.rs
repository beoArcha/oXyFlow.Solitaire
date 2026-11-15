//! Golden deals: the full initial layout of every variant for seed 42. A change to the shuffle,
//! the deck order or the dealing breaks these tests, and with them every saved game (T-03, R-18).
//! Cards are `7H`, a `*` marks a face-down card, piles run bottom to top.

use crate::common::{SEED, layout};
use oxysol_core::{DrawCount, PyramidRules, SuitCount, Variant, deal};

fn assert_golden(variant: Variant, expected: &str) {
    assert_eq!(
        layout(&deal(variant, SEED).piles()),
        expected,
        "{variant:?}"
    );
}

const KLONDIKE_DRAW_ONE: &str = "\
Stock: 4H* 3D* 10H* JS* 4C* 6S* 7S* 3S* 8D* QD* 9S* KH* 5H* 6C* QS* 9D* JD* QC* 3C* 9H* QH* 9C* JC* 10C*
Waste:
Foundation 0:
Foundation 1:
Foundation 2:
Foundation 3:
Tableau 0: 7C
Tableau 1: 3H* KC
Tableau 2: AS* 2S* AC
Tableau 3: AH* KD* 10D* 5C
Tableau 4: 8S* JH* 2H* 4S* 6H
Tableau 5: 5D* 2D* 5S* 6D* AD* 7D
Tableau 6: 2C* 8H* 8C* 4D* 7H* KS* 10S";

const KLONDIKE_DRAW_THREE: &str = "\
Stock: 4H* 3D* 10H* JS* 4C* 6S* 7S* 3S* 8D* QD* 9S* KH* 5H* 6C* QS* 9D* JD* QC* 3C* 9H* QH* 9C* JC* 10C*
Waste:
Foundation 0:
Foundation 1:
Foundation 2:
Foundation 3:
Tableau 0: 7C
Tableau 1: 3H* KC
Tableau 2: AS* 2S* AC
Tableau 3: AH* KD* 10D* 5C
Tableau 4: 8S* JH* 2H* 4S* 6H
Tableau 5: 5D* 2D* 5S* 6D* AD* 7D
Tableau 6: 2C* 8H* 8C* 4D* 7H* KS* 10S";

const SPIDER_ONE_SUIT: &str = "\
Stock: 9S* KS* AS* QS* 5S* JS* 4S* 6S* 8S* 3S* 9S* 2S* 3S* 10S* 5S* KS* 8S* 5S* 2S* 2S* JS* AS* 5S* 4S* 6S* 7S* KS* 9S* JS* 10S* 6S* 10S* AS* 7S* 4S* 6S* 9S* AS* 3S* QS* QS* 8S* AS* KS* 4S* QS* 6S* 7S* 7S* 10S*
Foundation 0:
Foundation 1:
Foundation 2:
Foundation 3:
Foundation 4:
Foundation 5:
Foundation 6:
Foundation 7:
Tableau 0: QS* 6S* 9S* 5S* AS* 10S
Tableau 1: 8S* 4S* 5S* 8S* 3S* JS
Tableau 2: 8S* QS* AS* QS* 5S* 7S
Tableau 3: JS* KS* 9S* 3S* 10S* AS
Tableau 4: 9S* 5S* 4S* 4S* JS
Tableau 5: 8S* 6S* JS* 9S* QS
Tableau 6: 3S* 7S* 10S* 2S* JS
Tableau 7: 3S* 8S* 10S* 3S* KS
Tableau 8: 7S* KS* 2S* 2S* KS
Tableau 9: 2S* 7S* 4S* 6S* 2S";

const SPIDER_TWO_SUITS: &str = "\
Stock: 9H* KS* AS* QH* 5S* JH* 4S* 6H* 8S* 3S* 9S* 2H* 3S* 10S* 5H* KH* 8H* 5H* 2S* 2S* JS* AH* 5S* 4S* 6S* 7S* KS* 9H* JS* 10H* 6S* 10H* AS* 7H* 4H* 6H* 9S* AS* 3H* QH* QS* 8S* AH* KH* 4H* QH* 6S* 7S* 7H* 10S*
Foundation 0:
Foundation 1:
Foundation 2:
Foundation 3:
Foundation 4:
Foundation 5:
Foundation 6:
Foundation 7:
Tableau 0: QH* 6S* 9S* 5S* AH* 10H
Tableau 1: 8H* 4S* 5S* 8S* 3S* JS
Tableau 2: 8S* QS* AS* QS* 5H* 7S
Tableau 3: JH* KS* 9H* 3H* 10S* AH
Tableau 4: 9S* 5H* 4H* 4S* JH
Tableau 5: 8H* 6H* JS* 9H* QS
Tableau 6: 3S* 7H* 10S* 2H* JH
Tableau 7: 3H* 8H* 10H* 3H* KH
Tableau 8: 7S* KS* 2H* 2S* KH
Tableau 9: 2H* 7H* 4H* 6H* 2S";

const SPIDER_FOUR_SUITS: &str = "\
Stock: 9S* KH* AC* QD* 5C* JD* 4C* 6D* 8C* 3H* 9H* 2S* 3C* 10H* 5S* KD* 8D* 5S* 2H* 2C* JH* AS* 5H* 4C* 6H* 7H* KC* 9D* JC* 10S* 6H* 10S* AC* 7S* 4D* 6S* 9C* AH* 3S* QS* QH* 8H* AS* KS* 4S* QS* 6C* 7C* 7D* 10C*
Foundation 0:
Foundation 1:
Foundation 2:
Foundation 3:
Foundation 4:
Foundation 5:
Foundation 6:
Foundation 7:
Tableau 0: QD* 6C* 9C* 5H* AD* 10D
Tableau 1: 8S* 4H* 5C* 8H* 3H* JC
Tableau 2: 8C* QC* AH* QC* 5D* 7H
Tableau 3: JD* KH* 9D* 3D* 10C* AD
Tableau 4: 9H* 5D* 4S* 4H* JS
Tableau 5: 8D* 6S* JH* 9S* QH
Tableau 6: 3C* 7D* 10H* 2D* JS
Tableau 7: 3S* 8S* 10D* 3D* KS
Tableau 8: 7C* KC* 2D* 2C* KD
Tableau 9: 2S* 7S* 4D* 6D* 2H";

const FREECELL: &str = "\
Foundation 0:
Foundation 1:
Foundation 2:
Foundation 3:
Cell 0:
Cell 1:
Cell 2:
Cell 3:
Tableau 0: 7C 3H KC AS 2S AC AH
Tableau 1: KD 10D 5C 8S JH 2H 4S
Tableau 2: 6H 5D 2D 5S 6D AD 7D
Tableau 3: 2C 8H 8C 4D 7H KS 10S
Tableau 4: 4H 3D 10H JS 4C 6S
Tableau 5: 7S 3S 8D QD 9S KH
Tableau 6: 5H 6C QS 9D JD QC
Tableau 7: 3C 9H QH 9C JC 10C";

const PYRAMID_RELAXED: &str = "\
Stock: 4H* 3D* 10H* JS* 4C* 6S* 7S* 3S* 8D* QD* 9S* KH* 5H* 6C* QS* 9D* JD* QC* 3C* 9H* QH* 9C* JC* 10C
Waste:
Foundation 0:
Pyramid 0.0: 7C
Pyramid 1.0: 3H
Pyramid 1.1: KC
Pyramid 2.0: AS
Pyramid 2.1: 2S
Pyramid 2.2: AC
Pyramid 3.0: AH
Pyramid 3.1: KD
Pyramid 3.2: 10D
Pyramid 3.3: 5C
Pyramid 4.0: 8S
Pyramid 4.1: JH
Pyramid 4.2: 2H
Pyramid 4.3: 4S
Pyramid 4.4: 6H
Pyramid 5.0: 5D
Pyramid 5.1: 2D
Pyramid 5.2: 5S
Pyramid 5.3: 6D
Pyramid 5.4: AD
Pyramid 5.5: 7D
Pyramid 6.0: 2C
Pyramid 6.1: 8H
Pyramid 6.2: 8C
Pyramid 6.3: 4D
Pyramid 6.4: 7H
Pyramid 6.5: KS
Pyramid 6.6: 10S";

const PYRAMID_CLASSIC: &str = "\
Stock: 4H* 3D* 10H* JS* 4C* 6S* 7S* 3S* 8D* QD* 9S* KH* 5H* 6C* QS* 9D* JD* QC* 3C* 9H* QH* 9C* JC* 10C
Waste:
Foundation 0:
Pyramid 0.0: 7C
Pyramid 1.0: 3H
Pyramid 1.1: KC
Pyramid 2.0: AS
Pyramid 2.1: 2S
Pyramid 2.2: AC
Pyramid 3.0: AH
Pyramid 3.1: KD
Pyramid 3.2: 10D
Pyramid 3.3: 5C
Pyramid 4.0: 8S
Pyramid 4.1: JH
Pyramid 4.2: 2H
Pyramid 4.3: 4S
Pyramid 4.4: 6H
Pyramid 5.0: 5D
Pyramid 5.1: 2D
Pyramid 5.2: 5S
Pyramid 5.3: 6D
Pyramid 5.4: AD
Pyramid 5.5: 7D
Pyramid 6.0: 2C
Pyramid 6.1: 8H
Pyramid 6.2: 8C
Pyramid 6.3: 4D
Pyramid 6.4: 7H
Pyramid 6.5: KS
Pyramid 6.6: 10S";

#[test]
fn klondike_draw_one_deal_is_pinned() {
    assert_golden(Variant::Klondike(DrawCount::One), KLONDIKE_DRAW_ONE);
}

#[test]
fn klondike_draw_three_deal_is_pinned() {
    assert_golden(Variant::Klondike(DrawCount::Three), KLONDIKE_DRAW_THREE);
}

#[test]
fn spider_one_suit_deal_is_pinned() {
    assert_golden(Variant::Spider(SuitCount::One), SPIDER_ONE_SUIT);
}

#[test]
fn spider_two_suits_deal_is_pinned() {
    assert_golden(Variant::Spider(SuitCount::Two), SPIDER_TWO_SUITS);
}

#[test]
fn spider_four_suits_deal_is_pinned() {
    assert_golden(Variant::Spider(SuitCount::Four), SPIDER_FOUR_SUITS);
}

#[test]
fn freecell_deal_is_pinned() {
    assert_golden(Variant::FreeCell, FREECELL);
}

#[test]
fn pyramid_relaxed_deal_is_pinned() {
    assert_golden(Variant::Pyramid(PyramidRules::Relaxed), PYRAMID_RELAXED);
}

#[test]
fn pyramid_classic_deal_is_pinned() {
    assert_golden(Variant::Pyramid(PyramidRules::Classic), PYRAMID_CLASSIC);
}
