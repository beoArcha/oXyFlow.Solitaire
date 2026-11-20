//! The winnability checks started from the statistics screen: `w` checks the selected game, `W`
//! every game without a proof (both are buttons too). The screen keeps the checks in flight and
//! counts their answers; `App` submits the queries, stores each proof on its history row and
//! reports the answers back here.

use oxysol_core::Variant;

use super::{Notice, StatsScreen};
use crate::screens::Effect;
use crate::solver::Answer;
use crate::storage::{GameId, GameRecord};

/// A game the statistics screen asks the solver about: its history row and its deal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Target {
    pub game: GameId,
    pub variant: Variant,
    pub seed: u64,
}

impl Target {
    fn of(record: &GameRecord) -> Self {
        Self {
            game: record.id,
            variant: record.variant,
            seed: record.seed,
        }
    }
}

/// The checks in flight: one job per query, as `(query id, game)`. Query ids grow in submission
/// order, so the jobs stay sorted by id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    jobs: Vec<(u64, GameId)>,
    answered: usize,
}

impl Run {
    fn total(&self) -> usize {
        self.jobs.len()
    }

    fn finished(&self) -> bool {
        self.answered >= self.jobs.len()
    }

    fn answer(&mut self, id: u64) -> Option<GameId> {
        let index = self.jobs.binary_search_by_key(&id, |(job, _)| *job).ok()?;
        self.answered += 1;
        Some(self.jobs[index].1)
    }
}

impl StatsScreen {
    /// The effect of `w` (`all` false) or `W` (`all` true): the games to check, which are the
    /// selected game or every game, without the ones already checked. Nothing starts while a check
    /// runs.
    pub(super) fn check(&mut self, all: bool) -> Effect {
        if self.run.is_some() || self.records.is_empty() {
            return Effect::None;
        }
        let picked: Vec<&GameRecord> = if all {
            self.records.iter().collect()
        } else {
            self.selected().into_iter().collect()
        };
        let targets: Vec<Target> = picked
            .into_iter()
            .filter(|record| record.proof.is_none())
            .map(Target::of)
            .collect();
        if targets.is_empty() {
            self.notice = Some(if all {
                Notice::NothingToCheck
            } else {
                Notice::AlreadyChecked
            });
            return Effect::None;
        }
        self.notice = None;
        Effect::CheckWinnable(targets)
    }

    /// Starts the checks whose queries `App` submitted; `jobs` pairs each query id with its game.
    pub fn begin_run(&mut self, jobs: Vec<(u64, GameId)>) {
        self.notice = None;
        self.run = Some(Run { jobs, answered: 0 });
    }

    /// Counts an answer of the running checks and sets the proof of its game in the list. Returns
    /// the game, so that `App` can store the proof; `None` for every other answer.
    pub fn record_answer(&mut self, answer: Answer) -> Option<GameId> {
        let run = self.run.as_mut()?;
        let game = run.answer(answer.id)?;
        let (total, finished) = (run.total(), run.finished());
        if let Some(record) = self.records.iter_mut().find(|record| record.id == game) {
            record.proof = Some(answer.proof);
        }
        if finished {
            self.run = None;
            self.notice = Some(Notice::Finished(total));
        }
        Some(game)
    }

    /// Stops the running checks; returns whether there were any, so that `App` can drop the
    /// queries still queued in the solver.
    pub fn end_run(&mut self) -> bool {
        self.run.take().is_some()
    }

    /// The progress line shown while the checks run.
    pub(super) fn progress_text(&self) -> Option<String> {
        let run = self.run.as_ref()?;
        Some(crate::i18n::tf(
            "stats.check.running",
            &[
                ("done", &run.answered.to_string()),
                ("total", &run.total().to_string()),
            ],
        ))
    }
}
