//! The possible moves panel: a framed list of every board action as a button. An action that can
//! be run now is active (bold, `•`); one that cannot stays listed, dimmed and marked `×`, with its
//! reason on the row below when the panel is tall enough. The same entries fill the moves window of
//! a Compact screen. The list is built by the board from the legal moves; nothing is decided here.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::Widget;

use super::guide::{FRAME_ROWS, SIDE, draw_titled_frame};
use super::text::clip;
use crate::action::Action;
use crate::i18n::tf;
use crate::input::primary_key;
use crate::mouse::contains;
use crate::screens::ViewCtx;
use crate::screens::focus::marker;
use crate::theme::{Theme, Token};

/// One board action in the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveEntry {
    pub action: Action,
    pub label: String,
    pub key: String,
    pub active: bool,
    /// Why the entry is inactive; `None` for an active entry.
    pub reason: Option<String>,
}

/// The active entries of the last draw: the rows a click runs.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PanelMap {
    hits: Vec<(Rect, Action)>,
}

impl PanelMap {
    /// The action of the active entry at `cell`; inactive entries and the frame give `None`.
    pub fn action_at(&self, cell: (u16, u16)) -> Option<Action> {
        self.hits
            .iter()
            .find(|(rect, _)| contains(*rect, cell))
            .map(|(_, action)| *action)
    }
}

/// The row of the panel that carries the title, its top border; a click on it hides the panel.
pub fn title_row(area: Rect) -> Rect {
    Rect::new(area.x, area.y, area.width, area.height.min(1))
}

/// The text of the entry row: marker, sign, label, and the key at the right end. The label is
/// clipped so that the key always stays.
pub fn row_text(entry: &MoveEntry, focused: bool, width: usize, ctx: &ViewCtx<'_>) -> String {
    let sign = if entry.active {
        ctx.glyphs.entry_on
    } else {
        ctx.glyphs.entry_off
    };
    let head = format!("{}{sign} ", marker(ctx, focused));
    let key_width = entry.key.chars().count();
    let room = width.saturating_sub(head.chars().count() + key_width + 1);
    let label = clip(&entry.label, room);
    let used = head.chars().count() + label.chars().count() + key_width;
    let gap = " ".repeat(width.saturating_sub(used).max(1));
    format!("{head}{label}{gap}{}", entry.key)
}

/// The style of an entry that cannot run now and of its reason. It is the muted text on the
/// surface in plain weight: the dim attribute made the text too faint on the dark background, so
/// the entry carries its state in the `×` mark and the reason, not in faintness.
pub fn unavailable_style(theme: &Theme) -> Style {
    theme.style(Token::TextMuted, Token::Surface)
}

/// The style of an entry row: bold text when it can run, the unavailable style otherwise, reversed
/// while it has the focus or the pointer.
pub fn entry_style(entry: &MoveEntry, focused: bool, hovered: bool, ctx: &ViewCtx<'_>) -> Style {
    let style = if entry.active {
        ctx.theme
            .style(Token::Text, Token::Surface)
            .add_modifier(Modifier::BOLD)
    } else {
        unavailable_style(ctx.theme)
    };
    if focused || hovered {
        style.add_modifier(Modifier::REVERSED)
    } else {
        style
    }
}

/// The entries with the available ones first; the order inside each group is kept.
pub fn available_first(entries: &[MoveEntry]) -> Vec<MoveEntry> {
    let mut ordered = entries.to_vec();
    ordered.sort_by_key(|entry| !entry.active);
    ordered
}

/// Whether every inactive entry can show its reason in `rows` rows.
pub fn reasons_fit(entries: &[MoveEntry], rows: usize) -> bool {
    let reasons = entries.iter().filter(|entry| !entry.active).count();
    entries.len() + reasons <= rows
}

fn draw_row(buf: &mut Buffer, row: Rect, text: &str, style: Style) {
    Span::styled(text.to_owned(), style).render(row, buf);
}

/// Draws the panel into `area`: the frame with its title, then one row per entry (and one per
/// reason when they fit). `focus` is the entry that has the focus. Returns the active rows.
pub fn draw(
    buf: &mut Buffer,
    area: Rect,
    entries: &[MoveEntry],
    focus: Option<usize>,
    ctx: &ViewCtx<'_>,
) -> PanelMap {
    let mut map = PanelMap::default();
    if area.width <= 2 * SIDE || area.height <= FRAME_ROWS {
        return map;
    }
    let hide = primary_key(Action::Moves).unwrap_or_default();
    draw_titled_frame(buf, area, &tf("board.moves.title", &[("key", &hide)]), ctx);
    let inner = Rect::new(
        area.x + SIDE,
        area.y + 1,
        area.width - 2 * SIDE,
        area.height - FRAME_ROWS,
    );
    let reasons = reasons_fit(entries, usize::from(inner.height));
    let width = usize::from(inner.width);
    let mut y = inner.y;
    for (index, entry) in entries.iter().enumerate() {
        if y >= inner.bottom() {
            break;
        }
        let row = Rect::new(inner.x, y, inner.width, 1);
        let focused = focus == Some(index);
        let hovered = entry.active && ctx.pointer.is_some_and(|cell| contains(row, cell));
        let text = row_text(entry, focused, width, ctx);
        draw_row(buf, row, &text, entry_style(entry, focused, hovered, ctx));
        if entry.active {
            map.hits.push((row, entry.action));
        }
        y += 1;
        let reason = entry
            .reason
            .as_deref()
            .filter(|_| reasons && y < inner.bottom());
        if let Some(reason) = reason {
            let indent = marker(ctx, false).chars().count() + 2;
            let text = format!("{}{}", " ".repeat(indent), clip(reason, width - indent));
            draw_row(
                buf,
                Rect::new(inner.x, y, inner.width, 1),
                &text,
                unavailable_style(ctx.theme),
            );
            y += 1;
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use ratatui::style::{Color, Modifier, Style};

    use super::{MoveEntry, available_first, unavailable_style};
    use crate::action::Action;
    use crate::theme::{Theme, ThemeId, Token};

    fn entry(action: Action, active: bool) -> MoveEntry {
        MoveEntry {
            action,
            label: String::new(),
            key: String::new(),
            active,
            reason: (!active).then(String::new),
        }
    }

    fn rgb_of(color: Option<Color>) -> [u8; 3] {
        match color {
            Some(Color::Rgb(r, g, b)) => [r, g, b],
            other => panic!("expected a true-colour value, found {other:?}"),
        }
    }

    #[test]
    fn the_unavailable_style_reaches_the_contrast_target_in_every_theme() {
        for id in ThemeId::ALL {
            let theme = Theme::new(id, crate::caps::ColorDepth::TrueColor);
            let style = unavailable_style(&theme);
            let ratio = crate::theme::contrast::contrast_ratio(rgb_of(style.fg), rgb_of(style.bg));
            assert!(
                ratio >= id.contrast_target(),
                "{id:?}: unavailable text is {ratio}"
            );
        }
    }

    #[test]
    fn the_unavailable_style_is_not_dimmed_in_any_theme() {
        for id in ThemeId::ALL {
            let theme = Theme::new(id, crate::caps::ColorDepth::TrueColor);
            let style: Style = unavailable_style(&theme);
            assert!(
                !style.add_modifier.contains(Modifier::DIM),
                "{id:?}: the dim attribute is set"
            );
        }
    }

    #[test]
    fn the_unavailable_style_is_the_muted_token_on_the_surface() {
        let theme = Theme::new(ThemeId::Dark, crate::caps::ColorDepth::TrueColor);
        assert_eq!(
            unavailable_style(&theme),
            theme.style(Token::TextMuted, Token::Surface)
        );
    }

    #[test]
    fn available_entries_come_first_and_keep_their_order() {
        let entries = [
            entry(Action::Stock, false),
            entry(Action::Select, true),
            entry(Action::Hint, false),
            entry(Action::Undo, true),
            entry(Action::Redo, false),
        ];
        let ordered = available_first(&entries);
        let actions: Vec<Action> = ordered.iter().map(|entry| entry.action).collect();
        assert_eq!(
            actions,
            [
                Action::Select,
                Action::Undo,
                Action::Stock,
                Action::Hint,
                Action::Redo
            ]
        );
    }

    #[test]
    fn ordering_leaves_the_input_untouched() {
        let entries = [entry(Action::Stock, false), entry(Action::Select, true)];
        let _ordered = available_first(&entries);
        assert_eq!(entries[0].action, Action::Stock);
    }
}
