//! The search for a winnable deal: the state of one search and what the dialog shows. A search
//! asks the solver about up to [`SEED_CAP`] fresh seeds, stops at the first that is proven
//! winnable and, when the cap is reached, keeps the first seed that stayed unknown as a candidate.

use oxysol_core::Variant;

use crate::solver::{Answer, Proof};

/// How many seeds one search tries before it asks the player what to do.
pub const SEED_CAP: u32 = 40;

/// What a button of the search dialog asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchCommand {
    /// Try another [`SEED_CAP`] seeds.
    More,
    /// Play the best candidate, a deal whose winnability is unknown.
    PlayUnproven,
    /// Stop searching and keep what was on the screen.
    Cancel,
}

/// What the search dialog shows: progress while it runs, the choices once the cap is reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchProgress {
    pub variant: Variant,
    pub tried: u32,
    pub cap: u32,
    pub capped: bool,
    pub can_play_unproven: bool,
    pub focus: usize,
}

impl SearchProgress {
    /// The buttons of the dialog, in order.
    pub fn choices(&self) -> Vec<SearchCommand> {
        match (self.capped, self.can_play_unproven) {
            (false, _) => vec![SearchCommand::Cancel],
            (true, true) => vec![
                SearchCommand::PlayUnproven,
                SearchCommand::More,
                SearchCommand::Cancel,
            ],
            (true, false) => vec![SearchCommand::More, SearchCommand::Cancel],
        }
    }

    /// Moves the focus to the next or the previous button, wrapping.
    pub fn step_focus(&mut self, forward: bool) {
        let count = self.choices().len();
        self.focus = if forward {
            (self.focus + 1) % count
        } else {
            (self.focus + count - 1) % count
        };
    }
}

/// How an answer changed the search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Step {
    Searching,
    Found(u64),
    Capped,
}

/// One running search.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Search {
    pub variant: Variant,
    pub undo_allowed: bool,
    first_id: u64,
    seeds: Vec<u64>,
    tried: u32,
    candidate: Option<u64>,
    capped: bool,
}

impl Search {
    pub fn new(variant: Variant, undo_allowed: bool) -> Self {
        Self {
            variant,
            undo_allowed,
            first_id: 0,
            seeds: Vec::new(),
            tried: 0,
            candidate: None,
            capped: false,
        }
    }

    /// Starts a batch of seeds whose queries have the ids `first_id`, `first_id + 1`, and so on.
    pub fn begin_batch(&mut self, first_id: u64, seeds: Vec<u64>) {
        self.first_id = first_id;
        self.seeds = seeds;
        self.tried = 0;
        self.capped = false;
    }

    fn seed_of(&self, id: u64) -> Option<u64> {
        let index = usize::try_from(id.checked_sub(self.first_id)?).ok()?;
        self.seeds.get(index).copied()
    }

    /// Whether `id` belongs to the batch in progress.
    pub fn owns(&self, id: u64) -> bool {
        !self.capped && self.seed_of(id).is_some()
    }

    /// Counts an answer of the batch in progress.
    pub fn record(&mut self, answer: Answer) -> Step {
        let Some(seed) = self.seed_of(answer.id) else {
            return Step::Searching;
        };
        self.tried += 1;
        match answer.proof {
            Proof::Winnable => return Step::Found(seed),
            Proof::Unknown => {
                self.candidate.get_or_insert(seed);
            }
            Proof::Unwinnable => {}
        }
        if usize::try_from(self.tried).is_ok_and(|tried| tried >= self.seeds.len()) {
            self.capped = true;
            return Step::Capped;
        }
        Step::Searching
    }

    /// The best seed that was not proven winnable: the first one the solver could not decide.
    pub fn candidate(&self) -> Option<u64> {
        self.candidate
    }

    pub fn progress(&self, focus: usize) -> SearchProgress {
        SearchProgress {
            variant: self.variant,
            tried: self.tried,
            cap: SEED_CAP,
            capped: self.capped,
            can_play_unproven: self.candidate.is_some(),
            focus,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn search(seeds: u64) -> Search {
        let mut search = Search::new(Variant::FreeCell, true);
        search.begin_batch(10, (100..100 + seeds).collect());
        search
    }

    fn answer(id: u64, proof: Proof) -> Answer {
        Answer { id, proof }
    }

    #[test]
    fn the_first_winnable_seed_ends_the_search() {
        let mut search = search(5);
        assert_eq!(search.record(answer(10, Proof::Unknown)), Step::Searching);
        assert_eq!(search.record(answer(11, Proof::Winnable)), Step::Found(101));
        assert_eq!(search.progress(0).tried, 2);
    }

    #[test]
    fn the_cap_is_reached_after_the_last_answer_and_keeps_the_first_unknown_seed() {
        let mut search = search(3);
        assert_eq!(
            search.record(answer(10, Proof::Unwinnable)),
            Step::Searching
        );
        assert_eq!(search.record(answer(11, Proof::Unknown)), Step::Searching);
        assert_eq!(search.record(answer(12, Proof::Unknown)), Step::Capped);
        assert_eq!(search.candidate(), Some(101));
        assert!(!search.owns(12));
        assert!(search.progress(0).capped);
    }

    #[test]
    fn a_batch_of_unwinnable_seeds_leaves_no_candidate() {
        let mut search = search(2);
        search.record(answer(10, Proof::Unwinnable));
        assert_eq!(search.record(answer(11, Proof::Unwinnable)), Step::Capped);
        assert_eq!(search.candidate(), None);
        assert_eq!(
            search.progress(0).choices(),
            vec![SearchCommand::More, SearchCommand::Cancel]
        );
    }

    #[test]
    fn ids_outside_the_batch_are_not_owned() {
        let search = search(2);
        assert!(search.owns(10) && search.owns(11));
        assert!(!search.owns(9) && !search.owns(12));
    }

    #[test]
    fn the_focus_wraps_both_ways() {
        let mut progress = SearchProgress {
            variant: Variant::FreeCell,
            tried: 40,
            cap: SEED_CAP,
            capped: true,
            can_play_unproven: true,
            focus: 0,
        };
        progress.step_focus(false);
        assert_eq!(progress.focus, 2);
        progress.step_focus(true);
        assert_eq!(progress.focus, 0);
    }
}
