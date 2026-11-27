//! Drawing helpers for the settings screen: title, row list, description, notes and footer.

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};

use super::FOOTER;
use super::arrows::{Arrows, glyphs, locate};
use super::options::Field;
use super::values::{is_unavailable, line_with_custom, unavailable_reason_key};
use crate::action::Action;
use crate::i18n::{t, tf};
use crate::input::primary_key;
use crate::screens::ViewCtx;
use crate::screens::focus::marker;
use crate::theme::table::{felt, felt_for, hex};
use crate::theme::{ThemeId, Token};
use crate::widgets::footer::{self, Segment};
use crate::widgets::scrollbar::{self, Metrics};

/// Rows of the screen outside the list band: title, gaps, tab bar, hidden-pane note, two lines of
/// description, reason line and footer.
pub(super) const CHROME_ROWS: u16 = 10;

pub(super) struct DrawnList {
    pub rows: Vec<(usize, Rect)>,
    pub arrows: Option<Arrows>,
    pub track: Rect,
}

pub(super) fn is_unavailable_field(field: Field, ctx: &ViewCtx<'_>) -> bool {
    is_unavailable(field, ctx.audio_status.is_ready(), ctx.font)
}

fn text_width(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(u16::MAX)
}

fn row_text(field: Field, focused: bool, columns: u16, ctx: &ViewCtx<'_>) -> String {
    let text = line_with_custom(
        field,
        ctx.settings,
        ctx.audio_status.is_ready(),
        ctx.sound_styles,
        (columns, ctx.font),
        focused.then(|| glyphs(ctx)),
        ctx.theme.custom_palette(),
    );
    format!("{} {text}", marker(ctx, focused))
}

/// The columns of the widest row of `fields`, never more than `columns`.
pub(super) fn list_width(fields: &[Field], columns: u16, ctx: &ViewCtx<'_>) -> u16 {
    fields
        .iter()
        .map(|field| text_width(&row_text(*field, true, columns, ctx)))
        .max()
        .unwrap_or_default()
        .min(columns)
}

pub(super) fn draw_title(
    frame: &mut Frame,
    area: Rect,
    scrolled: bool,
    focus: usize,
    total: usize,
    ctx: &ViewCtx<'_>,
) {
    let mut heading = t("common.settings.title").into_owned();
    if scrolled {
        let place = tf(
            "settings.preview.position",
            &[
                ("row", &(focus + 1).to_string()),
                ("total", &total.to_string()),
            ],
        );
        heading = format!("{heading}  ({place})");
    }
    let title = Line::styled(heading, ctx.theme.style(Token::Accent, Token::Background));
    frame.render_widget(Paragraph::new(title).alignment(Alignment::Center), area);
}

pub(super) fn draw_list(
    frame: &mut Frame,
    area: Rect,
    fields: &[Field],
    focus: usize,
    first_shown: usize,
    columns: u16,
    ctx: &ViewCtx<'_>,
) -> DrawnList {
    let metrics = Metrics {
        total: fields.len(),
        view: usize::from(area.height),
        offset: first_shown,
    };
    let scrolls = metrics.scrolls();
    let row_width = if scrolls {
        area.width.saturating_sub(1)
    } else {
        area.width
    };
    let track = if scrolls && area.width > 0 && area.height > 0 {
        Rect::new(area.right().saturating_sub(1), area.y, 1, area.height)
    } else {
        Rect::default()
    };
    let mut drawn = DrawnList {
        rows: Vec::new(),
        arrows: None,
        track,
    };
    let visible = fields.get(first_shown..).unwrap_or_default();
    for (offset, field) in visible.iter().copied().enumerate() {
        let Ok(dy) = u16::try_from(offset) else {
            break;
        };
        if dy >= area.height {
            break;
        }
        let index = first_shown + offset;
        let focused = index == focus;
        let text = row_text(field, focused, columns, ctx);
        let cell = Rect::new(area.x, area.y + dy, row_width, 1);
        let style = if focused {
            ctx.theme.focus()
        } else if is_unavailable_field(field, ctx) {
            ctx.theme.style(Token::TextMuted, Token::Background)
        } else {
            ctx.theme.base()
        };
        if focused {
            drawn.arrows = locate(index, &text, cell, glyphs(ctx));
        }
        drawn.rows.push((index, cell));
        frame.render_widget(Paragraph::new(Line::styled(text, style)), cell);
    }
    scrollbar::draw(frame, track, metrics, ctx);
    drawn
}

pub(super) fn draw_description(
    frame: &mut Frame,
    area: Rect,
    focus: Option<Field>,
    ctx: &ViewCtx<'_>,
) {
    let text = focus.map_or_else(
        || t("settings.desc.language").into_owned(),
        |field| field.describe(ctx.font),
    );
    let style = ctx.theme.style(Token::TextMuted, Token::Background);
    frame.render_widget(
        Paragraph::new(Line::styled(text, style))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        area,
    );
}

pub(super) fn draw_reason(
    frame: &mut Frame,
    area: Rect,
    previewing: bool,
    focus: Option<Field>,
    ctx: &ViewCtx<'_>,
) {
    let text = if previewing {
        t("settings.preview.note").into_owned()
    } else if let Some(note) = table_note(focus, ctx) {
        note
    } else if let Some(key) = unavailable_reason_key(ctx.audio_status) {
        t(key).into_owned()
    } else {
        return;
    };
    let style = ctx.theme.style(Token::TextMuted, Token::Background);
    let line = Line::styled(text, style);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn table_note(focus: Option<Field>, ctx: &ViewCtx<'_>) -> Option<String> {
    if focus != Some(Field::TableColour) {
        return None;
    }
    let choice = match (ctx.settings.theme, ctx.theme.custom_palette()) {
        (ThemeId::Custom, Some(palette)) => {
            felt(ThemeId::Custom, palette, ctx.settings.table_colour)
        }
        _ => felt_for(ctx.settings.theme, ctx.settings.table_colour),
    };
    choice.shifted.then(|| {
        tf(
            "settings.table.reason",
            &[("shade", &hex(choice.swatch.rgb))],
        )
    })
}

pub(super) fn footer_segments(ctx: &ViewCtx<'_>) -> Vec<Segment> {
    let (left, right) = glyphs(ctx);
    let arrows = tf(
        "settings.footer.arrows",
        &[("arrows", &format!("{left} {right}"))],
    );
    let prev = primary_key(Action::FocusPrevious).unwrap_or_default();
    let next = primary_key(Action::FocusNext).unwrap_or_default();
    let tabs = tf("settings.footer.tabs", &[("prev", &prev), ("next", &next)]);
    let mut segments = vec![
        Segment::plain(arrows),
        Segment::plain("   "),
        Segment::plain(tabs),
        Segment::plain("   "),
    ];
    segments.extend(footer::key_segments(&FOOTER));
    segments
}
