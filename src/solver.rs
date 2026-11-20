//! The worker thread behind [`oxysol_tui::Solver`]: deals are solved off the UI thread, one at a
//! time, in the order they were queued. Cancelling bumps a generation counter, so queued jobs and
//! the answer of the job in flight are dropped without any shared lock.

use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

use oxysol_core::{SolveBudget, Verdict, solve};
use oxysol_tui::{Answer, Proof, Query, Solver};
use thiserror::Error;

use crate::priority::{Native, SystemPriority, lower_solver_thread};

/// The worker thread could not be started.
#[derive(Debug, Error)]
#[error("the solver thread could not be started")]
pub struct SolverError(#[from] io::Error);

struct Job {
    query: Query,
    generation: u64,
}

fn proof_of(verdict: &Verdict) -> Proof {
    match verdict {
        Verdict::Winnable { .. } => Proof::Winnable,
        Verdict::Unwinnable => Proof::Unwinnable,
        Verdict::Unknown => Proof::Unknown,
    }
}

fn work(jobs: &Receiver<Job>, answers: &Sender<Answer>, generation: &AtomicU64) {
    while let Ok(job) = jobs.recv() {
        if job.generation != generation.load(Ordering::Acquire) {
            continue;
        }
        let Query { id, variant, seed } = job.query;
        let verdict = solve(variant, seed, SolveBudget::default_for(variant));
        if job.generation != generation.load(Ordering::Acquire) {
            continue;
        }
        let answer = Answer {
            id,
            proof: proof_of(&verdict),
        };
        if answers.send(answer).is_err() {
            return;
        }
    }
}

/// A [`Solver`] that runs [`oxysol_core::solve`] with the default budget of each game on its own
/// thread. When the thread is gone, queries are answered [`Proof::Unknown`] so that no search
/// waits for ever.
pub struct ThreadSolver {
    jobs: Sender<Job>,
    answers: Receiver<Answer>,
    generation: Arc<AtomicU64>,
    orphaned: Vec<Answer>,
}

impl ThreadSolver {
    /// Starts the worker thread below the UI thread.
    pub fn spawn() -> Result<Self, SolverError> {
        Self::spawn_with(Arc::new(SystemPriority))
    }

    /// Starts the worker thread; it lowers its own priority through `native` before it works.
    pub fn spawn_with(native: Arc<dyn Native>) -> Result<Self, SolverError> {
        let (job_sender, job_receiver) = channel();
        let (answer_sender, answer_receiver) = channel();
        let generation = Arc::new(AtomicU64::new(0));
        let worker_generation = Arc::clone(&generation);
        thread::Builder::new()
            .name("oxysol-solver".to_owned())
            .spawn(move || {
                lower_solver_thread(native.as_ref());
                work(&job_receiver, &answer_sender, &worker_generation);
            })?;
        Ok(Self {
            jobs: job_sender,
            answers: answer_receiver,
            generation,
            orphaned: Vec::new(),
        })
    }

    #[cfg(test)]
    fn without_worker() -> Self {
        let (jobs, _) = channel();
        let (_, answers) = channel();
        Self {
            jobs,
            answers,
            generation: Arc::new(AtomicU64::new(0)),
            orphaned: Vec::new(),
        }
    }
}

impl Solver for ThreadSolver {
    fn submit(&mut self, query: Query) {
        let generation = self.generation.load(Ordering::Acquire);
        if self.jobs.send(Job { query, generation }).is_err() {
            tracing::warn!("the solver thread has stopped; answering unknown");
            self.orphaned.push(Answer {
                id: query.id,
                proof: Proof::Unknown,
            });
        }
    }

    fn cancel(&mut self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
        self.orphaned.clear();
        while self.answers.try_recv().is_ok() {}
    }

    fn poll(&mut self) -> Vec<Answer> {
        let mut answers = std::mem::take(&mut self.orphaned);
        while let Ok(answer) = self.answers.try_recv() {
            answers.push(answer);
        }
        answers
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use oxysol_core::{DrawCount, Variant};

    use super::*;
    use crate::priority::Flavour;
    use crate::priority::fake::{Answer as FakeAnswer, FakeNative};

    const WAIT: Duration = Duration::from_secs(120);

    fn query(id: u64, variant: Variant, seed: u64) -> Query {
        Query { id, variant, seed }
    }

    fn collect(solver: &mut ThreadSolver, count: usize) -> Vec<Answer> {
        let started = Instant::now();
        let mut answers = Vec::new();
        while answers.len() < count && started.elapsed() < WAIT {
            answers.extend(solver.poll());
            thread::sleep(Duration::from_millis(5));
        }
        answers
    }

    #[test]
    fn deals_are_answered_in_order_and_some_freecell_deal_is_proven_winnable() {
        let Ok(mut solver) = ThreadSolver::spawn() else {
            panic!("the solver thread must start");
        };
        for seed in 0..4 {
            solver.submit(query(seed + 1, Variant::FreeCell, seed));
        }
        let answers = collect(&mut solver, 4);
        let ids: Vec<u64> = answers.iter().map(|answer| answer.id).collect();
        assert_eq!(ids, vec![1, 2, 3, 4]);
        assert!(answers.iter().any(|answer| answer.proof == Proof::Winnable));
    }

    #[test]
    fn cancelling_drops_queued_jobs_and_keeps_the_worker_usable() {
        let Ok(mut solver) = ThreadSolver::spawn() else {
            panic!("the solver thread must start");
        };
        for seed in 0..30 {
            solver.submit(query(seed, Variant::Klondike(DrawCount::Three), seed));
        }
        solver.cancel();
        solver.submit(query(99, Variant::FreeCell, 0));
        let answers = collect(&mut solver, 1);
        assert!(!answers.is_empty());
        assert!(answers.iter().all(|answer| answer.id == 99), "{answers:?}");
    }

    #[test]
    fn the_worker_lowers_its_own_priority_before_it_answers() {
        let fake = Arc::new(FakeNative::new(Flavour::Windows, FakeAnswer::Accept));
        let Ok(mut solver) = ThreadSolver::spawn_with(fake.clone()) else {
            panic!("the solver thread must start");
        };
        solver.submit(query(1, Variant::FreeCell, 0));
        assert_eq!(collect(&mut solver, 1).len(), 1);
        assert_eq!(fake.thread_calls(), 1);
        assert!(fake.process_calls().is_empty());
    }

    #[test]
    fn a_stopped_worker_answers_unknown_instead_of_hanging_the_search() {
        let mut solver = ThreadSolver::without_worker();
        solver.submit(query(7, Variant::FreeCell, 1));
        assert_eq!(
            solver.poll(),
            vec![Answer {
                id: 7,
                proof: Proof::Unknown
            }]
        );
    }
}
