//! Depth-first search with a visited set: children in the order of [`Model::steps`], one clone per
//! child, memory proportional to the depth plus the visited keys.

use super::{Finisher, Model, Outcome, Searched, solution};
use crate::moves::Move;
use crate::solver::hash::Seen;

struct Frame<M: Model> {
    state: M,
    steps: Vec<M::Step>,
    next: usize,
}

fn frame<M: Model>(state: M) -> Frame<M> {
    let mut steps = Vec::new();
    state.steps(&mut steps);
    Frame {
        state,
        steps,
        next: 0,
    }
}

pub(super) fn search<M: Model>(
    root: &M,
    prefix: &[Move],
    budget: u64,
    finish: Finisher,
) -> Searched {
    let mut seen = Seen::default();
    seen.insert(root.key());
    let mut nodes = 1u64;
    let mut stack = vec![frame(root.clone())];
    let mut path: Vec<M::Step> = Vec::new();
    let mut sink = Vec::new();
    loop {
        let Some(top) = stack.last_mut() else {
            return Searched {
                outcome: Outcome::Exhausted,
                nodes,
            };
        };
        let Some(&step) = top.steps.get(top.next) else {
            stack.pop();
            path.pop();
            continue;
        };
        top.next += 1;
        let mut child = top.state.clone();
        sink.clear();
        child.apply(step, &mut sink);
        if !seen.insert(child.key()) {
            continue;
        }
        if nodes >= budget {
            return Searched {
                outcome: Outcome::OutOfBudget,
                nodes,
            };
        }
        nodes += 1;
        path.push(step);
        if let Some(moves) = solution(root, prefix, &path, &child, finish) {
            return Searched {
                outcome: Outcome::Won(moves),
                nodes,
            };
        }
        stack.push(frame(child));
    }
}
