use super::*;
use crate::game::{Variant, YukonRules};
use crate::pile::PileSnapshot;
use crate::solver::search::{Outcome, Strategy, run};
use crate::solver::tests::{check_steps, no_finish, pile, run_of, same_moves, walk};

const FAMILY: [YukonRules; 5] = [
    YukonRules::Classic,
    YukonRules::Russian,
    YukonRules::Alaska,
    YukonRules::Moosehide,
    YukonRules::Australian,
];

fn make(rules: YukonRules) -> impl Fn(&dyn crate::game::Game) -> Option<Position> {
    move |game| Position::from_piles(rules, &game.piles())
}

fn normalized(position: &Position, mv: Move) -> Move {
    let Move::Transfer { from, to, count } = mv else {
        return mv;
    };
    let first_empty = position.len.iter().position(|&len| len == 0);
    let to = match to {
        PileRef::Foundation(_) => PileRef::Foundation(0),
        PileRef::Tableau(i) if position.len[usize::from(i)] == 0 => {
            tableau(first_empty.unwrap_or(0))
        }
        other => other,
    };
    Move::Transfer { from, to, count }
}

#[test]
fn direct_steps_are_the_engine_moves() {
    for rules in FAMILY {
        walk(Variant::Yukon(rules), |game, seed, history| {
            let position = Position::from_piles(rules, &game.piles()).unwrap();
            let mut steps = Vec::new();
            position.direct(&mut steps);
            let model = steps
                .iter()
                .filter_map(|&step| position.engine_move(step))
                .map(|mv| normalized(&position, mv))
                .collect();
            let engine = game
                .legal_moves()
                .into_iter()
                .filter(|&mv| mv != Move::Stock)
                .map(|mv| normalized(&position, mv))
                .collect();
            same_moves(
                model,
                engine,
                &format!("{rules:?} seed {seed} after {}", history.len()),
            );
        });
    }
}

#[test]
fn steps_play_legally_and_reach_the_engine_position() {
    for rules in FAMILY {
        check_steps(Variant::Yukon(rules), make(rules));
    }
}

fn other_suits_home() -> Vec<PileSnapshot> {
    vec![
        pile(PileRef::Foundation(0), "", &run_of('C', 1)),
        pile(PileRef::Foundation(1), "", &run_of('D', 1)),
        pile(PileRef::Foundation(2), "", &run_of('H', 1)),
    ]
}

/// Every card but the spades home; the ace lies under the two, and the three under a group whose
/// bottom fits nowhere, so no card can ever move.
fn buried_ace() -> Vec<PileSnapshot> {
    let mut piles = other_suits_home();
    piles.push(pile(PileRef::Tableau(0), "", "AS 2S"));
    piles.push(pile(
        PileRef::Tableau(1),
        "",
        "KS 3S QS JS 10S 9S 8S 7S 6S 5S 4S",
    ));
    piles
}

#[test]
fn a_position_with_no_way_out_is_exhausted() {
    for rules in [YukonRules::Classic, YukonRules::Russian] {
        let position = Position::from_piles(rules, &buried_ace()).unwrap();
        let searched = run(Strategy::DepthFirst, &position, 1_000, &no_finish);
        assert!(matches!(searched.outcome, Outcome::Exhausted), "{rules:?}");
    }
}

/// Every column taken, no spade onto a spade in Moosehide or Yukon, and the ace under the two.
fn spades_in_every_column() -> Vec<PileSnapshot> {
    let mut piles = other_suits_home();
    let columns = ["AS 2S", "3S 4S", "5S 6S", "7S 8S", "9S 10S", "JS QS", "KS"];
    for (column, cards) in columns.iter().enumerate() {
        let column = PileRef::Tableau(u8::try_from(column).unwrap());
        piles.push(pile(column, "", cards));
    }
    piles
}

#[test]
fn a_moosehide_position_with_no_way_out_is_exhausted() {
    for rules in [YukonRules::Moosehide, YukonRules::Classic] {
        let position = Position::from_piles(rules, &spades_in_every_column()).unwrap();
        let searched = run(Strategy::DepthFirst, &position, 1_000, &no_finish);
        assert!(matches!(searched.outcome, Outcome::Exhausted), "{rules:?}");
    }
}

#[test]
fn an_empty_column_frees_the_ace_and_wins() {
    let mut piles = other_suits_home();
    piles.push(pile(PileRef::Tableau(0), "AS", "KS"));
    piles.push(pile(
        PileRef::Tableau(1),
        "",
        "QS JS 10S 9S 8S 7S 6S 5S 4S 3S 2S",
    ));
    for rules in FAMILY {
        let position = Position::from_piles(rules, &piles).unwrap();
        let searched = run(Strategy::DepthFirst, &position, 1_000, &no_finish);
        assert!(matches!(searched.outcome, Outcome::Won(_)), "{rules:?}");
    }
}

#[test]
fn safety_follows_the_building_rule() {
    let piles = vec![
        pile(PileRef::Foundation(0), "", "AD 2D"),
        pile(PileRef::Tableau(0), "", "3S"),
        pile(PileRef::Tableau(1), "", "3D"),
    ];
    let safe = |rules: YukonRules, column: usize| {
        let position = Position::from_piles(rules, &piles).unwrap();
        position.is_safe(position.cols[column][0])
    };
    assert!(!safe(YukonRules::Classic, 0), "2H can still go onto 3S");
    assert!(safe(YukonRules::Russian, 1), "only 2D goes onto 3D");
    assert!(!safe(YukonRules::Alaska, 1), "4D can still go onto 3D");
    assert!(!safe(YukonRules::Moosehide, 1), "2S can still go onto 3D");
}

#[test]
fn the_stock_is_drawn_once_without_a_redeal() {
    let piles = vec![pile(PileRef::Stock, "AC 2C", "")];
    let mut position = Position::from_piles(YukonRules::Australian, &piles).unwrap();
    assert_eq!(position.talon.top_after(1), Some(1));
    assert_eq!(position.talon.top_after(2), Some(0));
    assert_eq!(position.talon.top_after(3), None);
    position.talon.draw();
    position.talon.draw();
    position.talon.draw();
    assert_eq!(position.talon.waste_top(), Some(0));
    assert_eq!(position.talon.stock_len, 0);
}

#[test]
fn column_order_does_not_change_the_key() {
    let a = vec![
        pile(PileRef::Tableau(0), "9C", "KS QH"),
        pile(PileRef::Tableau(1), "", "5D"),
    ];
    let b = vec![
        pile(PileRef::Tableau(3), "9C", "KS QH"),
        pile(PileRef::Tableau(6), "", "5D"),
    ];
    let key = |piles: &[PileSnapshot]| {
        Position::from_piles(YukonRules::Classic, piles)
            .unwrap()
            .key()
    };
    assert_eq!(key(&a), key(&b));
    let c = vec![
        pile(PileRef::Tableau(0), "9C", "KS QH"),
        pile(PileRef::Tableau(1), "", "5C"),
    ];
    assert_ne!(key(&a), key(&c));
}
