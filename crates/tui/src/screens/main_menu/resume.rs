//! The text of the Resume row: the saved game's variant, playing time and moves. A row too narrow for
//! all three drops the moves, then the time, and keeps the variant.

use ratatui::text::Line;

use crate::clock::format_elapsed;
use crate::i18n::{t, tf};
use crate::screens::{SavedSummary, variant_label_key};
use crate::widgets::menu_list::Row;

/// The room the marker and the margin of the list take from the row.
const MARGIN: u16 = 4;

/// The Resume label for `saved`, in the longest of its three forms that fits `columns`.
pub fn label(saved: SavedSummary, columns: u16) -> String {
    let game = t(variant_label_key(saved.variant)).into_owned();
    let time = format_elapsed(saved.elapsed);
    let moves = saved.moves.to_string();
    let full = tf(
        "menu.resume_full",
        &[("game", &game), ("time", &time), ("moves", &moves)],
    );
    let with_time = tf("menu.resume_time", &[("game", &game), ("time", &time)]);
    let shortest = tf("menu.resume_game", &[("game", &game)]);
    let room = usize::from(columns.saturating_sub(MARGIN));
    [full, with_time]
        .into_iter()
        .find(|text| Line::raw(text.as_str()).width() <= room)
        .unwrap_or(shortest)
}

/// The Resume row: the word `Resume` and, muted, the game details after it. The row is followed by
/// an empty row, so the other entries sit one row lower and apart from it. A text that does not
/// start with the word is drawn whole.
pub fn row(saved: SavedSummary, columns: u16) -> Row {
    let text = label(saved, columns);
    let word = t("menu.resume");
    if let Some(details) = text.strip_prefix(word.as_ref()) {
        return Row::new(word.as_ref())
            .with_detail(details)
            .with_gap_after();
    }
    Row::new(text).with_gap_after()
}
