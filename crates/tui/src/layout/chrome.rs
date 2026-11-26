//! The parts of the board screen and the size class that decides them.
//!
//! Compact screens (fewer than 130 columns) show the game information in a header row. Standard
//! (130 to 179 columns) and Wide (180 and more) screens show it in the top box of a framed side
//! panel instead, so the header row disappears, whatever the height: the terminal is never
//! smaller than the board's minimum, and the board keeps its 100 columns beside the panel. Under
//! the board there are always the two rows of the controls strip.

use ratatui::layout::{Constraint, Layout, Margin, Rect};

use super::scale::{ChromeScale, InterfaceSize};

/// Columns the board keeps; the side panel never shrinks it below this.
pub const BOARD_MIN_WIDTH: u16 = 100;
/// Columns from which the screen is Standard at the default interface scale.
pub const STANDARD_MIN_WIDTH: u16 = InterfaceSize::M.scale().chrome.standard_min_width;
/// Columns from which the screen is Wide at the default interface scale.
pub const WIDE_MIN_WIDTH: u16 = InterfaceSize::M.scale().chrome.wide_min_width;
/// Columns of the side panel of a Standard screen at the default interface scale, frame included.
pub const GUIDE_WIDTH: u16 = InterfaceSize::M.scale().chrome.guide_width;
/// Columns of the side panel of a Wide screen at the default interface scale, frame included.
pub const GUIDE_WIDTH_WIDE: u16 = InterfaceSize::M.scale().chrome.guide_width_wide;

/// The size class of a terminal, decided from its size at every draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    Compact,
    Standard,
    Wide,
}

impl Class {
    /// The class of a screen of `area`'s size at the default interface scale.
    pub const fn of(area: Rect) -> Self {
        Self::of_for(area, &InterfaceSize::M.scale().chrome)
    }

    /// The class of a screen of `area`'s size using `scale`'s width thresholds.
    pub const fn of_for(area: Rect, scale: &ChromeScale) -> Self {
        if area.width < scale.standard_min_width {
            Self::Compact
        } else if area.width < scale.wide_min_width {
            Self::Standard
        } else {
            Self::Wide
        }
    }

    /// Columns of the side panel in this class at the default interface scale; none in Compact.
    pub const fn panel_width(self) -> u16 {
        self.panel_width_for(&InterfaceSize::M.scale().chrome)
    }

    /// Columns of the side panel in this class for `scale`; none in Compact.
    pub const fn panel_width_for(self, scale: &ChromeScale) -> u16 {
        match self {
            Self::Compact => 0,
            Self::Standard => scale.guide_width,
            Self::Wide => scale.guide_width_wide,
        }
    }
}

/// Cells the field frame takes on each side of the board when the theme draws one.
pub const FRAME_INSET: u16 = 1;

/// Rows of the possible moves panel when it shares the side column with the guide panel at the
/// default interface scale.
pub const MOVES_ROWS: u16 = InterfaceSize::M.scale().chrome.moves_rows;
/// Rows of the moves panel when the screen is tall enough for the reasons at the default scale.
pub const MOVES_ROWS_TALL: u16 = InterfaceSize::M.scale().chrome.moves_rows_tall;

/// The parts of the board screen. `header` is the game-information row, present exactly when
/// there is no guide panel; `guide` is the framed guide panel and `moves` the framed possible
/// moves panel, both in the side column (alone, either takes the full height; together the guide
/// is above); `field` is the playing field beside the side column, where the field frame is drawn
/// when the theme has one; `body` is the area to pass to `compute`: `field` itself without a
/// frame, `field` inset by [`FRAME_INSET`] with one; `now` and `possible` are the two rows of the
/// controls strip, as wide as the field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chrome {
    pub class: Class,
    pub header: Option<Rect>,
    pub field: Rect,
    pub body: Rect,
    pub guide: Option<Rect>,
    pub moves: Option<Rect>,
    pub now: Rect,
    pub possible: Rect,
}

/// Splits the screen without a side panel and without a field frame: a header row, the field and
/// the controls strip. `body` is the area to pass to `compute`.
pub fn chrome(area: Rect) -> Chrome {
    chrome_with(area, false, false)
}

/// Like [`chrome`], and with the guide panel at the right when `guide` is set and the screen is
/// Standard or Wide. `frame` insets `body` by [`FRAME_INSET`] for the field frame of the theme
/// (`Theme::field_edge`); without it `body` is `field`.
pub fn chrome_with(area: Rect, guide: bool, frame: bool) -> Chrome {
    chrome_panels(area, guide, false, frame)
}

/// Splits the side column between the panels that are on: one takes it all, two share it with
/// the guide above and the moves panel below.
fn stack(
    column: Rect,
    guide: bool,
    moves: bool,
    scale: &ChromeScale,
) -> (Option<Rect>, Option<Rect>) {
    match (guide, moves) {
        (true, true) => {
            let wanted = if column.height >= scale.moves_reasons_from {
                scale.moves_rows_tall
            } else {
                scale.moves_rows
            };
            let [top, bottom] = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(wanted.min(column.height / 2)),
            ])
            .areas(column);
            (Some(top), Some(bottom))
        }
        (true, false) => (Some(column), None),
        (false, true) => (None, Some(column)),
        (false, false) => (None, None),
    }
}

/// Like [`chrome_with`], with the possible moves panel too when `moves` is set.
pub fn chrome_panels(area: Rect, guide: bool, moves: bool, frame: bool) -> Chrome {
    chrome_panels_for(area, guide, moves, frame, InterfaceSize::default())
}

/// Like [`chrome_panels`], scaled to `interface_size`.
pub fn chrome_panels_for(
    area: Rect,
    guide: bool,
    moves: bool,
    frame: bool,
    interface_size: InterfaceSize,
) -> Chrome {
    let chrome = &interface_size.scale().chrome;
    let class = Class::of_for(area, chrome);
    let width = class.panel_width_for(chrome);
    let (main, column) = if (guide || moves) && width > 0 {
        let [main, side] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(width)]).areas(area);
        (main, Some(side))
    } else {
        (area, None)
    };
    let (guide, moves) = column.map_or((None, None), |column| stack(column, guide, moves, chrome));
    let [header, field, now, possible] = Layout::vertical([
        Constraint::Length(u16::from(guide.is_none())),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(main);
    let inset = if frame { FRAME_INSET } else { 0 };
    Chrome {
        class,
        header: guide.is_none().then_some(header),
        field,
        body: field.inner(Margin::new(inset, inset)),
        guide,
        moves,
        now,
        possible,
    }
}
