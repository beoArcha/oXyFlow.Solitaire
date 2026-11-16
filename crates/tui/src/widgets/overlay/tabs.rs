//! The top tabs of the help window ("Keys", "Rules" and "Command line") and the footer row that the
//! scrolling tabs share.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Margin, Rect};
use ratatui::style::Modifier;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use crate::i18n::{t, tf};
use crate::screens::ViewCtx;
use crate::theme::Token;

const TAB_GAP: u16 = 2;

/// Which tab of the help window is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpTab {
    Keys,
    Rules,
    CommandLine,
}

impl HelpTab {
    /// Every top tab in display order.
    pub const ALL: [HelpTab; 3] = [HelpTab::Keys, HelpTab::Rules, HelpTab::CommandLine];

    /// Localization key for the tab label.
    pub const fn label_key(self) -> &'static str {
        match self {
            HelpTab::Keys => "help.tab.keys",
            HelpTab::Rules => "help.tab.rules",
            HelpTab::CommandLine => "help.tab.command_line",
        }
    }

    /// The tab to the right, wrapping around.
    pub const fn next(self) -> Self {
        match self {
            HelpTab::Keys => HelpTab::Rules,
            HelpTab::Rules => HelpTab::CommandLine,
            HelpTab::CommandLine => HelpTab::Keys,
        }
    }

    /// The tab to the left, wrapping around.
    pub const fn previous(self) -> Self {
        match self {
            HelpTab::Keys => HelpTab::CommandLine,
            HelpTab::Rules => HelpTab::Keys,
            HelpTab::CommandLine => HelpTab::Rules,
        }
    }
}

/// Where the top tab strip puts its tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TopTabs {
    pub keys: Rect,
    pub rules: Rect,
    pub command_line: Rect,
}

impl TopTabs {
    /// The rectangle of `tab`.
    pub const fn rect(&self, tab: HelpTab) -> Rect {
        match tab {
            HelpTab::Keys => self.keys,
            HelpTab::Rules => self.rules,
            HelpTab::CommandLine => self.command_line,
        }
    }

    /// The tab under `cell`, if any.
    pub fn tab_at(&self, cell: (u16, u16)) -> Option<HelpTab> {
        HelpTab::ALL
            .into_iter()
            .find(|tab| crate::mouse::contains(self.rect(*tab), cell))
    }
}

fn columns_of(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(u16::MAX)
}

fn tab_width(tab: HelpTab) -> u16 {
    columns_of(&t(tab.label_key())).saturating_add(2)
}

/// The row of the box body that holds the top tabs.
pub fn top_row(body: Rect) -> Rect {
    let [row, _] = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(body);
    row
}

/// Places the three tabs centered in `row`.
pub fn top_tabs(row: Rect) -> TopTabs {
    let widths = HelpTab::ALL.map(tab_width);
    let total = widths
        .iter()
        .fold(0_u16, |sum, width| sum.saturating_add(*width))
        .saturating_add(TAB_GAP * 2);
    let mut x = row.x.saturating_add(row.width.saturating_sub(total) / 2);
    let [keys, rules, command_line] = widths.map(|width| {
        let rect = Rect::new(x, row.y, width, 1).intersection(row);
        x = x.saturating_add(width).saturating_add(TAB_GAP);
        rect
    });
    TopTabs {
        keys,
        rules,
        command_line,
    }
}

/// Draws the tabs in `row`.
pub fn draw_top_tabs(frame: &mut Frame, row: Rect, current: HelpTab, ctx: &ViewCtx<'_>) {
    let placed = top_tabs(row);
    let plain = ctx.theme.surface();
    for tab in HelpTab::ALL {
        let name = t(tab.label_key());
        let (text, style) = if tab == current {
            (
                format!("[{name}]"),
                plain.add_modifier(Modifier::BOLD | Modifier::REVERSED),
            )
        } else {
            (format!(" {name} "), plain)
        };
        frame.render_widget(Paragraph::new(Line::styled(text, style)), placed.rect(tab));
    }
}

/// Draws the footer of a scrolling tab: `hints` on the left and the visible range on the right.
pub fn draw_footer(
    frame: &mut Frame,
    row: Rect,
    hints: String,
    shown: (usize, usize, usize),
    ctx: &ViewCtx<'_>,
) {
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
