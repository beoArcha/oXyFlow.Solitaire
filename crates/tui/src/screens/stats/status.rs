//! The status line under the buttons: the progress of a running check, else the last notice, else
//! the keys of the checks.

use ratatui::Frame;
use ratatui::layout::Rect;

use super::assist::assist_line;
use super::view::centered;
use super::{Notice, StatsScreen};
use crate::i18n::{t, tf};
use crate::screens::ViewCtx;
use crate::theme::Token;

impl StatsScreen {
    /// The assist line of the selected game, or the keys of the checks when it has no counted moves.
    fn default_line(&self) -> String {
        self.records
            .get(self.list.selected())
            .and_then(|record| assist_line(&record.counters.assist))
            .unwrap_or_else(|| t("stats.check.hint").into_owned())
    }

    /// Draws the status line.
    pub(super) fn draw_status(&self, frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
        if let Some(progress) = self.progress_text() {
            centered(
                frame,
                area,
                progress,
                ctx.theme.style(Token::Accent, Token::Background),
            );
            return;
        }
        let Some(notice) = self.notice else {
            if !self.records.is_empty() {
                let style = ctx.theme.style(Token::TextMuted, Token::Background);
                centered(frame, area, self.default_line(), style);
            }
            return;
        };
        let (text, token) = match notice {
            Notice::Copied(seed) => (
                tf("stats.copied", &[("seed", &seed.to_string())]),
                Token::Accent,
            ),
            Notice::CopyFailed(seed) => (
                tf("stats.copy_failed", &[("seed", &seed.to_string())]),
                Token::Error,
            ),
            Notice::AlreadyChecked => (t("stats.check.already").into_owned(), Token::Accent),
            Notice::NothingToCheck => (t("stats.check.nothing").into_owned(), Token::Accent),
            Notice::Finished(total) => (
                tf("stats.check.finished", &[("total", &total.to_string())]),
                Token::Accent,
            ),
        };
        centered(frame, area, text, ctx.theme.style(token, Token::Background));
    }
}
