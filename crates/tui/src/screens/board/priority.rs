//! The drop order of the `Do:` row: when the row is too narrow, the part with the highest number
//! goes first. The numbers are distinct, so the order never depends on where a part stands in the
//! row. Pause and Moves are the first hints with a key of their own to go, right after the parts
//! that only explain (rules, arrow keys, key list, guide) and redo; undo and hint outlast them,
//! and restart, options and the way out outlast those. The `Now:` row never carries buttons: a hint
//! that does not fit is dropped, not moved there.

pub(super) const PRIMARY: u8 = 0;
pub(super) const MENU: u8 = 1;
pub(super) const PUT_BACK: u8 = 2;
pub(super) const FINISH: u8 = 3;
pub(super) const QUICK: u8 = 4;
pub(super) const STOCK: u8 = 5;
pub(super) const RESTART: u8 = 6;
pub(super) const OPTIONS: u8 = 7;
pub(super) const RUN: u8 = 8;
pub(super) const CHOOSE_TARGET: u8 = 9;
pub(super) const QUICK_CARRIED: u8 = 10;
pub(super) const HINT: u8 = 11;
pub(super) const UNDO: u8 = 12;
pub(super) const PAUSE: u8 = 13;
pub(super) const MOVES: u8 = 14;
pub(super) const REDO: u8 = 15;
pub(super) const GUIDE: u8 = 16;
pub(super) const HELP: u8 = 17;
pub(super) const MOVE_KEYS: u8 = 18;
pub(super) const RULES: u8 = 19;
