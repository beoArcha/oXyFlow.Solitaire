//! Move scoring, pruning, and execution helpers for the 2- and 4-suit Spider search.

use super::position::{COLUMNS, Position, RUN};
use super::steps::Step;
use super::tableau;
use crate::moves::Move;
use crate::solver::face::{Face, rank, suit};
use crate::solver::search::Finisher;
use std::cmp::Reverse;

fn index(value: usize) -> u8 {
    u8::try_from(value).unwrap_or(u8::MAX)
}

pub(super) fn is_finished(pos: &Position) -> bool {
    pos.is_won() || may_finish(pos)
}

pub(super) fn may_finish(pos: &Position) -> bool {
    pos.stock_len == 0
        && (0..COLUMNS)
            .all(|column| pos.hidden[column] == 0 && pos.run_len(column) == pos.len[column])
}

pub(super) fn apply_step(pos: &mut Position, step: Step, moves: &mut Vec<Move>) {
    match step {
        Step::Deal => {
            moves.push(Move::Stock);
            pos.deal_row();
        }
        Step::Run { from, to, count } => {
            let (from, to, count) = (usize::from(from), usize::from(to), usize::from(count));
            moves.push(Move::Transfer {
                from: tableau(from),
                to: tableau(to),
                count: index(count),
            });
            pos.move_run(from, to, count);
        }
    }
}

pub(super) fn build_solution(
    root: &Position,
    steps: &[Step],
    at: &Position,
    finish: Finisher,
) -> Option<Vec<Move>> {
    let won = at.is_won();
    if !won && !may_finish(at) {
        return None;
    }
    let mut moves = Vec::new();
    let mut state = root.clone();
    for &step in steps {
        apply_step(&mut state, step, &mut moves);
    }
    if won {
        return Some(moves);
    }
    moves.extend(finish(&moves)?);
    Some(moves)
}

/// Collects the top `cap` pruned transfer moves by heuristic score, plus `Deal` if legal.
pub(super) fn collect_pruned_moves(pos: &Position, cap: usize, out: &mut Vec<(u32, Step)>) {
    out.clear();
    let first_empty = pos.first_empty_column();
    let mut run_lens = [0usize; COLUMNS];
    for (col, slot) in run_lens.iter_mut().enumerate() {
        *slot = pos.run_len(col);
    }
    for from in 0..COLUMNS {
        if pos.len[from] == 0 {
            continue;
        }
        collect_from_column(pos, from, &run_lens, first_empty, out);
    }
    out.sort_unstable_by_key(|&(score, _)| Reverse(score));
    out.truncate(cap);
    if pos.can_deal() {
        out.push((150, Step::Deal));
    }
}

fn collect_from_column(
    pos: &Position,
    from: usize,
    run_lens: &[usize; COLUMNS],
    first_empty: Option<usize>,
    out: &mut Vec<(u32, Step)>,
) {
    let max_run = run_lens[from];
    for count in 1..=max_run {
        let start = pos.len[from] - count;
        let card = pos.cols[from][start];
        let is_whole = count == max_run;
        for to in 0..COLUMNS {
            if to == from || !pos.accepts(to, card) {
                continue;
            }
            if let Some(score) =
                score_candidate(pos, from, to, start, count, is_whole, run_lens, first_empty)
            {
                out.push((
                    score,
                    Step::Run {
                        from: index(from),
                        to: index(to),
                        count: index(count),
                    },
                ));
            }
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "move scoring reads already-computed run context without reallocating"
)]
fn score_candidate(
    pos: &Position,
    from: usize,
    to: usize,
    start: usize,
    count: usize,
    is_whole: bool,
    run_lens: &[usize; COLUMNS],
    first_empty: Option<usize>,
) -> Option<u32> {
    let to_empty = pos.len[to] == 0;
    if to_empty && first_empty != Some(to) {
        return None;
    }
    if to_empty && count == pos.len[from] && pos.hidden[from] == 0 {
        return None;
    }
    let card: Face = pos.cols[from][start];
    let joins = pos.top(to).is_some_and(|top| suit(top) == suit(card));
    if !is_whole && !joins {
        return None;
    }
    let reveals = start == pos.hidden[from] && pos.hidden[from] > 0;
    let empties = start == 0 && pos.hidden[from] == 0;
    let chain = if joins { run_lens[to] + count } else { count };
    let score = if joins && chain >= RUN {
        4000
    } else if joins && is_whole {
        2000 + 20 * u32::try_from(chain).unwrap_or(0) + bonus(reveals, empties)
    } else if reveals {
        1500 + u32::try_from(pos.hidden[from]).unwrap_or(0)
    } else if empties {
        1100
    } else if joins {
        900 + 10 * u32::try_from(chain).unwrap_or(0)
    } else if to_empty {
        300
    } else {
        base_off_suit_score(pos, from, start, card)
    };
    Some(score)
}

fn bonus(reveals: bool, empties: bool) -> u32 {
    u32::from(reveals) * 300 + u32::from(empties) * 200
}

fn base_off_suit_score(pos: &Position, from: usize, start: usize, card: Face) -> u32 {
    let beneath = pos.cols[from][start - 1];
    if rank(beneath) == rank(card) + 1 {
        100
    } else if rank(beneath) < rank(card) {
        450
    } else {
        350
    }
}
