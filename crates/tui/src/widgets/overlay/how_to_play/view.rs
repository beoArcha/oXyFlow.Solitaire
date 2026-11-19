//! Drawing and geometry of the "How to play" window: a row of game tabs with arrows, the scrolled
//! text, a footer with the keys and the position, and a Close button. The chosen tab is framed by
//! brackets and drawn bold and reversed, so it shows without color; arrows and tabs are plain
//! WGL4 or ASCII characters.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::content::{self, Kind, Row, VIEW_ROWS};
use super::state::{Focus, HowToPlay, Rules};
use crate::action::Action;
use crate::i18n::{t, tf};
use crate::input::primary_key;
use crate::screens::ViewCtx;
use crate::theme::Token;
use crate::widgets::footer::{FooterMap, Segment};
use crate::widgets::overlay::dialog::{self, Geometry};
use crate::widgets::overlay::help;
use crate::widgets::overlay::tabs::{HelpTab, arrows, draw_footer_buttons, draw_top_tabs};
use crate::widgets::scrollbar;

const CHROME_ROWS: u16 = 3;
const ARROW_COLUMNS: u16 = 3;
const TAB_PADDING: u16 = 2;
const TAB_GAP: u16 = 1;

/// The four bands inside the box: top tabs, game tabs, text and footer.
pub struct Parts {
    pub top_tabs: Rect,
    pub game_tabs: Rect,
    pub text: Rect,
    pub footer: Rect,
}

/// Where the tab strip puts its arrows and tabs.
pub struct Tabs {
    pub previous: Rect,
    pub next: Rect,
    pub games: Vec<(Rules, Rect)>,
}

/// The box of the window, centered in `area`. Its size does not depend on the text: the text
/// scrolls inside a fixed number of rows, and the box fits the smallest supported terminal.
pub fn geometry(area: Rect) -> Geometry {
    let width = crate::widgets::overlay::help::geometry(area).frame.width;
    let rows = u16::try_from(VIEW_ROWS).unwrap_or(u16::MAX) + CHROME_ROWS;
    dialog::boxed(area, width, rows, &help::buttons())
}

/// Splits the box body into top tabs, game tabs, text rows and the footer row.
pub fn parts(body: Rect) -> Parts {
    let [top_tabs, game_tabs, text, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(body);
    Parts {
        top_tabs,
        game_tabs,
        text,
        footer,
    }
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

fn draw_tabs(frame: &mut Frame, row: Rect, state: &HowToPlay, ctx: &ViewCtx<'_>) {
    let placed = tabs(row);
    let focused = state.focus == Focus::Games;
    let plain = ctx.theme.surface();
    if focused {
        let (left, right) = arrows(ctx);
        frame.render_widget(Paragraph::new(Line::styled(left, plain)), placed.previous);
        frame.render_widget(Paragraph::new(Line::styled(right, plain)), placed.next);
    }
    for (rules, rect) in placed.games {
        let name = t(rules.name_key());
        let (text, style) = if rules == state.rules {
            let selected = if focused {
                plain.add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                plain
            };
            (format!("[{name}]"), selected)
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

fn footer_segments(focus: Focus) -> Vec<Segment> {
    let move_keys = tf(
        footer_key(focus),
        &[
            ("left", &key_name(Action::Left)),
            ("right", &key_name(Action::Right)),
            ("up", &key_name(Action::Up)),
            ("down", &key_name(Action::Down)),
        ],
    );
    vec![
        Segment::plain(move_keys),
        Segment::plain("   "),
        Segment::button(Action::Back, close_label()),
    ]
}

fn footer_key(focus: Focus) -> &'static str {
    match focus {
        Focus::Tabs => "help.rules.footer_tabs",
        Focus::Games => "help.rules.footer_games",
        Focus::Text => "help.rules.footer_text",
    }
}

fn close_label() -> String {
    tf(
        "help.rules.footer_close",
        &[("back", &key_name(Action::Back))],
    )
}

/// Draws the window over `area` and returns its footer buttons.
pub fn draw(frame: &mut Frame, area: Rect, state: &HowToPlay, ctx: &ViewCtx<'_>) -> FooterMap {
    let geometry = geometry(area);
    dialog::draw_box(frame, &geometry, &t("common.dialog.help_title"), ctx);
    let parts = parts(geometry.body);
    draw_top_tabs(
        frame,
        parts.top_tabs,
        HelpTab::Rules,
        state.focus == Focus::Tabs,
        ctx,
    );
    draw_tabs(frame, parts.game_tabs, state, ctx);
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
    if state.focus == Focus::Text {
        let mark = Line::styled(ctx.glyphs.focus_left, ctx.theme.surface());
        frame.render_widget(
            Paragraph::new(mark),
            Rect {
                height: 1,
                ..parts.text
            },
        );
    }
    scrollbar::draw(
        frame,
        scrollbar::track_of(parts.text),
        state.metrics(parts.text.height),
        ctx,
    );
    let buttons_drawn = draw_footer_buttons(
        frame,
        parts.footer,
        &footer_segments(state.focus),
        (first + 1, first + shown, rows.len()),
        ctx,
    );
    dialog::draw_buttons(frame, &geometry, &help::buttons(), ctx);
    buttons_drawn
}
