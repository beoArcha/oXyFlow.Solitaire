//! The `Now:` row at the narrowest board width beside each side panel: no footer text on it, the
//! source column kept while carrying, and the fixed order in which `Do:` hints are dropped.

use oxysol_core::{DrawCount, Move, PileRef, Variant};
use ratatui::crossterm::event::KeyCode;

use super::*;
use crate::action::Action;
use crate::announce::pile_name;
use crate::app::Modal;
use crate::i18n::{Locale, set_locale, t, tf};
use crate::input::primary_key;
use crate::screens::Effect;
use crate::settings::Settings;

const NARROWEST_SIZES: [(u16, u16); 2] = [(100, 30), (130, 40)];
const PANELS: [(bool, bool); 3] = [(true, false), (false, true), (true, true)];
const SEEDS: [u64; 8] = [5, 6, 7, 8, 9, 10, 11, 12];

fn game_in(locale: Locale, seed: u64, (width, height): (u16, u16)) -> Harness {
    let storage = MemoryStorage {
        settings: Some(Settings {
            locale,
            ..Settings::default()
        }),
        ..MemoryStorage::default()
    };
    let mut h = Harness::with(storage, TRUE_COLOR, width, height);
    h.app.apply(Effect::Start {
        variant: Variant::Klondike(DrawCount::One),
        seed: Some(seed),
        undo_allowed: true,
    });
    set_locale(locale);
    h.lines();
    h
}

fn open_panels(h: &mut Harness, (guide, moves): (bool, bool)) {
    if h.app.aids().guide_panel != guide {
        h.type_char('g');
    }
    if h.app.aids().moves_panel != moves {
        h.type_char('v');
    }
    assert_eq!(h.app.aids().guide_panel, guide);
    assert_eq!(h.app.aids().moves_panel, moves);
}

/// Picks up the `which`-th tableau card that can move and drops it where it cannot go, so that the
/// board carries it with a refusal shown. Returns the source.
fn carry_with_refusal(h: &mut Harness, which: usize) -> Option<PileRef> {
    let sources: Vec<(PileRef, u8)> = legal(h)
        .into_iter()
        .filter_map(|mv| match mv {
            Move::Transfer {
                from: from @ PileRef::Tableau(_),
                count,
                ..
            } => Some((from, count)),
            _ => None,
        })
        .collect();
    let (from, count) = *sources.get(which)?;
    put_cursor(h, from);
    h.press(KeyCode::Enter);
    let refused = (0..7).map(PileRef::Tableau).find(|pile| {
        *pile != from
            && !legal(h).contains(&Move::Transfer {
                from,
                to: *pile,
                count,
            })
    })?;
    put_cursor(h, refused);
    h.press(KeyCode::Enter);
    status(h)
        .is_some_and(|key| key.starts_with("board.error."))
        .then_some(from)
}

/// The `Now:` and `Do:` rows as far as the board reaches, without the side column.
fn controls(h: &mut Harness) -> (String, String) {
    let width = usize::from(board(h).chrome().now.width);
    let lines = h.lines();
    let cut = |line: &String| line.chars().take(width).collect::<String>();
    (cut(&lines[lines.len() - 2]), cut(&lines[lines.len() - 1]))
}

fn hint_text(action: Action, key: &str) -> String {
    tf(key, &[("key", &primary_key(action).unwrap_or_default())])
}

fn short_name(pile: PileRef) -> String {
    let PileRef::Tableau(index) = pile else {
        return pile_name(pile);
    };
    tf(
        "board.name.tableau_short",
        &[("number", &(u16::from(index) + 1).to_string())],
    )
}

fn each_case(check: impl Fn(Locale, (u16, u16), (bool, bool), &mut Harness, PileRef)) {
    let mut checked = 0;
    for locale in Locale::offered() {
        for size in NARROWEST_SIZES {
            for panels in PANELS {
                if size.0 < 130 && panels != (true, true) {
                    continue;
                }
                for seed in SEEDS {
                    for which in 0..3 {
                        let mut h = game_in(locale, seed, size);
                        if size.0 >= 130 {
                            open_panels(&mut h, panels);
                        }
                        if let Some(from) = carry_with_refusal(&mut h, which) {
                            check(locale, size, panels, &mut h, from);
                            checked += 1;
                        }
                    }
                }
            }
        }
    }
    set_locale(Locale::EnUs);
    assert!(checked > 100, "only {checked} cases were checked");
}

#[test]
fn the_now_row_holds_no_footer_text_while_carrying_with_a_refusal() {
    each_case(|locale, size, panels, h, _| {
        let (now, _) = controls(h);
        for (action, key) in [
            (Action::Pause, "board.action.pause"),
            (Action::Moves, "board.action.moves"),
            (Action::Options, "board.action.options"),
            (Action::Restart, "board.action.restart"),
        ] {
            let text = hint_text(action, key);
            assert!(
                !now.contains(&text),
                "{locale:?} {size:?} {panels:?}: {text:?} in {now:?}"
            );
        }
    });
}

#[test]
fn the_source_column_stays_visible_while_carrying_with_a_refusal() {
    each_case(|locale, size, panels, h, from| {
        let (now, _) = controls(h);
        let long = pile_name(from);
        let short = short_name(from);
        assert!(
            now.contains(&long) || now.contains(&short),
            "{locale:?} {size:?} {panels:?}: {long} / {short} not in {now:?}"
        );
    });
}

#[test]
fn a_narrow_now_row_uses_the_compact_column_name_before_it_cuts_anything() {
    let mut h = game_in(Locale::EnUs, 5, (100, 30));
    let Some((from, _, _)) = first_transfer(&h) else {
        panic!("seed 5 has a transfer");
    };
    put_cursor(&mut h, from);
    h.press(KeyCode::Enter);
    let Some(guidance) = board(&h).guidance() else {
        panic!("a carried card has guidance");
    };
    let long = pile_name(from);
    let short = short_name(from);
    let full = guidance.now_line(200);
    assert!(full.contains(&long), "{full}");
    let compact = guidance.now_line(full.chars().count() - 1);
    assert!(
        compact.contains(&short) && !compact.contains(&long),
        "{compact}"
    );
    let tightest = guidance.now_line(compact.chars().count() - 1);
    assert!(tightest.contains(&short), "{tightest}");
    assert!(
        tightest.chars().count() < compact.chars().count(),
        "{tightest}"
    );
}

/// The hints of a `Do:` row in the order a narrowing row loses them.
fn drop_sequence(h: &Harness) -> Vec<String> {
    let Some(guidance) = board(h).guidance() else {
        panic!("the board has guidance");
    };
    let parts = |width: usize| -> Vec<String> {
        guidance
            .do_segments(width)
            .into_iter()
            .map(|segment| segment.text)
            .filter(|text| !matches!(text.trim(), "" | "|" | "Do:"))
            .collect()
    };
    let mut sequence = Vec::new();
    let mut shown = parts(300);
    for width in (0..300).rev() {
        let now = parts(width);
        sequence.extend(shown.iter().filter(|part| !now.contains(part)).cloned());
        shown = now;
    }
    sequence
}

#[test]
fn the_do_row_drops_its_hints_in_a_fixed_order() {
    let mut h = game_in(Locale::EnUs, 5, (100, 30));
    let Some((from, to, _)) = first_transfer(&h) else {
        panic!("seed 5 has a transfer");
    };
    drag(&mut h, from, to);
    let run_keys = [Action::RunMore, Action::RunLess]
        .into_iter()
        .filter_map(primary_key)
        .collect::<Vec<_>>()
        .join("/");
    let expected = vec![
        hint_text(Action::Rules, "board.action.rules"),
        t("board.action.move").into_owned(),
        hint_text(Action::Help, "board.action.help"),
        hint_text(Action::Guide, "board.action.guide"),
        hint_text(Action::Hint, "board.action.hint"),
        hint_text(Action::Undo, "board.action.undo"),
        tf("board.action.run", &[("keys", &run_keys)]),
        hint_text(Action::Options, "board.action.options"),
        hint_text(Action::Restart, "board.action.restart"),
        hint_text(Action::Stock, "board.action.turn_stock"),
        hint_text(Action::Moves, "board.action.moves"),
        hint_text(Action::Pause, "board.action.pause"),
        hint_text(Action::Back, "board.action.menu"),
    ];
    assert_eq!(drop_sequence(&h), expected);
}

/// A 100-column board in `locale` after one move, or carrying a card with a refusal shown.
fn narrow_state(locale: Locale, carrying: bool) -> Harness {
    let mut h = game_in(locale, 5, (100, 30));
    if carrying {
        let Some(_) = carry_with_refusal(&mut h, 0) else {
            panic!("seed 5 has a tableau card that can move and a refusing column");
        };
        return h;
    }
    let Some((from, to, _)) = first_transfer(&h) else {
        panic!("seed 5 has a transfer");
    };
    drag(&mut h, from, to);
    h
}

#[test]
fn pause_and_moves_stay_clickable_at_100_columns_after_a_move_and_while_carrying() {
    for locale in Locale::offered() {
        for carrying in [false, true] {
            for (action, key) in [
                (Action::Pause, "board.action.pause"),
                (Action::Moves, "board.action.moves"),
            ] {
                let mut h = narrow_state(locale, carrying);
                let (now, possible) = controls(&mut h);
                let text = hint_text(action, key);
                let case = format!("{locale:?} carrying={carrying} {text:?}");
                assert!(possible.contains(&text), "{case}: not in {possible:?}");
                assert!(!now.contains(&text), "{case}: on the Now row {now:?}");
                let Some((column, row)) = h.find(&text) else {
                    panic!("{case}: not drawn");
                };
                h.click(column + 1, row);
                let opened = match action {
                    Action::Pause => matches!(h.app.modal(), Some(Modal::Pause { .. })),
                    _ => matches!(h.app.modal(), Some(Modal::Moves(_))),
                };
                assert!(opened, "{case}: the click did not run {action:?}");
            }
        }
    }
    set_locale(Locale::EnUs);
}
