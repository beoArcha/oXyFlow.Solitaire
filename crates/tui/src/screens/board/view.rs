//! Drawing of the board screen: the game information (a header row, or the `Game` box of the guide
//! panel), the board with its auras (inside the field frame in the themes that draw one), the
//! framed guide panel and the two rows of the controls strip.
//! Geometry comes from [`layout`], so the mouse sees the same board that is drawn.

use oxysol_core::Session;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Paragraph};

use super::controls;
use super::guidance::Guidance;
use super::header::Facts;
use super::status::Tone;
use super::{BoardScreen, counters};
use crate::layout;
use crate::screens::ViewCtx;
use crate::theme::Token;
use crate::widgets::guide::{self, Guide};
use crate::widgets::moves::{self, MoveEntry, PanelMap};
use crate::widgets::pile::{self, Marks};
use crate::widgets::{aura, edge};

/// What the board screen shows: the game, its facts, the highlights and the last event.
pub(super) struct Scene<'a> {
    pub session: Option<&'a Session>,
    pub facts: Option<&'a Facts>,
    pub undo_allowed: bool,
    pub marks: &'a Marks,
    pub event: Option<&'a (String, Tone)>,
    pub covered: bool,
    pub guidance: Option<&'a Guidance>,
    pub chrome: layout::Chrome,
    pub entries: &'a [MoveEntry],
    pub focus: Option<usize>,
}

fn draw_board(frame: &mut Frame, body: Rect, session: &Session, marks: &Marks, ctx: &ViewCtx<'_>) {
    let piles = session.game().piles();
    let limit = ctx.settings.card_size.size();
    let geometry = layout::compute_within(session.variant(), &piles, body, limit);
    let recycle = counters::can_recycle(session);
    for (snapshot, placed) in piles.iter().zip(&geometry.piles) {
        pile::draw(frame.buffer_mut(), snapshot, placed, marks, recycle, ctx);
    }
    aura::draw(frame.buffer_mut(), body, &geometry, marks, ctx);
}

fn draw_panel(frame: &mut Frame, rect: Rect, scene: &Scene<'_>, ctx: &ViewCtx<'_>) {
    let (Some(session), Some(facts)) = (scene.session, scene.facts) else {
        return;
    };
    let lines = facts.lines(ctx.theme.style(Token::Text, Token::Surface));
    let usable = scene.guidance.map_or(&[][..], |guidance| &guidance.usable);
    let panel = Guide {
        variant: session.variant(),
        usable,
        undo_allowed: scene.undo_allowed,
        game: &lines,
    };
    guide::draw(frame.buffer_mut(), rect, &panel, ctx);
}

/// Draws the whole board screen into the parts of `scene.chrome`.
pub(super) fn render(
    frame: &mut Frame,
    scene: &Scene<'_>,
    ctx: &ViewCtx<'_>,
) -> (controls::Buttons, PanelMap) {
    let parts = scene.chrome;
    if let (Some(rect), Some(facts)) = (parts.header, scene.facts) {
        let row = facts.row(rect, ctx.theme.style(Token::Accent, Token::Background));
        frame.render_widget(Paragraph::new(row.line), row.area);
    }
    frame.render_widget(
        Block::new().style(ctx.theme.style(Token::TableText, Token::Table)),
        parts.body,
    );
    if ctx.theme.field_edge() {
        edge::draw(frame.buffer_mut(), parts.field, ctx);
    }
    if let Some(session) = scene.session.filter(|_| !scene.covered) {
        draw_board(frame, parts.body, session, scene.marks, ctx);
    }
    let buttons = controls::draw(
        frame,
        parts.now,
        parts.possible,
        scene.guidance,
        scene.event.filter(|_| !scene.covered),
        ctx,
    );
    if let Some(rect) = parts.guide {
        draw_panel(frame, rect, scene, ctx);
    }
    let panel = parts.moves.map_or_else(PanelMap::default, |rect| {
        moves::draw(frame.buffer_mut(), rect, scene.entries, scene.focus, ctx)
    });
    (buttons, panel)
}

pub(super) fn draw(
    frame: &mut Frame,
    board: &BoardScreen,
    ctx: &ViewCtx<'_>,
) -> (controls::Buttons, PanelMap) {
    let guidance = board.guidance();
    let entries = board.move_entries();
    let facts = board.facts(ctx.now);
    let event = board.event();
    let scene = Scene {
        session: board.session(),
        facts: facts.as_ref(),
        undo_allowed: board.undo_allowed,
        marks: &board.marks(),
        event: event.as_ref(),
        covered: board.covered,
        guidance: guidance.as_ref(),
        chrome: board.chrome(),
        entries: &entries,
        focus: board.panel_focus.filter(|_| board.moves_shown()),
    };
    render(frame, &scene, ctx)
}
