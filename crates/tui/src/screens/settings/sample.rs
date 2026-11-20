//! The aura sample under the settings list: a small card framed the way the board frames the
//! focused card, the carried run and a drop target, in the style, size and colour now chosen.

use ratatui::text::{Line, Span};

use crate::i18n::t;
use crate::screens::ViewCtx;
use crate::theme::Token;
use crate::widgets::aura::{Role, side_symbol};

const ROLES: [Role; 3] = [Role::Focus, Role::Carry, Role::Target];

fn sample_of(role: Role, ctx: &ViewCtx<'_>) -> Vec<Span<'static>> {
    let settings = ctx.settings;
    let token = settings.aura_colour.token().unwrap_or(role.token());
    let side = side_symbol(ctx.glyphs, settings.aura_style, role)
        .map(|symbol| symbol.repeat(usize::from(settings.aura_size.rings())))
        .unwrap_or_default();
    let card = ctx.glyphs.face_down.repeat(2);
    vec![
        Span::styled(side.clone(), ctx.theme.style(token, Token::Table)),
        Span::styled(card, ctx.theme.style(Token::CardBack, Token::Table)),
        Span::styled(side, ctx.theme.style(token, Token::Table)),
    ]
}

/// The sample line: its label, then the focus, carry and target samples.
pub(super) fn line(ctx: &ViewCtx<'_>) -> Line<'static> {
    let mut spans = vec![Span::styled(
        format!("{} ", t("settings.sample.aura")),
        ctx.theme.style(Token::TextMuted, Token::Background),
    )];
    for role in ROLES {
        spans.extend(sample_of(role, ctx));
        spans.push(Span::styled("  ", ctx.theme.base()));
    }
    Line::from(spans)
}
