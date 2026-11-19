//! The dialog of the search for a winnable deal: progress while the solver works, then the choices
//! once the seed cap is reached.

use super::dialog::{Button, Text};
use crate::app::{SearchCommand, SearchProgress};
use crate::i18n::{t, tf};
use crate::screens::variant_label_key;
use crate::solver::slow_to_prove;

fn label_key(command: SearchCommand) -> &'static str {
    match command {
        SearchCommand::More => "common.search.more",
        SearchCommand::PlayUnproven => "common.search.play_unproven",
        SearchCommand::Cancel => "common.search.cancel",
    }
}

fn lines(progress: &SearchProgress) -> Vec<String> {
    let cap = progress.cap.to_string();
    let variant = t(variant_label_key(progress.variant));
    if progress.capped {
        let found = tf("common.search.capped", &[("cap", &cap)]);
        let offer = if progress.can_play_unproven {
            t("common.search.capped_offer")
        } else {
            t("common.search.capped_offer_none")
        };
        return vec![found, offer.into_owned()];
    }
    let mut lines = vec![
        tf("common.search.running", &[("variant", &variant)]),
        tf(
            "common.search.progress",
            &[("tried", &progress.tried.to_string()), ("cap", &cap)],
        ),
    ];
    if slow_to_prove(progress.variant) {
        lines.push(t("common.search.slow").into_owned());
    }
    lines.push(t("common.search.escape").into_owned());
    lines
}

pub(super) fn text(progress: &SearchProgress) -> Text {
    let buttons = progress
        .choices()
        .into_iter()
        .enumerate()
        .map(|(index, command)| Button {
            label: t(label_key(command)).into_owned(),
            focused: index == progress.focus,
        })
        .collect();
    Text {
        title: t("common.search.title").into_owned(),
        lines: lines(progress),
        buttons,
    }
}

/// The command of button `index`; a stale index presses the last button, which always cancels.
pub(super) fn command_at(progress: &SearchProgress, index: usize) -> SearchCommand {
    let choices = progress.choices();
    choices
        .get(index)
        .or(choices.last())
        .copied()
        .unwrap_or(SearchCommand::Cancel)
}
