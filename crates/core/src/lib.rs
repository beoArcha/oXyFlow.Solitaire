//! Engine of oXySol. Pure logic: no terminal, no files, no clock, no locale.

pub mod card;
pub mod deck;
pub mod moves;
pub mod pile;
pub mod rng;
pub mod rules;

pub use card::{Card, CardId, Color, Rank, Suit};
pub use moves::{Move, MoveError};
pub use pile::{PileRef, PileSnapshot, TableCard};
pub use rng::SplitMix64;
