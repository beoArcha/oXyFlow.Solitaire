//! The top tabs of the help window: "Keys" and "Rules".

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use crate::i18n::t;
use crate::screens::ViewCtx;

const TAB_GAP: u16 = 2;

/// Which tab of the help window is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpTab {
    Keys,
    Rules,
}

impl HelpTab {
    /// Every top tab in display order.
    pub const ALL: [HelpTab; 2] = [HelpTab::Keys, HelpTab::Rules];

    /// Localization key for the tab label.
    pub const fn label_key(self) -> &'static str {
        match self {
            HelpTab::Keys => "help.tab.keys",
            HelpTab::Rules => "help.tab.rules",
        }
    }
}

/// Where the top tab strip puts its tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TopTabs {
    pub keys: Rect,
    pub rules: Rect,
}

fn columns_of(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(u16::MAX)
}

fn tab_width(tab: HelpTab) -> u16 {
    columns_of(&t(tab.label_key())).saturating_add(2)
}

/// Places the "Keys" and "Rules" tabs centered in `row`.
pub fn top_tabs(row: Rect) -> TopTabs {
    let keys_w = tab_width(HelpTab::Keys);
    let rules_w = tab_width(HelpTab::Rules);
    let total = keys_w.saturating_add(TAB_GAP).saturating_add(rules_w);
    let x = row.x.saturating_add(row.width.saturating_sub(total) / 2);
    let keys = Rect::new(x, row.y, keys_w, 1).intersection(row);
    let rules = Rect::new(
        x.saturating_add(keys_w).saturating_add(TAB_GAP),
        row.y,
        rules_w,
        1,
    )
    .intersection(row);
    TopTabs { keys, rules }
}

/// Draws the "Keys" and "Rules" tabs in `row`.
pub fn draw_top_tabs(frame: &mut Frame, row: Rect, current: HelpTab, ctx: &ViewCtx<'_>) {
    let placed = top_tabs(row);
    let plain = ctx.theme.surface();
    for (tab, rect) in [(HelpTab::Keys, placed.keys), (HelpTab::Rules, placed.rules)] {
        let name = t(tab.label_key());
        let (text, style) = if tab == current {
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
