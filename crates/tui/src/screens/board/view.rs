//! Drawing of the board screen. Stub: a header with the game, seed and time, and a notice.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use super::BoardScreen;
use crate::action::Action;
use crate::clock::format_elapsed;
use crate::i18n::{t, tf};
use crate::screens::{ViewCtx, variant_label_key};
use crate::theme::Token;
use crate::widgets::footer;

const FOOTER: [Action; 2] = [Action::Back, Action::Help];

fn header(board: &BoardScreen, ctx: &ViewCtx<'_>) -> String {
    let Some(session) = board.session() else {
        return String::new();
    };
    tf(
        "common.board.header",
        &[
            ("variant", &t(variant_label_key(session.variant()))),
            ("seed", &session.seed().to_string()),
            ("time", &format_elapsed(board.elapsed(ctx.now))),
        ],
    )
}

pub(super) fn draw(frame: &mut Frame, area: Rect, board: &BoardScreen, ctx: &ViewCtx<'_>) {
    let [header_area, body_area, footer_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);
    let style = ctx.theme.style(Token::Accent, Token::Background);
    frame.render_widget(
        Paragraph::new(Line::styled(header(board, ctx), style)).alignment(Alignment::Center),
        header_area,
    );
    let table = ctx.theme.style(Token::TableText, Token::Table);
    frame.render_widget(
        Paragraph::new(Line::raw(t("common.board.placeholder").into_owned()))
            .alignment(Alignment::Center)
            .style(table),
        body_area,
    );
    footer::draw(frame, footer_area, &FOOTER, ctx);
}
