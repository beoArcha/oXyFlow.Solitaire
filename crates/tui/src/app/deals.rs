//! Deals and the solver: the search for a winnable deal behind a dialog, and the background check
//! of a deal the player chose (a typed seed, a replay, a resumed game). A seed the player chose is
//! never replaced; the header only reports what the solver found out about it.

use oxysol_core::Variant;

use super::search::{SEED_CAP, Search, SearchCommand, Step};
use super::{App, Modal};
use crate::solver::{Answer, DealCheck, Proof, Query};

/// A deal being checked in the background.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Check {
    id: u64,
    seed: u64,
}

/// Where the seed of a new game came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Origin {
    /// A random seed of a game that did not ask for a winnable deal: nothing is known.
    Fresh,
    /// A seed the solver proved or could not decide during a search.
    Searched(Proof),
    /// A seed the player chose: it is dealt as given and checked in the background.
    Chosen,
}

impl App {
    fn next_query_id(&mut self) -> u64 {
        let id = self.next_query;
        self.next_query += 1;
        id
    }

    fn submit_batch(&mut self) {
        let Some(variant) = self.search.as_ref().map(|search| search.variant) else {
            return;
        };
        let first_id = self.next_query;
        let seeds: Vec<u64> = (0..SEED_CAP).map(|_| (self.seeds)()).collect();
        for seed in &seeds {
            let id = self.next_query_id();
            self.solver.submit(Query {
                id,
                variant,
                seed: *seed,
            });
        }
        if let Some(search) = self.search.as_mut() {
            search.begin_batch(first_id, seeds);
        }
    }

    /// Opens the search dialog and queues the first batch of seeds.
    pub(super) fn begin_search(&mut self, variant: Variant, undo_allowed: bool) {
        self.solver.cancel();
        self.check = None;
        self.search = Some(Search::new(variant, undo_allowed));
        self.submit_batch();
        self.show_search(0);
    }

    fn show_search(&mut self, focus: usize) {
        let Some(search) = &self.search else {
            return;
        };
        let modal = Modal::Search(search.progress(focus));
        if matches!(self.modal, Some(Modal::Search(_))) {
            self.modal = Some(modal);
        } else {
            self.open_modal(modal);
        }
    }

    fn search_focus(&self) -> usize {
        match &self.modal {
            Some(Modal::Search(progress)) => progress.focus,
            _ => 0,
        }
    }

    /// Applies a button of the search dialog; the dialog is already closed.
    pub(super) fn search_step(&mut self, command: SearchCommand) {
        match command {
            SearchCommand::Cancel => self.end_search(),
            SearchCommand::More => {
                self.submit_batch();
                self.show_search(0);
            }
            SearchCommand::PlayUnproven => {
                let chosen = self
                    .search
                    .as_ref()
                    .and_then(|search| search.candidate().map(|seed| (search, seed)))
                    .map(|(search, seed)| (search.variant, seed, search.undo_allowed));
                self.end_search();
                if let Some((variant, seed, undo_allowed)) = chosen {
                    self.begin_game(
                        variant,
                        seed,
                        undo_allowed,
                        Origin::Searched(Proof::Unknown),
                    );
                }
            }
        }
    }

    fn end_search(&mut self) {
        self.solver.cancel();
        self.search = None;
    }

    /// Collects what the solver has answered; called on every input, so the 200 ms tick of the
    /// event loop also advances a search.
    pub(super) fn poll_solver(&mut self) {
        for answer in self.solver.poll() {
            self.on_answer(answer);
        }
    }

    fn on_answer(&mut self, answer: Answer) {
        if self
            .search
            .as_ref()
            .is_some_and(|search| search.owns(answer.id))
        {
            self.on_search_answer(answer);
        } else if self.check.is_some_and(|check| check.id == answer.id) {
            self.on_check_answer(answer.proof);
        }
    }

    fn on_search_answer(&mut self, answer: Answer) {
        let Some(search) = self.search.as_mut() else {
            return;
        };
        let (variant, undo_allowed) = (search.variant, search.undo_allowed);
        match search.record(answer) {
            Step::Found(seed) => {
                self.end_search();
                self.close_modal();
                self.begin_game(
                    variant,
                    seed,
                    undo_allowed,
                    Origin::Searched(Proof::Winnable),
                );
            }
            Step::Searching | Step::Capped => {
                let focus = self.search_focus();
                self.show_search(focus);
            }
        }
    }

    /// Asks the solver about a deal the player chose and shows "checking" in the header.
    pub(super) fn start_check(&mut self, variant: Variant, seed: u64) {
        let id = self.next_query_id();
        self.check = Some(Check { id, seed });
        self.board.set_deal(DealCheck::Checking);
        self.solver.submit(Query { id, variant, seed });
    }

    fn on_check_answer(&mut self, proof: Proof) {
        let Some(check) = self.check.take() else {
            return;
        };
        if self.board.session().map(|session| session.seed()) != Some(check.seed) {
            return;
        }
        self.board.set_deal(DealCheck::Known(proof));
        self.record_proof(proof);
    }

    /// Writes what is known about the running deal to its history row. The mark is advisory, so a
    /// failure is logged and never alerted.
    pub(super) fn record_proof(&mut self, proof: Proof) {
        let Some(id) = self.board.game_id() else {
            return;
        };
        if let Err(error) = self.storage.record_proof(id, proof) {
            tracing::warn!(%error, "the winnability of the deal could not be recorded");
        }
    }
}
