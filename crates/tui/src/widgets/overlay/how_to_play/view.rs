//! Drawing and geometry of the "How to play" window: a row of game tabs with arrows, the scrolled
//! text, a footer with the keys and the position, and a Close button. The chosen tab is framed by
//! brackets and drawn bold and reversed, so it shows without color; arrows and tabs are plain
//! WGL4 or ASCII characters.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Margin, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::content::{self, Kind, Row, VIEW_ROWS};
use super::state::{HowToPlay, Rules};
use crate::action::Action;
use crate::i18n::{t, tf};
use crate::input::primary_key;
use crate::screens::ViewCtx;
use crate::theme::Token;
use crate::widgets::overlay::dialog::{self, Button, Geometry};

const DIALOG_WIDTH: u16 = 80;
const CHROME_ROWS: u16 = 2;
const ARROW_COLUMNS: u16 = 3;
const TAB_PADDING: u16 = 2;
const TAB_GAP: u16 = 1;

/// The three bands inside the box: tabs, text and footer.
pub struct Parts {
    pub tabs: Rect,
    pub text: Rect,
    pub footer: Rect,
}

/// Where the tab strip puts its arrows and tabs.
pub struct Tabs {
    pub previous: Rect,
    pub next: Rect,
    pub games: Vec<(Rules, Rect)>,
}

fn buttons() -> Vec<Button> {
    vec![Button {
        label: t("help.rules.close").into_owned(),
        focused: true,
    }]
}

/// The box of the window, centered in `area`. Its size does not depend on the text: the text
/// scrolls inside a fixed number of rows, and the box fits the smallest supported terminal.
pub fn geometry(area: Rect) -> Geometry {
    let rows = u16::try_from(VIEW_ROWS).unwrap_or(u16::MAX) + CHROME_ROWS;
    dialog::boxed(area, DIALOG_WIDTH, rows, &buttons())
}

/// Splits the box body into the tab row, the text rows and the footer row.
pub fn parts(body: Rect) -> Parts {
    let [tabs, text, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(body);
    Parts { tabs, text, footer }
}

fn columns_of(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(u16::MAX)
}

fn tab_width(rules: Rules) -> u16 {
    columns_of(&t(rules.name_key())).saturating_add(TAB_PADDING)
}

/// Places the arrows and tabs centered in `row`.
pub fn tabs(row: Rect) -> Tabs {
    let widths: Vec<u16> = Rules::ALL.iter().map(|rules| tab_width(*rules)).collect();
    let gaps = u16::try_from(widths.len() + 1).unwrap_or(0) * TAB_GAP;
    let total = ARROW_COLUMNS * 2 + widths.iter().sum::<u16>() + gaps;
    let mut x = row.x.saturating_add(row.width.saturating_sub(total) / 2);
    let mut place = |width: u16| {
        let rect = Rect::new(x, row.y, width, 1).intersection(row);
        x = x.saturating_add(width).saturating_add(TAB_GAP);
        rect
    };
    let previous = place(ARROW_COLUMNS);
    let games = Rules::ALL
        .iter()
        .zip(&widths)
        .map(|(rules, width)| (*rules, place(*width)))
        .collect();
    let next = place(ARROW_COLUMNS);
    Tabs {
        previous,
        next,
        games,
    }
}

fn arrows(ctx: &ViewCtx<'_>) -> (&'static str, &'static str) {
    if ctx.glyphs.unicode {
        (" ◄ ", " ► ")
    } else {
        (" < ", " > ")
    }
}

fn draw_tabs(frame: &mut Frame, row: Rect, state: &HowToPlay, ctx: &ViewCtx<'_>) {
    let placed = tabs(row);
    let (left, right) = arrows(ctx);
    let plain = ctx.theme.surface();
    frame.render_widget(Paragraph::new(Line::styled(left, plain)), placed.previous);
    frame.render_widget(Paragraph::new(Line::styled(right, plain)), placed.next);
    for (rules, rect) in placed.games {
        let name = t(rules.name_key());
        let (text, style) = if rules == state.rules {
            (
                format!("[{name}]"),
                plain.add_modifier(Modifier::BOLD | Modifier::REVERSED),
            )
        } else {
            (format!(" {name} "), plain)
        };
        frame.render_widget(Paragraph::new(Line::styled(text, style)), rect);
    }
}

fn line_of(row: &Row, ctx: &ViewCtx<'_>) -> Line<'static> {
    let plain = ctx.theme.surface();
    match row.kind {
        Kind::Heading => Line::styled(row.text.clone(), plain.add_modifier(Modifier::BOLD)),
        Kind::Body => Line::from(vec![
            Span::styled(row.lead.clone(), plain),
            Span::styled(row.text.clone(), plain),
        ]),
        Kind::Key => Line::from(vec![
            Span::styled(
                row.lead.clone(),
                ctx.theme.style(Token::Accent, Token::Surface),
            ),
            Span::styled(row.text.clone(), plain),
        ]),
    }
}

fn key_name(action: Action) -> String {
    primary_key(action).unwrap_or_default()
}

fn draw_footer(frame: &mut Frame, row: Rect, shown: (usize, usize, usize), ctx: &ViewCtx<'_>) {
    let hints = tf(
        "help.rules.footer",
        &[
            ("left", &key_name(Action::Left)),
            ("right", &key_name(Action::Right)),
            ("up", &key_name(Action::Up)),
            ("down", &key_name(Action::Down)),
            ("back", &key_name(Action::Back)),
        ],
    );
    let (from, to, total) = shown;
    let position = tf(
        "help.rules.position",
        &[
            ("from", &from.to_string()),
            ("to", &to.to_string()),
            ("total", &total.to_string()),
        ],
    );
    let [left, right] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(columns_of(&position)),
    ])
    .areas(row.inner(Margin::new(1, 0)));
    let muted = ctx.theme.style(Token::TextMuted, Token::Surface);
    frame.render_widget(Paragraph::new(Line::styled(hints, muted)), left);
    frame.render_widget(
        Paragraph::new(Line::styled(position, muted)).alignment(Alignment::Right),
        right,
    );
}

/// Draws the window over `area`.
pub fn draw(frame: &mut Frame, area: Rect, state: &HowToPlay, ctx: &ViewCtx<'_>) {
    let geometry = geometry(area);
    dialog::draw_box(frame, &geometry, &t("help.rules.title"), ctx);
    let parts = parts(geometry.body);
    draw_tabs(frame, parts.tabs, state, ctx);
    let rows = content::rows(state.rules);
    let first = usize::from(state.scroll);
    let lines: Vec<Line> = rows
        .iter()
        .skip(first)
        .take(usize::from(parts.text.height))
        .map(|row| line_of(row, ctx))
        .collect();
    let shown = lines.len();
    frame.render_widget(
        Paragraph::new(lines).style(ctx.theme.surface()),
        parts.text.inner(Margin::new(1, 0)),
    );
    draw_footer(
        frame,
        parts.footer,
        (first + 1, first + shown, rows.len()),
        ctx,
    );
    dialog::draw_buttons(frame, &geometry, &buttons(), ctx);
}
