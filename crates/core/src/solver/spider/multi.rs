//! Search for 2- and 4-suit Spider:
//! 1. Fast weighted best-first (`PHASE1_CAP` nodes) for quick greedy wins.
//! 2. Fixed-width beam search (`BEAM_WIDTH`) for breadth across deals.
//! 3. Wider beam search (`WIDE_BEAM_WIDTH`) if the first beam empties early.
//! 4. Exploratory weighted best-first for any remaining budget.
//!
//! Because moves are pruned, giving up always returns `OutOfBudget` (`Unknown`), never `Exhausted`.

use super::multi_moves::{apply_step, build_solution, collect_pruned_moves, is_finished};
use super::position::Position;
use super::steps::Step;
use crate::moves::Move;
use crate::solver::hash::Seen;
use crate::solver::search::{Finisher, Outcome, Searched};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

const PHASE1_CAP: u64 = 15_000;
const PHASE1_G_WEIGHT: u32 = 4;
const PHASE1_CHILDREN: usize = 8;
const BEAM_WIDTH: usize = 50;
const BEAM_CHILDREN: usize = 5;
const WIDE_BEAM_WIDTH: usize = 120;
const WIDE_BEAM_CHILDREN: usize = 7;
const PHASE4_G_WEIGHT: u32 = 7;
const PHASE4_CHILDREN: usize = 12;

struct Node {
    state: Position,
    parent: usize,
    step: Option<Step>,
    depth: u32,
}

struct HistoryEntry {
    parent: usize,
    step: Option<Step>,
}

struct BeamItem {
    state: Position,
    history_id: usize,
}

struct BeamContext<'a> {
    start: &'a Position,
    used: u64,
    limit: u64,
    children_cap: usize,
    finish: Finisher<'a>,
    seen: Seen,
    history: Vec<HistoryEntry>,
    candidates: Vec<(u32, Step)>,
    sink: Vec<Move>,
}

type Open = BinaryHeap<Reverse<(u32, Reverse<usize>)>>;
type NextBeamEntry = (u32, Position, usize, Step);

fn path_to_node(nodes: &[Node], last: usize) -> Vec<Step> {
    let mut path = Vec::new();
    let mut at = last;
    while let Some(node) = nodes.get(at) {
        let Some(step) = node.step else {
            break;
        };
        path.push(step);
        at = node.parent;
    }
    path.reverse();
    path
}

fn path_to_history(history: &[HistoryEntry], last: usize) -> Vec<Step> {
    let mut path = Vec::new();
    let mut at = last;
    while let Some(entry) = history.get(at) {
        let Some(step) = entry.step else {
            break;
        };
        path.push(step);
        at = entry.parent;
    }
    path.reverse();
    path
}

/// Runs a multi-phase search on `start` up to `budget` generated positions.
pub(in crate::solver) fn search(start: &Position, budget: u64, finish: Finisher) -> Searched {
    if budget == 0 {
        return Searched {
            outcome: Outcome::OutOfBudget,
            nodes: 0,
        };
    }
    if let Some(moves) = build_solution(start, &[], start, finish) {
        return Searched {
            outcome: Outcome::Won(moves),
            nodes: 1,
        };
    }
    if budget == 1 || start.is_dead() {
        return Searched {
            outcome: Outcome::OutOfBudget,
            nodes: 1,
        };
    }
    let phase1 = run_weighted_best_first(
        start,
        1,
        budget.min(PHASE1_CAP),
        PHASE1_G_WEIGHT,
        PHASE1_CHILDREN,
        finish,
    );
    if matches!(phase1.outcome, Outcome::Won(_)) || phase1.nodes >= budget {
        return phase1;
    }
    let phase2 = run_beam(
        start,
        phase1.nodes,
        budget,
        BEAM_WIDTH,
        BEAM_CHILDREN,
        finish,
    );
    if matches!(phase2.outcome, Outcome::Won(_)) || phase2.nodes >= budget {
        return phase2;
    }
    let phase3 = run_beam(
        start,
        phase2.nodes,
        budget,
        WIDE_BEAM_WIDTH,
        WIDE_BEAM_CHILDREN,
        finish,
    );
    if matches!(phase3.outcome, Outcome::Won(_)) || phase3.nodes >= budget {
        return phase3;
    }
    run_weighted_best_first(
        start,
        phase3.nodes,
        budget,
        PHASE4_G_WEIGHT,
        PHASE4_CHILDREN,
        finish,
    )
}

fn run_weighted_best_first(
    start: &Position,
    mut used: u64,
    limit: u64,
    g_weight: u32,
    children_cap: usize,
    finish: Finisher,
) -> Searched {
    let mut seen = Seen::default();
    seen.insert(start.key());
    let mut nodes = vec![Node {
        state: start.clone(),
        parent: 0,
        step: None,
        depth: 0,
    }];
    let mut open = Open::new();
    open.push(Reverse((start.work_left(), Reverse(0))));
    let mut sink = Vec::new();
    let mut candidates = Vec::new();
    while let Some(Reverse((_, Reverse(id)))) = open.pop() {
        let Some(parent_depth) = nodes.get(id).map(|n| n.depth) else {
            continue;
        };
        let parent = nodes[id].state.clone();
        collect_pruned_moves(&parent, children_cap, &mut candidates);
        for &(_, step) in &candidates {
            let mut child = parent.clone();
            sink.clear();
            apply_step(&mut child, step, &mut sink);
            if child.is_dead() || !seen.insert(child.key()) {
                continue;
            }
            if used >= limit {
                return Searched {
                    outcome: Outcome::OutOfBudget,
                    nodes: used,
                };
            }
            used += 1;
            if is_finished(&child) {
                let mut path = path_to_node(&nodes, id);
                path.push(step);
                if let Some(moves) = build_solution(start, &path, &child, finish) {
                    return Searched {
                        outcome: Outcome::Won(moves),
                        nodes: used,
                    };
                }
            }
            let child_depth = parent_depth + 1;
            let priority = child_depth
                .saturating_mul(g_weight)
                .saturating_add(child.work_left());
            open.push(Reverse((priority, Reverse(nodes.len()))));
            nodes.push(Node {
                state: child,
                parent: id,
                step: Some(step),
                depth: child_depth,
            });
        }
    }
    Searched {
        outcome: Outcome::OutOfBudget,
        nodes: used,
    }
}

fn expand_beam_item(
    ctx: &mut BeamContext<'_>,
    item: &BeamItem,
    next_beam: &mut Vec<NextBeamEntry>,
) -> Option<Searched> {
    collect_pruned_moves(&item.state, ctx.children_cap, &mut ctx.candidates);
    for &(_, step) in &ctx.candidates {
        let mut child = item.state.clone();
        ctx.sink.clear();
        apply_step(&mut child, step, &mut ctx.sink);
        if child.is_dead() || !ctx.seen.insert(child.key()) {
            continue;
        }
        if ctx.used >= ctx.limit {
            return Some(Searched {
                outcome: Outcome::OutOfBudget,
                nodes: ctx.used,
            });
        }
        ctx.used += 1;
        if is_finished(&child) {
            let mut path = path_to_history(&ctx.history, item.history_id);
            path.push(step);
            if let Some(moves) = build_solution(ctx.start, &path, &child, ctx.finish) {
                return Some(Searched {
                    outcome: Outcome::Won(moves),
                    nodes: ctx.used,
                });
            }
        }
        next_beam.push((child.work_left(), child, item.history_id, step));
    }
    None
}

fn run_beam(
    start: &Position,
    used: u64,
    limit: u64,
    width: usize,
    children_cap: usize,
    finish: Finisher,
) -> Searched {
    let mut seen = Seen::default();
    seen.insert(start.key());
    let mut ctx = BeamContext {
        start,
        used,
        limit,
        children_cap,
        finish,
        seen,
        history: vec![HistoryEntry {
            parent: 0,
            step: None,
        }],
        candidates: Vec::new(),
        sink: Vec::new(),
    };
    let mut beam = vec![BeamItem {
        state: start.clone(),
        history_id: 0,
    }];
    while !beam.is_empty() {
        let mut next_beam = Vec::new();
        for item in &beam {
            if let Some(done) = expand_beam_item(&mut ctx, item, &mut next_beam) {
                return done;
            }
        }
        next_beam.sort_unstable_by_key(|entry| entry.0);
        next_beam.truncate(width);
        beam.clear();
        for (_, state, parent, step) in next_beam {
            let history_id = ctx.history.len();
            ctx.history.push(HistoryEntry {
                parent,
                step: Some(step),
            });
            beam.push(BeamItem { state, history_id });
        }
    }
    Searched {
        outcome: Outcome::OutOfBudget,
        nodes: ctx.used,
    }
}
