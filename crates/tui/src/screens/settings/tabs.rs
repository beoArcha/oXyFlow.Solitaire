//! The three tabs of the settings screen (`Audio & Visual`, `Popular` by default, `Gameplay`),
//! their rows and the centered tab bar with active-tab styling and mouse hit targets.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;

use super::options::Field;
use crate::i18n::t;
use crate::mouse::contains;
use crate::screens::ViewCtx;
use crate::theme::Token;

const ARROW_WIDTH: u16 = 2;
const SEPARATOR_WIDTH: u16 = 3;
const TAB_PADDING: u16 = 2;

/// One of the three settings tabs, in left-to-right display order.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Tab {
    AudioVisual,
    #[default]
    Popular,
    Gameplay,
}

const AUDIO_VISUAL_FIELDS: [Field; 26] = [
    Field::Colors,
    Field::FullScreen,
    Field::WindowSize,
    Field::LowVision,
    Field::Theme,
    Field::ColorVision,
    Field::CardBack,
    Field::CardFace,
    Field::FourColorDeck,
    Field::TableColour,
    Field::TableFeel,
    Field::FrontStyle,
    Field::BackStyle,
    Field::Glyphs,
    Field::ConsoleFont,
    Field::FontFace,
    Field::FontSize,
    Field::CardSize,
    Field::AuraColour,
    Field::AuraStyle,
    Field::AuraSize,
    Field::SoundStyle,
    Field::Cues,
    Field::Music,
    Field::Volume,
    Field::Priority,
];

const POPULAR_FIELDS: [Field; 8] = [
    Field::Language,
    Field::LowVision,
    Field::Theme,
    Field::CardSize,
    Field::CardBack,
    Field::Volume,
    Field::Aura,
    Field::AutoFinish,
];

const GAMEPLAY_FIELDS: [Field; 7] = [
    Field::Language,
    Field::Aura,
    Field::TargetMarks,
    Field::GuidePanel,
    Field::MovesPanel,
    Field::AutoFinish,
    Field::QuickMove,
];

impl Tab {
    /// Every tab, left to right.
    pub const ALL: [Tab; 3] = [Tab::AudioVisual, Tab::Popular, Tab::Gameplay];

    /// Index of this tab in [`Tab::ALL`].
    pub const fn index(self) -> usize {
        match self {
            Tab::AudioVisual => 0,
            Tab::Popular => 1,
            Tab::Gameplay => 2,
        }
    }

    /// The tab to the right, wrapping around.
    pub const fn next(self) -> Self {
        match self {
            Tab::AudioVisual => Tab::Popular,
            Tab::Popular => Tab::Gameplay,
            Tab::Gameplay => Tab::AudioVisual,
        }
    }

    /// The tab to the left, wrapping around.
    pub const fn previous(self) -> Self {
        match self {
            Tab::AudioVisual => Tab::Gameplay,
            Tab::Popular => Tab::AudioVisual,
            Tab::Gameplay => Tab::Popular,
        }
    }

    /// Localization key of the tab label.
    pub const fn label_key(self) -> &'static str {
        match self {
            Tab::AudioVisual => "settings.tab.audio_visual",
            Tab::Popular => "settings.tab.popular",
            Tab::Gameplay => "settings.tab.gameplay",
        }
    }

    /// The rows shown while this tab is active.
    pub const fn fields(self) -> &'static [Field] {
        match self {
            Tab::AudioVisual => &AUDIO_VISUAL_FIELDS,
            Tab::Popular => &POPULAR_FIELDS,
            Tab::Gameplay => &GAMEPLAY_FIELDS,
        }
    }
}

/// Clickable rectangles recorded when drawing the tab bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct TabBar {
    pub previous: Rect,
    pub next: Rect,
    pub tabs: [(Tab, Rect); 3],
}

impl TabBar {
    /// The tab under `cell`, if any.
    pub(super) fn tab_at(&self, cell: (u16, u16)) -> Option<Tab> {
        self.tabs
            .iter()
            .find(|(_, rect)| contains(*rect, cell))
            .map(|(tab, _)| *tab)
    }

    /// The target tab when clicking one of the arrows at `cell`, if any.
    pub(super) fn arrow_target(&self, current: Tab, cell: (u16, u16)) -> Option<Tab> {
        if contains(self.previous, cell) {
            Some(current.previous())
        } else if contains(self.next, cell) {
            Some(current.next())
        } else {
            None
        }
    }
}

fn columns_of(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(u16::MAX)
}

/// Draws the centered tab bar (`◄  Audio & Visual  │ [Popular] │  Gameplay  ►`) in `area`.
pub(super) fn draw_tabs(frame: &mut Frame, area: Rect, active: Tab, ctx: &ViewCtx<'_>) -> TabBar {
    let widths = Tab::ALL.map(|tab| columns_of(&t(tab.label_key())).saturating_add(TAB_PADDING));
    let tabs_width: u16 = widths.iter().sum();
    let total = ARROW_WIDTH * 2 + tabs_width + SEPARATOR_WIDTH * 2;
    let mut x = area.x.saturating_add(area.width.saturating_sub(total) / 2);
    let plain = ctx.theme.base();
    let sep_style = ctx.theme.style(Token::TextMuted, Token::Background);

    let previous = Rect::new(x, area.y, ARROW_WIDTH, 1).intersection(area);
    let left_text = format!("{} ", ctx.glyphs.focus_right);
    frame.render_widget(Paragraph::new(Line::styled(left_text, plain)), previous);
    x = x.saturating_add(ARROW_WIDTH);

    let sep = format!(" {} ", ctx.glyphs.edge);
    let mut placed = [(Tab::AudioVisual, Rect::default()); 3];
    for (index, (tab, width)) in Tab::ALL.into_iter().zip(widths).enumerate() {
        if index > 0 {
            let sep_rect = Rect::new(x, area.y, SEPARATOR_WIDTH, 1).intersection(area);
            frame.render_widget(
                Paragraph::new(Line::styled(sep.clone(), sep_style)),
                sep_rect,
            );
            x = x.saturating_add(SEPARATOR_WIDTH);
        }
        let rect = Rect::new(x, area.y, width, 1).intersection(area);
        let name = t(tab.label_key());
        let (text, style) = if tab == active {
            (format!("[{name}]"), ctx.theme.active_tab())
        } else {
            (format!(" {name} "), plain)
        };
        frame.render_widget(Paragraph::new(Line::styled(text, style)), rect);
        placed[index] = (tab, rect);
        x = x.saturating_add(width);
    }

    let next = Rect::new(x, area.y, ARROW_WIDTH, 1).intersection(area);
    let right_text = format!(" {}", ctx.glyphs.focus_left);
    frame.render_widget(Paragraph::new(Line::styled(right_text, plain)), next);

    TabBar {
        previous,
        next,
        tabs: placed,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::screens::settings::options::FIELDS;

    #[test]
    fn popular_is_active_by_default_and_sits_in_the_middle() {
        assert_eq!(Tab::default(), Tab::Popular);
        assert_eq!(Tab::ALL, [Tab::AudioVisual, Tab::Popular, Tab::Gameplay]);
    }

    #[test]
    fn next_and_previous_wrap_around_all_three_tabs() {
        assert_eq!(Tab::AudioVisual.next(), Tab::Popular);
        assert_eq!(Tab::Popular.next(), Tab::Gameplay);
        assert_eq!(Tab::Gameplay.next(), Tab::AudioVisual);
        assert_eq!(Tab::AudioVisual.previous(), Tab::Gameplay);
        assert_eq!(Tab::Popular.previous(), Tab::AudioVisual);
        assert_eq!(Tab::Gameplay.previous(), Tab::Popular);
    }

    #[test]
    fn every_setting_belongs_to_exactly_one_canonical_tab_and_popular_aliases_them() {
        let mut canonical = Vec::new();
        canonical.extend_from_slice(Tab::AudioVisual.fields());
        canonical.extend_from_slice(Tab::Gameplay.fields());
        let unique: HashSet<Field> = canonical.iter().copied().collect();
        assert_eq!(canonical.len(), unique.len(), "duplicate canonical fields");
        assert_eq!(unique.len(), FIELDS.len());
        for field in FIELDS {
            assert!(
                unique.contains(&field),
                "{field:?} missing from canonical tabs"
            );
        }
        let popular: HashSet<Field> = Tab::Popular.fields().iter().copied().collect();
        assert_eq!(popular.len(), Tab::Popular.fields().len());
        assert!((7..=8).contains(&popular.len()));
        for field in Tab::Popular.fields() {
            assert!(
                unique.contains(field),
                "popular alias {field:?} not canonical"
            );
        }
    }
}
