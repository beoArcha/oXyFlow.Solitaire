//! First-start inline banner on the main menu offering low-vision mode once, with `Yes` and
//! `Not now` buttons reachable by keyboard and mouse and marked without relying on colour.

use ratatui::Frame;
use ratatui::crossterm::event::MouseEvent;
use ratatui::layout::{Alignment, Margin, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::symbols::border;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph};

use crate::action::Action;
use crate::i18n::t;
use crate::mouse::{contains, left_press};
use crate::screens::focus::marker;
use crate::screens::{Effect, ViewCtx};
use crate::settings::TerminalKind;
use crate::theme::Token;

/// Rows needed by the framed banner (top border, prompt, buttons, bottom border).
pub(super) const HEIGHT: u16 = 4;
const MAX_WIDTH: u16 = 78;
const BUTTON_GAP: &str = "   ";

const ASCII_BORDER: border::Set<'static> = border::Set {
    top_left: "+",
    top_right: "+",
    bottom_left: "+",
    bottom_right: "+",
    vertical_left: "|",
    vertical_right: "|",
    horizontal_top: "-",
    horizontal_bottom: "-",
};

/// One of the two buttons of the low-vision offer banner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BannerButton {
    Yes,
    NotNow,
}

impl BannerButton {
    pub(super) const fn toggled(self) -> Self {
        match self {
            Self::Yes => Self::NotNow,
            Self::NotNow => Self::Yes,
        }
    }
}

/// Hit rectangles and drawing for the first-start low-vision offer banner.
#[derive(Debug, Default)]
pub(super) struct Banner {
    yes: Rect,
    not_now: Rect,
}

fn width_of(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(u16::MAX)
}

fn button_label(key: &str, focused: bool, ctx: &ViewCtx<'_>) -> String {
    format!("{} [ {} ]", marker(ctx, focused), t(key))
}

fn button_style(focused: bool, ctx: &ViewCtx<'_>) -> Style {
    if focused {
        ctx.theme.focused_button()
    } else {
        ctx.theme.style(Token::Accent, Token::Surface)
    }
}

/// Keyboard input while the banner holds focus. Returns `Some(effect)` when handled, or `None`
/// when the focus leaves back to the menu list.
pub(super) fn handle_action(
    focused: &mut Option<BannerButton>,
    action: Action,
    ctx: &ViewCtx<'_>,
) -> Option<Effect> {
    let current = (*focused)?;
    match action {
        Action::Left | Action::Right | Action::FocusNext | Action::FocusPrevious => {
            *focused = Some(current.toggled());
            Some(Effect::None)
        }
        Action::Select => {
            *focused = None;
            Some(effect(current == BannerButton::Yes, ctx))
        }
        Action::Up | Action::Down | Action::Back => {
            *focused = None;
            Some(Effect::None)
        }
        Action::Quit => Some(Effect::Quit),
        _ => Some(Effect::None),
    }
}

/// Applies the player's answer (`Yes` or `Not now`), marks the offer as answered, and saves.
pub(super) fn effect(accept: bool, ctx: &ViewCtx<'_>) -> Effect {
    let classic_console = ctx.font.terminal == TerminalKind::ClassicConsole && ctx.font.owns_window;
    let mut next = if accept {
        ctx.settings.with_low_vision(true, classic_console)
    } else {
        ctx.settings.clone()
    };
    next.low_vision_offered = true;
    Effect::Apply(next)
}

impl Banner {
    pub(super) fn clear(&mut self) {
        self.yes = Rect::default();
        self.not_now = Rect::default();
    }

    pub(super) fn hit(&self, event: &MouseEvent) -> Option<BannerButton> {
        let cell = left_press(event)?;
        if contains(self.yes, cell) {
            Some(BannerButton::Yes)
        } else if contains(self.not_now, cell) {
            Some(BannerButton::NotNow)
        } else {
            None
        }
    }

    pub(super) fn draw(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        focused: Option<BannerButton>,
        ctx: &ViewCtx<'_>,
    ) {
        if area.height < HEIGHT || area.width < 24 {
            self.clear();
            return;
        }
        let width = area.width.saturating_sub(4).min(MAX_WIDTH);
        let x = area.x + (area.width - width) / 2;
        let box_rect = Rect::new(x, area.y, width, HEIGHT);
        let border_style = if focused.is_some() {
            ctx.theme
                .style(Token::BorderFocus, Token::Surface)
                .add_modifier(Modifier::BOLD)
        } else {
            ctx.theme.style(Token::Border, Token::Surface)
        };
        frame.render_widget(Clear, box_rect);
        let block = Block::bordered()
            .border_style(border_style)
            .style(ctx.theme.surface());
        let block = if ctx.glyphs.unicode {
            block.border_type(ctx.glyphs.border)
        } else {
            block.border_set(ASCII_BORDER)
        };
        frame.render_widget(block, box_rect);
        let inner = box_rect.inner(Margin::new(1, 1));
        let prompt = Line::styled(
            t("menu.low_vision.prompt").into_owned(),
            ctx.theme.surface(),
        );
        frame.render_widget(
            Paragraph::new(prompt).alignment(Alignment::Center),
            Rect::new(inner.x, inner.y, inner.width, 1),
        );

        let yes_text = button_label(
            "menu.low_vision.yes",
            focused == Some(BannerButton::Yes),
            ctx,
        );
        let not_now_text = button_label(
            "menu.low_vision.not_now",
            focused == Some(BannerButton::NotNow),
            ctx,
        );
        let yes_w = width_of(&yes_text);
        let not_now_w = width_of(&not_now_text);
        let gap_w = width_of(BUTTON_GAP);
        let total = yes_w + gap_w + not_now_w;
        let buttons_y = inner.y + 1;
        let start_x = inner.x + inner.width.saturating_sub(total) / 2;
        self.yes = Rect::new(start_x, buttons_y, yes_w, 1).intersection(inner);
        self.not_now =
            Rect::new(start_x + yes_w + gap_w, buttons_y, not_now_w, 1).intersection(inner);

        let line = Line::from(vec![
            Span::styled(
                yes_text,
                button_style(focused == Some(BannerButton::Yes), ctx),
            ),
            Span::styled(BUTTON_GAP, ctx.theme.surface()),
            Span::styled(
                not_now_text,
                button_style(focused == Some(BannerButton::NotNow), ctx),
            ),
        ]);
        frame.render_widget(
            Paragraph::new(line).alignment(Alignment::Center),
            Rect::new(inner.x, buttons_y, inner.width, 1),
        );
    }
}
