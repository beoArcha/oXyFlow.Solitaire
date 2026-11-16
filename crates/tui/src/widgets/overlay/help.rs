//! The help window: every binding of the key table grouped by topic, then the mouse gestures. It is
//! generated from [`KEY_TABLE`], so a new binding appears here without further work. The topics
//! sit in two columns so the whole list fits a 100 × 30 terminal.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::dialog::{self, Button, Geometry};
use super::tabs::{HelpTab, draw_top_tabs};
use crate::action::HelpGroup;
use crate::i18n::t;
use crate::input::{KEY_TABLE, KeySpec};
use crate::screens::ViewCtx;
use crate::theme::Token;

const GAP: u16 = 4;
const KEY_PADDING: usize = 2;
const LEFT_TOPICS: usize = 2;
const BODY_ROWS: u16 = 23;
const DIALOG_MIN_WIDTH: u16 = 80;

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
struct Columns {
    left: Vec<Section>,
    right: Vec<Section>,
    key_width: usize,
}

impl Columns {
    fn new() -> Self {
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

    fn width_of(&self, sections: &[Section]) -> u16 {
        let text = sections
            .iter()
            .flat_map(|section| &section.rows)
            .map(|row| row.text.chars().count())
            .max()
            .unwrap_or(0);
        u16::try_from(self.key_width + text).unwrap_or(u16::MAX)
    }

    fn lines(&self, sections: &[Section], ctx: &ViewCtx<'_>) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        for (index, section) in sections.iter().enumerate() {
            if index > 0 {
                lines.push(Line::raw(""));
            }
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

pub(crate) fn buttons() -> Vec<Button> {
    vec![Button {
        label: t("help.rules.close").into_owned(),
        focused: true,
    }]
}

/// The box of the help window, centered in `area`.
pub fn geometry(area: Rect) -> Geometry {
    let columns = Columns::new();
    let width = (columns.width_of(&columns.left) + GAP + columns.width_of(&columns.right) + 4)
        .max(DIALOG_MIN_WIDTH);
    dialog::boxed(area, width, BODY_ROWS, &buttons())
}

/// Draws the help window over `area`.
pub fn draw(frame: &mut Frame, area: Rect, ctx: &ViewCtx<'_>) {
    let geometry = geometry(area);
    dialog::draw_box(frame, &geometry, &t("common.dialog.help_title"), ctx);
    let columns = Columns::new();
    let [top_tab_row, body, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(geometry.body);
    draw_top_tabs(frame, top_tab_row, HelpTab::Keys, ctx);
    let [left, _, right] = Layout::horizontal([
        Constraint::Length(columns.width_of(&columns.left)),
        Constraint::Length(GAP),
        Constraint::Fill(1),
    ])
    .areas(body);
    frame.render_widget(
        Paragraph::new(columns.lines(&columns.left, ctx)).style(ctx.theme.surface()),
        left,
    );
    frame.render_widget(
        Paragraph::new(columns.lines(&columns.right, ctx)).style(ctx.theme.surface()),
        right,
    );
    frame.render_widget(
        Paragraph::new(Line::raw(t("help.overlay.help_footer")))
            .alignment(Alignment::Center)
            .style(ctx.theme.surface()),
        footer,
    );
    dialog::draw_buttons(frame, &geometry, &buttons(), ctx);
}
