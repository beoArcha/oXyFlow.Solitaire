//! The assist line of the statistics (P4-06): for the selected game the share of automatic moves,
//! the failed quick moves, the longest run of quick moves and the auto-finish, with a tag when
//! the game counts as assisted. It is information only.

use crate::i18n::{t, tf};
use crate::storage::AssistCounters;

const SEPARATOR: &str = " · ";

/// The line for a game with counted moves; `None` for a game from before the counters existed.
pub(super) fn assist_line(assist: &AssistCounters) -> Option<String> {
    let share = assist.share_percent()?;
    let params = [
        ("share", share.to_string()),
        ("auto", assist.quick.to_string()),
        ("total", assist.decided().to_string()),
        ("failed", assist.failed.to_string()),
        ("run", assist.longest_streak.to_string()),
        ("finish", assist.finish.to_string()),
    ];
    let fill = |key: &str| {
        let borrowed: Vec<(&str, &str)> = params
            .iter()
            .map(|(name, value)| (*name, value.as_str()))
            .collect();
        tf(key, &borrowed)
    };
    let mut parts = vec![
        fill("stats.assist.share"),
        fill("stats.assist.failed"),
        fill("stats.assist.run"),
        fill("stats.assist.finish"),
    ];
    if assist.is_assisted() {
        parts.push(t("stats.assist.tag").into_owned());
    }
    Some(parts.join(SEPARATOR))
}
