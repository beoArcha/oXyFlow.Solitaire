//! The guide as a window: the same content the side panel shows, laid out for the window, and the
//! key that opens it on a screen too narrow for the panel.

use ratatui::text::Line;

use super::BoardScreen;
use crate::app::Modal;
use crate::layout::Class;
use crate::screens::{Effect, ViewCtx};
use crate::theme::Token;
use crate::widgets::guide::{self, Guide};
use crate::widgets::overlay::{GuideWindow, TEXT_WIDTH};

impl BoardScreen {
    /// Every line of the guide, laid out in `width` columns; empty without a game.
    pub fn guide_lines(&self, width: usize, ctx: &ViewCtx<'_>) -> Vec<Line<'static>> {
        let (Some(session), Some(facts)) = (self.session(), self.facts(ctx.now)) else {
            return Vec::new();
        };
        let game = facts.lines(ctx.theme.style(Token::Text, Token::Surface));
        let guidance = self.guidance();
        let guide = Guide {
            variant: session.variant(),
            usable: guidance.as_ref().map_or(&[][..], |g| &g.usable),
            undo_allowed: self.undo_allowed,
            game: &game,
        };
        guide::window_lines(&guide, width, ctx)
    }

    /// Whether the screen is too narrow for the side panel, so the guide opens as a window.
    pub(super) fn panel_unavailable(&self) -> bool {
        Class::of(self.screen) == Class::Compact
    }

    pub(super) fn open_guide_window(&self, ctx: &ViewCtx<'_>) -> Effect {
        let total = self.guide_lines(TEXT_WIDTH, ctx).len();
        Effect::Open(Modal::Guide(GuideWindow::new(total)))
    }
}
