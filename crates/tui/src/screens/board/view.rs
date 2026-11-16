//! Drawing of the board screen: header (game, seed, time, moves), the board with its auras and the
//! stock counter, the guide panel, the status line, the two rows of the action line and the key
//! hints. Geometry comes from
//! [`layout`], so the mouse sees the same board that is drawn.

use oxysol_core::Session;
use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

use super::guidance::Guidance;
use super::header::Header;
use super::{BoardScreen, counters};
use crate::action::Action;
use crate::layout;
use crate::screens::ViewCtx;
use crate::theme::Token;
use crate::widgets::guide::{self, Guide};
use crate::widgets::pile::{self, Marks};
use crate::widgets::{aura, footer};

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

fn footer_actions(undo_allowed: bool) -> Vec<Action> {
    FOOTER
        .into_iter()
        .filter(|action| undo_allowed || !matches!(action, Action::Undo | Action::Redo))
        .collect()
}

/// What the board screen shows: the game, its header, the highlights and the status message.
pub(super) struct Scene<'a> {
    pub session: Option<&'a Session>,
    pub header: Option<&'a Header>,
    pub undo_allowed: bool,
    pub marks: &'a Marks,
    pub status: &'a str,
    pub covered: bool,
    pub guidance: Option<&'a Guidance>,
    pub guide: bool,
}

fn draw_board(
    frame: &mut Frame,
    body: Rect,
    session: &Session,
    marks: &Marks,
    counter: Option<&str>,
    ctx: &ViewCtx<'_>,
) -> bool {
    let piles = session.game().piles();
    let geometry = layout::compute(session.variant(), &piles, body);
    for (snapshot, placed) in piles.iter().zip(&geometry.piles) {
        pile::draw(frame.buffer_mut(), snapshot, placed, marks, ctx);
    }
    aura::draw(frame.buffer_mut(), body, &geometry, marks, ctx);
    let Some((text, row)) = counter
        .and_then(|text| counters::beside_stock(&geometry, body, text).map(|row| (text, row)))
    else {
        return false;
    };
    let style = ctx.theme.style(Token::TableText, Token::Table);
    frame.render_widget(Paragraph::new(Line::styled(text.to_owned(), style)), row);
    true
}

fn action_row(frame: &mut Frame, area: Rect, text: String, ctx: &ViewCtx<'_>) {
    let style = ctx.theme.style(Token::Text, Token::Background);
    frame.render_widget(Paragraph::new(Line::styled(text, style)), area);
}

fn centered_line(text: String, style: ratatui::style::Style) -> Paragraph<'static> {
    Paragraph::new(Line::styled(text, style)).alignment(Alignment::Center)
}

/// Draws the whole board screen into `area`.
pub(super) fn render(frame: &mut Frame, area: Rect, scene: &Scene<'_>, ctx: &ViewCtx<'_>) {
    let parts = layout::chrome_with(area, scene.guide);
    let background = ctx.theme.style(Token::Text, Token::Background);
    let accent = ctx.theme.style(Token::Accent, Token::Background);
    if let Some(header) = scene.header {
        frame.render_widget(Paragraph::new(header.line(accent)), parts.header);
    }
    frame.render_widget(
        Block::new().style(ctx.theme.style(Token::TableText, Token::Table)),
        parts.body,
    );
    let counter = scene.session.and_then(counters::stock_text);
    let mut beside = false;
    if let Some(session) = scene.session.filter(|_| !scene.covered) {
        beside = draw_board(
            frame,
            parts.body,
            session,
            scene.marks,
            counter.as_deref(),
            ctx,
        );
    }
    frame.render_widget(
        centered_line(scene.status.to_owned(), background),
        parts.status,
    );
    let width = usize::from(area.width);
    if let Some(guidance) = scene.guidance {
        let shown = counter.as_deref().filter(|_| !beside);
        action_row(frame, parts.now, guidance.now_line(width, shown), ctx);
        action_row(frame, parts.possible, guidance.possible_line(width), ctx);
    }
    if let (Some(rect), Some(session)) = (parts.guide, scene.session) {
        let usable = scene.guidance.map_or(&[][..], |guidance| &guidance.usable);
        let panel = Guide {
            variant: session.variant(),
            usable,
            undo_allowed: scene.undo_allowed,
            stock: counter.as_deref(),
        };
        guide::draw(frame.buffer_mut(), rect, &panel, ctx);
    }
    footer::draw(
        frame,
        parts.footer,
        &footer_actions(scene.undo_allowed),
        ctx,
    );
}

pub(super) fn draw(frame: &mut Frame, area: Rect, board: &BoardScreen, ctx: &ViewCtx<'_>) {
    let guidance = board.guidance();
    let header = board.header(ctx.now);
    let scene = Scene {
        session: board.session(),
        header: header.as_ref(),
        undo_allowed: board.undo_allowed,
        marks: &board.marks(),
        status: &board.status_text(),
        covered: board.covered,
        guidance: guidance.as_ref(),
        guide: ctx.settings.guide_panel,
    };
    render(frame, area, &scene, ctx);
}
