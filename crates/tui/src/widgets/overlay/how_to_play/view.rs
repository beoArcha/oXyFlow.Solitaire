//! Drawing and geometry of the "How to play" window: a row of game tabs with arrows, the scrolled
//! text, a footer with the keys and the position, and a Close button. The chosen tab is framed by
//! brackets and drawn bold and underlined in the accent color, so it shows without color and stays
//! distinct from the focused button; arrows and tabs are plain WGL4 or ASCII characters.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::content::{self, Kind, Row};
use super::state::{HowToPlay, Rules};
use crate::i18n::t;
use crate::screens::ViewCtx;
use crate::theme::Token;
use crate::widgets::footer::FooterMap;
use crate::widgets::overlay::dialog::{self, Geometry};
use crate::widgets::overlay::help;
use crate::widgets::overlay::tabs::{
    HelpTab, arrows, draw_footer_buttons, draw_top_tabs, help_footer_segments,
};
use crate::widgets::scrollbar;

/// Rows of the window that are not text: the borders, the button row, the top tab row, the game
/// tab row, the separator row and the footer row.
const CHROME_ROWS: u16 = 8;
const ARROW_COLUMNS: u16 = 3;
const TAB_PADDING: u16 = 2;
const TAB_GAP: u16 = 1;

/// The five bands inside the box: top tabs, game tabs, text, separator and footer.
pub struct Parts {
    pub top_tabs: Rect,
    pub game_tabs: Rect,
    pub text: Rect,
    pub rule: Rect,
    pub footer: Rect,
}

/// Where the tab strip puts its arrows and tabs.
pub struct Tabs {
    pub previous: Rect,
    pub next: Rect,
    pub games: Vec<(Rules, Rect)>,
}

/// The box of the window, centered in `area`: the same box as every other Help tab, so switching
/// tabs does not move the window.
pub fn geometry(area: Rect) -> Geometry {
    help::geometry(area)
}

/// Text rows the box shows at `area`: the box less the rows of the tabs and the footer.
pub fn text_rows(area: Rect) -> u16 {
    help::frame_rows(area).saturating_sub(CHROME_ROWS)
}

/// Splits the box body into top tabs, game tabs, text rows, the separator row and the footer row.
pub fn parts(body: Rect) -> Parts {
    let [top_tabs, game_tabs, text, rule, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(body);
    Parts {
        top_tabs,
        game_tabs,
        text,
        rule,
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
    let plain = ctx.theme.surface();
    let (left, right) = arrows(ctx);
    frame.render_widget(Paragraph::new(Line::styled(left, plain)), placed.previous);
    frame.render_widget(Paragraph::new(Line::styled(right, plain)), placed.next);
    for (rules, rect) in placed.games {
        let name = t(rules.name_key());
        let (text, style) = if rules == state.rules {
            (format!("[{name}]"), ctx.theme.active_tab())
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

/// Draws the window over `area` and returns its footer buttons.
pub fn draw(frame: &mut Frame, area: Rect, state: &HowToPlay, ctx: &ViewCtx<'_>) -> FooterMap {
    let geometry = geometry(area);
    dialog::draw_box(frame, &geometry, &t(state.title_key()), ctx);
    let parts = parts(geometry.body);
    draw_top_tabs(frame, parts.top_tabs, HelpTab::Rules, false, ctx);
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
    scrollbar::draw(
        frame,
        scrollbar::track_of(parts.text),
        state.metrics(parts.text.height),
        ctx,
    );
    help::draw_rule(frame, parts.rule, ctx);
    let buttons_drawn = draw_footer_buttons(
        frame,
        parts.footer,
        &help_footer_segments(true),
        (first + 1, first + shown, rows.len()),
        ctx,
    );
    dialog::draw_buttons(frame, &geometry, &help::buttons(), ctx);
    buttons_drawn
}
