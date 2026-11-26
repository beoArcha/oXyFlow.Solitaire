//! Drawing of the docked help window and the text of its three tabs. The text is word-wrapped to
//! the window's width, with the same content as the centred window: the keys in two columns when
//! they fit (wrapping long combinations), the rules and the command line wrapped. It scrolls line
//! by line, the scroll bar counts wrapped lines.

use oxysol_core::GameOptions;
use ratatui::Frame;
use ratatui::layout::{Margin, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph};

use super::super::command_line;
use super::super::dialog;
use super::super::help;
use super::super::how_to_play::content::{self, Kind, Row};
use super::super::how_to_play::{self, Rules};
use super::super::nav;
use super::{
    DockState, button_label, button_rect, close_label, close_rect, parts, span, text_width,
};
use crate::action::Action;
use crate::app::Modal;
use crate::i18n::{t, tf};
use crate::input::primary_key;
use crate::layout::InterfaceSize;
use crate::screens::ViewCtx;
use crate::settings::DockSide;
use crate::theme::Token;
use crate::widgets::scrollbar::{self, Metrics};
use crate::widgets::text::wrap;

const DETAIL_INDENT: &str = "  ";
const KEY_COLUMN: usize = 12;
const KEY_GAP: usize = 2;

/// The wrapped lines of the active tab of `modal` for a text area `width` columns wide.
pub fn body(modal: &Modal, width: u16) -> Vec<Row> {
    let width = usize::from(width).max(1);
    match modal {
        Modal::HowToPlay(state) => rules_body(state.rules, state.options, width),
        Modal::CommandLine(_) => command_body(width),
        _ => keys_body(width),
    }
}

fn push(rows: &mut Vec<Row>, text: &str, width: usize, lead: &str, kind: Kind) {
    let lead_width = lead.chars().count();
    let lines = wrap(text, width.saturating_sub(lead_width).max(1));
    if lines.is_empty() {
        rows.push(Row {
            lead: lead.to_owned(),
            text: String::new(),
            kind,
        });
        return;
    }
    for (index, line) in lines.into_iter().enumerate() {
        let lead = if index == 0 {
            lead.to_owned()
        } else {
            " ".repeat(lead_width)
        };
        rows.push(Row {
            lead,
            text: line,
            kind,
        });
    }
}

fn push_key_only(rows: &mut Vec<Row>, keys: &str, width: usize) {
    for line in wrap(keys, width.max(1)) {
        rows.push(Row {
            lead: line,
            text: String::new(),
            kind: Kind::Key,
        });
    }
}

fn push_key_row(rows: &mut Vec<Row>, keys: &str, text: &str, width: usize) {
    let desc_width = width.saturating_sub(KEY_COLUMN);
    let fits_beside = keys.chars().count() + KEY_GAP <= KEY_COLUMN
        && desc_width > 0
        && text
            .split_whitespace()
            .all(|word| word.chars().count() <= desc_width);
    if !fits_beside {
        push_key_only(rows, keys, width);
        push(rows, text, width, DETAIL_INDENT, Kind::Body);
        return;
    }
    let lines = wrap(text, desc_width);
    if lines.is_empty() {
        rows.push(Row {
            lead: format!("{keys:<KEY_COLUMN$}"),
            text: String::new(),
            kind: Kind::Key,
        });
        return;
    }
    for (index, line) in lines.into_iter().enumerate() {
        let (lead, kind) = if index == 0 {
            (format!("{keys:<KEY_COLUMN$}"), Kind::Key)
        } else {
            (" ".repeat(KEY_COLUMN), Kind::Body)
        };
        rows.push(Row {
            lead,
            text: line,
            kind,
        });
    }
}

fn keys_body(width: usize) -> Vec<Row> {
    let mut rows = Vec::new();
    for (index, section) in help::sections().iter().enumerate() {
        if index > 0 {
            push(&mut rows, "", width, "", Kind::Body);
        }
        push(&mut rows, &section.title, width, "", Kind::Heading);
        for row in &section.rows {
            push_key_row(&mut rows, &row.keys, &row.text, width);
        }
    }
    rows
}

fn rules_body(rules: Rules, options: GameOptions, width: usize) -> Vec<Row> {
    let mut rows = Vec::new();
    let mut detail = false;
    for row in content::rows_with_options(rules, options) {
        let blank_lead = row.lead.trim().is_empty();
        match row.kind {
            Kind::Key if !blank_lead => {
                detail = true;
                push_key_only(&mut rows, row.lead.trim_end(), width);
                push(&mut rows, &row.text, width, DETAIL_INDENT, Kind::Body);
            }
            Kind::Body if detail && blank_lead => {
                push(&mut rows, &row.text, width, DETAIL_INDENT, Kind::Body);
            }
            _ => {
                detail = false;
                push(&mut rows, &row.text, width, &row.lead, row.kind);
            }
        }
    }
    rows
}

fn command_body(width: usize) -> Vec<Row> {
    let mut rows = Vec::new();
    for line in command_line::lines() {
        let trimmed = line.trim_start();
        let indented = trimmed.len() < line.len();
        let lead = if indented { DETAIL_INDENT } else { "" };
        let kind = if command_line::is_heading(&line) {
            Kind::Heading
        } else {
            Kind::Body
        };
        push(&mut rows, trimmed, width, lead, kind);
    }
    rows
}

fn line_of(row: &Row, ctx: &ViewCtx<'_>) -> Line<'static> {
    let plain = ctx.theme.surface();
    match row.kind {
        Kind::Heading => Line::styled(
            format!("{}{}", row.lead, row.text),
            plain.add_modifier(Modifier::BOLD),
        ),
        Kind::Key => Line::from(vec![
            Span::styled(
                row.lead.clone(),
                ctx.theme.style(Token::Accent, Token::Surface),
            ),
            Span::styled(row.text.clone(), plain),
        ]),
        Kind::Body => Line::styled(format!("{}{}", row.lead, row.text), plain),
    }
}

fn draw_row(frame: &mut Frame, row: Rect, name: &str, ctx: &ViewCtx<'_>) {
    let text = format!(
        "{} {} {}",
        ctx.glyphs.focus_right, name, ctx.glyphs.focus_left
    );
    let style = ctx.theme.active_tab();
    let (start, end) = span(row, name);
    let placed = Rect::new(start, row.y, end.saturating_sub(start), row.height).intersection(row);
    frame.render_widget(Paragraph::new(Line::styled(text, style)), placed);
}

/// Draws the placement button on the top border of the box at `frame`, showing `side`.
pub fn draw_button(frame: &mut Frame, boxed: Rect, side: DockSide, ctx: &ViewCtx<'_>) {
    let style = ctx.theme.style(Token::Accent, Token::Surface);
    let label = button_label(side);
    frame.render_widget(
        Paragraph::new(Line::styled(label, style)),
        button_rect(boxed, side),
    );
}

/// Draws the docked window over `rect`: its border and title, the tab and game rows, the wrapped
/// text from `state.scroll` on, the scroll bar, the Close button and the placement button.
pub fn draw(
    frame: &mut Frame,
    rect: Rect,
    modal: &Modal,
    state: &DockState,
    side: DockSide,
    ctx: &ViewCtx<'_>,
) {
    frame.render_widget(Clear, rect);
    let mut title = nav::window_title(modal).into_owned();
    let mut title_style = ctx.theme.style(Token::Accent, Token::Surface);
    if state.focused {
        title = format!("{} {title}", ctx.glyphs.focus_left);
        title_style = title_style.add_modifier(Modifier::BOLD | Modifier::REVERSED);
    }
    let block = Block::bordered()
        .border_set(dialog::border_set(ctx))
        .border_style(ctx.theme.style(Token::BorderFocus, Token::Surface))
        .title(Line::styled(format!(" {title} "), title_style))
        .style(ctx.theme.surface());
    frame.render_widget(block, rect);
    let places = parts(rect);
    if let Some(tab) = nav::tab_of(modal) {
        draw_row(frame, places.top, &t(tab.label_key()), ctx);
    }
    if let Modal::HowToPlay(game) = modal {
        draw_row(frame, places.games, &t(game.rules.name_key()), ctx);
    }
    let rows = body(modal, text_width(rect));
    let first = usize::from(state.scroll);
    let height = usize::from(places.text.height);
    let lines: Vec<Line> = rows
        .iter()
        .skip(first)
        .take(height)
        .map(|row| line_of(row, ctx))
        .collect();
    frame.render_widget(
        Paragraph::new(lines).style(ctx.theme.surface()),
        places.text.inner(Margin::new(1, 0)),
    );
    scrollbar::draw(
        frame,
        scrollbar::track_of(places.text),
        Metrics {
            total: rows.len(),
            view: height,
            offset: first,
        },
        ctx,
    );
    frame.render_widget(
        Paragraph::new(Line::styled(footer_hints(), ctx.theme.surface())),
        hints_rect(rect),
    );
    let close_style = ctx.theme.focused_button();
    frame.render_widget(
        Paragraph::new(Line::styled(close_label(), close_style)),
        close_rect(rect),
    );
    draw_button(frame, rect, side, ctx);
}

/// The compact hints of the docked footer, for example `d Dock  F6 Focus`: the keys of the dock
/// and the pane focus actions from the key table, with their localized words.
pub fn footer_hints() -> String {
    tf(
        "help.dock.footer",
        &[
            ("dock", &primary_key(Action::Dock).unwrap_or_default()),
            ("focus", &primary_key(Action::DockFocus).unwrap_or_default()),
        ],
    )
}

/// The footer cells left of the Close button of a docked window at `rect`, where the hints go.
pub fn hints_rect(rect: Rect) -> Rect {
    let footer = parts(rect).footer;
    let close = close_rect(rect);
    let width = footer.width.saturating_sub(close.width.saturating_add(1));
    Rect::new(footer.x, footer.y, width, footer.height)
}

/// Draws the note that docking is not possible at this size, on the last row of `area`.
pub fn draw_note(frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
    let Some(row) = area.height.checked_sub(1).map(|offset| Rect {
        y: area.y + offset,
        height: 1,
        ..area
    }) else {
        return;
    };
    frame.render_widget(Clear, row);
    frame.render_widget(
        Paragraph::new(Line::styled(t("help.dock.too_narrow"), ctx.theme.surface())),
        row,
    );
}

/// Draws what a centred help window adds to its box: the placement button on the top border, and
/// the note that docking stays off on a screen too narrow for it.
pub fn draw_placement(
    frame: &mut Frame,
    area: Rect,
    modal: &Modal,
    side: DockSide,
    ctx: &ViewCtx<'_>,
) {
    let Some(boxed) = centred_frame_for(modal, area, ctx.settings.interface_size) else {
        return;
    };
    draw_button(frame, boxed, side, ctx);
    if side != DockSide::Centre {
        draw_note(frame, area, ctx);
    }
}

/// Whether `modal` is the help window (its keys, rules or command-line tab), the only dockable one.
pub const fn is_dockable(modal: &Modal) -> bool {
    matches!(
        modal,
        Modal::Help(_) | Modal::HowToPlay(_) | Modal::CommandLine(_)
    )
}

/// The box of a dockable modal drawn centred in `area` at the default interface scale.
#[cfg(test)]
pub fn centred_frame(modal: &Modal, area: Rect) -> Option<Rect> {
    centred_frame_for(modal, area, InterfaceSize::default())
}

/// The box of a dockable modal drawn centred in `area` at `size`; `None` for any other modal.
pub fn centred_frame_for(modal: &Modal, area: Rect, size: InterfaceSize) -> Option<Rect> {
    match modal {
        Modal::Help(_) => Some(help::geometry_for(area, size).frame),
        Modal::HowToPlay(_) => Some(how_to_play::geometry_for(area, size).frame),
        Modal::CommandLine(_) => Some(command_line::geometry_for(area, size).frame),
        _ => None,
    }
}
