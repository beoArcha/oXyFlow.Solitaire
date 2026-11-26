//! Depth-first search with a visited set: children in the order of [`Model::steps`], one clone per
//! child, memory proportional to the depth plus the visited keys.

use super::{Finisher, Model, Outcome, Searched, solution};
use crate::moves::Move;
use crate::solver::hash::seen_with_budget;

/// One level of the search: its children are `arena[start..start + len]`.
struct Frame<M: Model> {
    state: M,
    start: usize,
    len: usize,
    next: usize,
}

/// The children of every open frame, in stack order; one buffer for the whole search.
struct Arena<M: Model> {
    steps: Vec<M::Step>,
    scratch: Vec<M::Step>,
}

impl<M: Model> Arena<M> {
    fn frame(&mut self, state: M) -> Frame<M> {
        self.scratch.clear();
        state.steps(&mut self.scratch);
        let start = self.steps.len();
        self.steps.extend_from_slice(&self.scratch);
        Frame {
            state,
            start,
            len: self.scratch.len(),
            next: 0,
        }
    }

    fn step(&self, frame: &Frame<M>) -> Option<M::Step> {
        if frame.next < frame.len {
            self.steps.get(frame.start + frame.next).copied()
        } else {
            None
        }
    }
}

pub(super) fn search<M: Model>(
    root: &M,
    prefix: &[Move],
    budget: u64,
    finish: Finisher,
) -> Searched {
    let mut seen = seen_with_budget(budget);
    seen.insert(root.key());
    let mut nodes = 1u64;
    let mut arena = Arena {
        steps: Vec::new(),
        scratch: Vec::new(),
    };
    let mut stack = vec![arena.frame(root.clone())];
    let mut path: Vec<M::Step> = Vec::new();
    let mut sink = Vec::new();
    loop {
        let Some(top) = stack.last_mut() else {
            return Searched {
                outcome: Outcome::Exhausted,
                nodes,
            };
        };
        let Some(step) = arena.step(top) else {
            arena.steps.truncate(top.start);
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
        stack.push(arena.frame(child));
    }
}
