//! Drawing of the board screen: header (game, seed, time), the board, a status line and the key
//! hints. Geometry comes from [`layout`], so the mouse sees the same board that is drawn.

use std::time::Duration;

use oxysol_core::Session;
use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

use super::BoardScreen;
use crate::action::Action;
use crate::clock::format_elapsed;
use crate::i18n::{t, tf};
use crate::layout;
use crate::screens::{ViewCtx, variant_label_key};
use crate::theme::Token;
use crate::widgets::footer;
use crate::widgets::pile::{self, Marks};

const FOOTER: [Action; 8] = [
    Action::Select,
    Action::Undo,
    Action::Redo,
    Action::Hint,
    Action::Stock,
    Action::Pause,
    Action::Help,
    Action::Back,
];

/// What the board screen shows: the game, its clock, the highlights and the status message.
pub(super) struct Scene<'a> {
    pub session: Option<&'a Session>,
    pub elapsed: Duration,
    pub marks: &'a Marks,
    pub status: &'a str,
}

fn header(session: &Session, elapsed: Duration) -> String {
    tf(
        "board.header.format",
        &[
            ("variant", &t(variant_label_key(session.variant()))),
            ("seed", &session.seed().to_string()),
            ("time", &format_elapsed(elapsed)),
        ],
    )
}

fn draw_board(frame: &mut Frame, body: Rect, session: &Session, marks: &Marks, ctx: &ViewCtx<'_>) {
    let piles = session.game().piles();
    let geometry = layout::compute(session.variant(), &piles, body);
    for (snapshot, placed) in piles.iter().zip(&geometry.piles) {
        pile::draw(frame.buffer_mut(), snapshot, placed, marks, ctx);
    }
}

fn centered_line(text: String, style: ratatui::style::Style) -> Paragraph<'static> {
    Paragraph::new(Line::styled(text, style)).alignment(Alignment::Center)
}

/// Draws the whole board screen into `area`.
pub(super) fn render(frame: &mut Frame, area: Rect, scene: &Scene<'_>, ctx: &ViewCtx<'_>) {
    let parts = layout::chrome(area);
    let background = ctx.theme.style(Token::Text, Token::Background);
    let accent = ctx.theme.style(Token::Accent, Token::Background);
    let title = scene
        .session
        .map(|session| header(session, scene.elapsed))
        .unwrap_or_default();
    frame.render_widget(centered_line(title, accent), parts.header);
    frame.render_widget(
        Block::new().style(ctx.theme.style(Token::TableText, Token::Table)),
        parts.body,
    );
    if let Some(session) = scene.session {
        draw_board(frame, parts.body, session, scene.marks, ctx);
    }
    frame.render_widget(
        centered_line(scene.status.to_owned(), background),
        parts.status,
    );
    footer::draw(frame, parts.footer, &FOOTER, ctx);
}

pub(super) fn draw(frame: &mut Frame, area: Rect, board: &BoardScreen, ctx: &ViewCtx<'_>) {
    let scene = Scene {
        session: board.session(),
        elapsed: board.elapsed(ctx.now),
        marks: &Marks::default(),
        status: "",
    };
    render(frame, area, &scene, ctx);
}
