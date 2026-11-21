//! The help window: every binding of the key table grouped by topic, then the mouse gestures. It is
//! generated from [`KEY_TABLE`], so a new binding appears here without further work. The topics
//! sit in two columns, one under the other without blank rows (the bold heading separates them), so
//! the whole list fits a 100 × 30 terminal.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::dialog::{self, Button, Geometry};
use super::help_keys::HelpKeys;
use super::tabs::{HelpTab, draw_top_tabs};
use crate::action::{Action, HelpGroup};
use crate::i18n::{t, tf};
use crate::input::{KEY_TABLE, KeySpec, primary_key};
use crate::screens::ViewCtx;
use crate::theme::Token;
use crate::widgets::footer::{self, FooterMap, Segment};
use crate::widgets::scrollbar;

const GAP: u16 = 4;
const KEY_PADDING: usize = 2;
const LEFT_TOPICS: usize = 2;
const DIALOG_MIN_WIDTH: u16 = 80;
/// The share of the terminal height the window takes, in percent. A share of 60 covers the logo,
/// whose top row sits at about a quarter of the height, from above at every terminal height.
const SHARE_PERCENT: u16 = 60;
const MIN_FRAME_ROWS: u16 = 28;
const MAX_FRAME_ROWS: u16 = 40;
/// Rows of the box that are not text on the Keys tab: the borders, the tab row, the separator row,
/// the footer row and the button row.
const KEYS_CHROME_ROWS: u16 = 7;
/// Rows of the box that are not in its body: the borders and the button row.
const BOX_CHROME_ROWS: u16 = 4;
/// Text rows of the Keys tab at 100 by 30, the smallest window: the first value before any event.
pub(super) const VIEW_ROWS: usize = 21;
const RULE_UNICODE: &str = "─";
const RULE_ASCII: &str = "-";

/// Rows of the box, borders and buttons included, in `area`. Every Help tab uses this height, so
/// switching tabs does not move the window.
pub(super) fn frame_rows(area: Rect) -> u16 {
    let share = area.height.saturating_mul(SHARE_PERCENT) / 100;
    share.clamp(MIN_FRAME_ROWS, MAX_FRAME_ROWS).min(area.height)
}

/// Text rows of the Keys tab and the Command line tab in `area`.
pub(super) fn view_rows(area: Rect) -> u16 {
    frame_rows(area).saturating_sub(KEYS_CHROME_ROWS)
}
/// The top tab row, the text, the separator row and the footer row of the box body.
pub(super) fn parts(body: Rect) -> (Rect, Rect, Rect, Rect) {
    let [top, text, rule, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(body);
    (top, text, rule, footer)
}

/// Draws the separator row between the scrolling text and the footer: a rule of `─`, or `-` in
/// ASCII mode, in the border colour. Every Help tab draws it, so the footer never touches the text.
pub(super) fn draw_rule(frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
    let glyph = if ctx.glyphs.unicode {
        RULE_UNICODE
    } else {
        RULE_ASCII
    };
    frame.render_widget(
        Paragraph::new(Line::styled(
            glyph.repeat(usize::from(area.width)),
            ctx.theme.style(Token::Border, Token::Surface),
        )),
        area,
    );
}

const GESTURES: [(&str, &str); 5] = [
    ("help.mouse_keys.click_card", "help.mouse.click_card"),
    ("help.mouse_keys.click_stock", "help.mouse.click_stock"),
    ("help.mouse_keys.double_click", "help.mouse.double_click"),
    ("help.mouse_keys.right_click", "help.mouse.right_click"),
    ("help.mouse_keys.drag", "help.mouse.drag"),
];

/// One line of help: the keys and what they do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub keys: String,
    pub text: String,
}

/// A topic of help with its rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub title: String,
    pub rows: Vec<Row>,
}

/// The row that names the zoom of the terminal itself, which is the last row of the Display topic:
/// nearly every terminal zooms its font with these keys and never passes them on, the classic
/// Windows console passes them on and the game changes its font size.
fn terminal_zoom(group: HelpGroup) -> Option<Row> {
    (group == HelpGroup::Display).then(|| Row {
        keys: t("help.display.terminal_keys").into_owned(),
        text: t("help.display.terminal_text").into_owned(),
    })
}

/// Every topic: the key table grouped by [`HelpGroup`], then the mouse gestures.
pub fn sections() -> Vec<Section> {
    let mut sections: Vec<Section> = HelpGroup::ALL
        .into_iter()
        .map(|group| Section {
            title: t(group.title_key()).into_owned(),
            rows: KEY_TABLE
                .iter()
                .filter(|binding| binding.group == group)
                .map(|binding| Row {
                    keys: binding
                        .keys
                        .iter()
                        .map(KeySpec::display)
                        .collect::<Vec<_>>()
                        .join(" / "),
                    text: t(binding.action.label_key()).into_owned(),
                })
                .chain(terminal_zoom(group))
                .collect(),
        })
        .collect();
    sections.push(Section {
        title: t("help.group.mouse").into_owned(),
        rows: GESTURES
            .iter()
            .map(|(keys, text)| Row {
                keys: t(keys).into_owned(),
                text: t(text).into_owned(),
            })
            .collect(),
    });
    sections
}

/// The two columns of help and the width of their key column.
pub(super) struct Columns {
    pub(super) left: Vec<Section>,
    pub(super) right: Vec<Section>,
    key_width: usize,
}

impl Columns {
    pub(super) fn new() -> Self {
        let mut sections = sections();
        let right = sections.split_off(LEFT_TOPICS.min(sections.len()));
        let key_width = sections
            .iter()
            .chain(&right)
            .flat_map(|section| &section.rows)
            .map(|row| row.keys.chars().count())
            .max()
            .unwrap_or(0)
            + KEY_PADDING;
        Self {
            left: sections,
            right,
            key_width,
        }
    }

    pub(super) fn width_of(&self, sections: &[Section]) -> u16 {
        let text = sections
            .iter()
            .flat_map(|section| &section.rows)
            .map(|row| row.text.chars().count())
            .max()
            .unwrap_or(0);
        u16::try_from(self.key_width + text).unwrap_or(u16::MAX)
    }

    pub(super) fn lines(&self, sections: &[Section], ctx: &ViewCtx<'_>) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        for section in sections {
            lines.push(Line::styled(
                section.title.clone(),
                ctx.theme.surface().add_modifier(Modifier::BOLD),
            ));
            for row in &section.rows {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{:<width$}", row.keys, width = self.key_width),
                        ctx.theme.style(Token::Accent, Token::Surface),
                    ),
                    Span::styled(row.text.clone(), ctx.theme.surface()),
                ]));
            }
        }
        lines
    }
}

/// The Close button every help-window tab and the guide window share; its label names the key.
pub(crate) fn buttons() -> Vec<Button> {
    let key = primary_key(Action::Back).unwrap_or_default();
    vec![Button {
        label: tf("help.rules.close", &[("key", &key)]),
        focused: true,
    }]
}

/// The box of the help window, centered in `area`.
pub(super) fn geometry(area: Rect) -> Geometry {
    let columns = Columns::new();
    let width = (columns.width_of(&columns.left) + GAP + columns.width_of(&columns.right) + 4)
        .max(DIALOG_MIN_WIDTH);
    dialog::boxed(
        area,
        width,
        frame_rows(area).saturating_sub(BOX_CHROME_ROWS),
        &buttons(),
    )
}

/// The footer of the keys tab: the two keys that close it, each a button.
fn close_segments() -> Vec<Segment> {
    let key = |action: Action| primary_key(action).unwrap_or_default();
    vec![
        Segment::button(Action::Back, key(Action::Back)),
        Segment::plain(t("help.overlay.close_or")),
        Segment::button(Action::Select, key(Action::Select)),
        Segment::plain(t("help.overlay.close_rest")),
    ]
}

/// Draws the help window over `area` and returns its footer buttons.
pub fn draw(frame: &mut Frame, area: Rect, state: &HelpKeys, ctx: &ViewCtx<'_>) -> FooterMap {
    let geometry = geometry(area);
    dialog::draw_box(frame, &geometry, &t("common.dialog.help_title"), ctx);
    let columns = Columns::new();
    let (top_tab_row, text, rule, footer) = parts(geometry.body);
    draw_top_tabs(frame, top_tab_row, HelpTab::Keys, true, ctx);
    let body = Rect {
        width: text.width.saturating_sub(1),
        ..text
    };
    let [left, _, right] = Layout::horizontal([
        Constraint::Length(columns.width_of(&columns.left)),
        Constraint::Length(GAP),
        Constraint::Fill(1),
    ])
    .areas(body);
    let first = usize::from(state.scroll);
    let rows = usize::from(text.height);
    let column = |sections: &[Section]| -> Vec<Line<'static>> {
        columns
            .lines(sections, ctx)
            .into_iter()
            .skip(first)
            .take(rows)
            .collect()
    };
    frame.render_widget(
        Paragraph::new(column(&columns.left)).style(ctx.theme.surface()),
        left,
    );
    frame.render_widget(
        Paragraph::new(column(&columns.right)).style(ctx.theme.surface()),
        right,
    );
    scrollbar::draw(frame, scrollbar::track_of(text), state.metrics(), ctx);
    draw_rule(frame, rule, ctx);
    let buttons_drawn = footer::draw(
        frame,
        footer,
        &close_segments(),
        Alignment::Center,
        ctx.theme.surface(),
        ctx,
    );
    dialog::draw_buttons(frame, &geometry, &buttons(), ctx);
    buttons_drawn
}
