//! Engine of oXySol. Pure logic: no terminal, no files, no clock, no locale.

pub mod card;
pub mod deck;
pub mod game;
mod games;
pub mod moves;
pub mod pile;
pub mod rng;
pub mod rules;
pub mod session;
pub mod solver;

pub use card::{Card, CardId, Color, Rank, Suit};
pub use game::{DrawCount, Game, PyramidRules, StockStatus, SuitCount, Variant, YukonRules, deal};
pub use moves::{Move, MoveError};
pub use pile::{PileRef, PileSnapshot, TableCard};
pub use rng::SplitMix64;
pub use session::{RestoreError, Session, SessionRecord};
pub use solver::{SolveBudget, Verdict, solve};
