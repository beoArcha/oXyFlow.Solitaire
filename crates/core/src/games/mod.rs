//! Rules of the four games. Internal: callers reach them only through [`crate::deal`].

pub(crate) mod freecell;
pub(crate) mod klondike;
pub(crate) mod pyramid;
pub(crate) mod spider;
mod stub;
