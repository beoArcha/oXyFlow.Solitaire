//! Greedy best-first search: the open position with the lowest [`Model::score`] is expanded next,
//! the newest first among equals. Every generated position stays in memory, so the budget also
//! bounds the memory.

use super::{Finisher, Model, Outcome, Searched, solution};
use crate::moves::Move;
use crate::solver::hash::seen_with_budget;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

struct Node<M: Model> {
    state: M,
    parent: usize,
    step: Option<M::Step>,
}

type Open = BinaryHeap<Reverse<(u32, Reverse<usize>)>>;

fn path_to<M: Model>(nodes: &[Node<M>], last: usize) -> Vec<M::Step> {
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

pub(super) fn search<M: Model>(
    root: &M,
    prefix: &[Move],
    budget: u64,
    finish: Finisher,
) -> Searched {
    let mut seen = seen_with_budget(budget);
    seen.insert(root.key());
    let mut nodes = vec![Node {
        state: root.clone(),
        parent: 0,
        step: None,
    }];
    let mut open = Open::new();
    open.push(Reverse((root.score(), Reverse(0))));
    let mut sink = Vec::new();
    let mut steps = Vec::new();
    while let Some(Reverse((_, Reverse(id)))) = open.pop() {
        steps.clear();
        let Some(parent) = nodes.get(id).map(|node| node.state.clone()) else {
            continue;
        };
        parent.steps(&mut steps);
        for &step in &steps {
            let mut child = parent.clone();
            sink.clear();
            child.apply(step, &mut sink);
            if !seen.insert(child.key()) {
                continue;
            }
            if nodes.len() as u64 >= budget {
                return Searched {
                    outcome: Outcome::OutOfBudget,
                    nodes: nodes.len() as u64,
                };
            }
            if child.is_won() || child.may_finish() {
                let mut path = path_to(&nodes, id);
                path.push(step);
                if let Some(moves) = solution(root, prefix, &path, &child, finish) {
                    return Searched {
                        outcome: Outcome::Won(moves),
                        nodes: nodes.len() as u64 + 1,
                    };
                }
            }
            open.push(Reverse((child.score(), Reverse(nodes.len()))));
            nodes.push(Node {
                state: child,
                parent: id,
                step: Some(step),
            });
        }
    }
    Searched {
        outcome: Outcome::Exhausted,
        nodes: nodes.len() as u64,
    }
}
