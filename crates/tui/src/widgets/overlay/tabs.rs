//! The top tabs of the help window ("Keys", "Rules" and "Command line") and the footer row that the
//! scrolling tabs share.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Margin, Rect};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use crate::action::Action;
use crate::i18n::{t, tf};
use crate::input::primary_key;
use crate::screens::ViewCtx;
use crate::theme::Token;
use crate::widgets::footer::{self, FooterMap, Segment};

const TAB_GAP: u16 = 2;
const ARROW_COLUMNS: u16 = 3;

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
    pub previous: Rect,
    pub next: Rect,
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

    /// The tab a click on an arrow goes to, if `cell` lies on one.
    pub fn arrow_target(&self, current: HelpTab, cell: (u16, u16)) -> Option<HelpTab> {
        if crate::mouse::contains(self.previous, cell) {
            Some(current.previous())
        } else if crate::mouse::contains(self.next, cell) {
            Some(current.next())
        } else {
            None
        }
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
        previous: Rect::new(
            keys.x.saturating_sub(ARROW_COLUMNS),
            row.y,
            ARROW_COLUMNS,
            1,
        )
        .intersection(row),
        next: Rect::new(command_line.right(), row.y, ARROW_COLUMNS, 1).intersection(row),
    }
}

/// The arrows that frame a selector row while it has the focus.
pub fn arrows(ctx: &ViewCtx<'_>) -> (String, String) {
    (
        format!(" {} ", ctx.glyphs.focus_right),
        format!(" {} ", ctx.glyphs.focus_left),
    )
}

/// Draws the tabs in `row`. The row that has the focus shows the arrows and its tab in bold
/// underlined accent text; a row without the focus shows its tab in brackets, in plain type.
pub fn draw_top_tabs(
    frame: &mut Frame,
    row: Rect,
    current: HelpTab,
    focused: bool,
    ctx: &ViewCtx<'_>,
) {
    let placed = top_tabs(row);
    let plain = ctx.theme.surface();
    if focused {
        let (left, right) = arrows(ctx);
        frame.render_widget(Paragraph::new(Line::styled(left, plain)), placed.previous);
        frame.render_widget(Paragraph::new(Line::styled(right, plain)), placed.next);
    }
    for tab in HelpTab::ALL {
        let name = t(tab.label_key());
        let (text, style) = if tab == current {
            let selected = if focused {
                ctx.theme.active_tab()
            } else {
                plain
            };
            (format!("[{name}]"), selected)
        } else {
            (format!(" {name} "), plain)
        };
        frame.render_widget(Paragraph::new(Line::styled(text, style)), placed.rect(tab));
    }
}

fn key_name(action: Action) -> String {
    primary_key(action).unwrap_or_default()
}

/// The footer hints shared by the Help window tabs: main tab switch, optional game switch on
/// Rules, scrolling, and closing.
pub fn help_footer_segments(game_switch: bool) -> Vec<Segment> {
    let mut segments = vec![
        Segment::button(
            Action::FocusNext,
            tf(
                "help.command_line.footer_tab",
                &[("key", &key_name(Action::FocusNext))],
            ),
        ),
        Segment::plain("  "),
    ];
    if game_switch {
        segments.push(Segment::plain(tf(
            "help.rules.footer_game",
            &[
                ("left", &key_name(Action::Left)),
                ("right", &key_name(Action::Right)),
            ],
        )));
        segments.push(Segment::plain("  "));
    }
    segments.push(Segment::plain(tf(
        "help.command_line.footer_move",
        &[
            ("up", &key_name(Action::Up)),
            ("down", &key_name(Action::Down)),
        ],
    )));
    segments.push(Segment::plain("  "));
    segments.push(Segment::button(
        Action::Back,
        tf(
            "help.command_line.footer_close",
            &[("back", &key_name(Action::Back))],
        ),
    ));
    segments
}

/// Draws the footer of a scrolling tab: the segments on the left, and the visible range on the
/// right whenever the text overflows.
pub fn draw_footer_buttons(
    frame: &mut Frame,
    row: Rect,
    segments: &[Segment],
    shown: (usize, usize, usize),
    ctx: &ViewCtx<'_>,
) -> FooterMap {
    let (from, to, total) = shown;
    let overflow = total > 0 && (from > 1 || to < total);
    let position = if overflow {
        tf(
            "help.rules.position",
            &[
                ("from", &from.to_string()),
                ("to", &to.to_string()),
                ("total", &total.to_string()),
            ],
        )
    } else {
        String::new()
    };
    let pos_width = if overflow {
        columns_of(&position).saturating_add(1)
    } else {
        0
    };
    let [left, right] = Layout::horizontal([Constraint::Fill(1), Constraint::Length(pos_width)])
        .areas(row.inner(Margin::new(1, 0)));
    let muted = ctx.theme.style(Token::TextMuted, Token::Surface);
    let buttons = footer::draw(frame, left, segments, Alignment::Left, muted, ctx);
    if overflow {
        frame.render_widget(
            Paragraph::new(Line::styled(position, muted)).alignment(Alignment::Right),
            right,
        );
    }
    buttons
}
