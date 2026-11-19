//! The parts of the board screen and the size class that decides them.
//!
//! Compact screens (fewer than 130 columns) show the game information in a header row. Standard
//! (130 to 179 columns) and Wide (180 and more) screens show it in the top box of a framed side
//! panel instead, so the header row disappears, whatever the height: the terminal is never
//! smaller than the board's minimum, and the board keeps its 100 columns beside the panel. Under
//! the board there are always the two rows of the controls strip.

use ratatui::layout::{Constraint, Layout, Rect};

/// Columns the board keeps; the side panel never shrinks it below this.
pub const BOARD_MIN_WIDTH: u16 = 100;
/// Columns from which the screen is Standard.
pub const STANDARD_MIN_WIDTH: u16 = 130;
/// Columns from which the screen is Wide.
pub const WIDE_MIN_WIDTH: u16 = 180;
/// Columns of the side panel of a Standard screen, its frame included.
pub const GUIDE_WIDTH: u16 = 30;
/// Columns of the side panel of a Wide screen, its frame included.
pub const GUIDE_WIDTH_WIDE: u16 = 40;

/// The size class of a terminal, decided from its size at every draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    Compact,
    Standard,
    Wide,
}

impl Class {
    /// The class of a screen of `area`'s size.
    pub const fn of(area: Rect) -> Self {
        if area.width < STANDARD_MIN_WIDTH {
            Self::Compact
        } else if area.width < WIDE_MIN_WIDTH {
            Self::Standard
        } else {
            Self::Wide
        }
    }

    /// Columns of the side panel in this class; none in a Compact screen.
    pub const fn panel_width(self) -> u16 {
        match self {
            Self::Compact => 0,
            Self::Standard => GUIDE_WIDTH,
            Self::Wide => GUIDE_WIDTH_WIDE,
        }
    }
}

/// The parts of the board screen. `header` is the game-information row, present exactly when
/// there is no side panel; `guide` is the framed side panel, full height; `now` and `possible` are
/// the two rows of the controls strip, as wide as the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chrome {
    pub class: Class,
    pub header: Option<Rect>,
    pub body: Rect,
    pub guide: Option<Rect>,
    pub now: Rect,
    pub possible: Rect,
}

/// Splits the screen without a side panel: a header row, the board and the controls strip. `body`
/// is the area to pass to `compute`.
pub fn chrome(area: Rect) -> Chrome {
    chrome_with(area, false)
}

/// Like [`chrome`], and with the side panel at the right when `guide` is set and the screen is
/// Standard or Wide.
pub fn chrome_with(area: Rect, guide: bool) -> Chrome {
    let class = Class::of(area);
    let width = class.panel_width();
    let (main, guide) = if guide && width > 0 {
        let [main, side] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(width)]).areas(area);
        (main, Some(side))
    } else {
        (area, None)
    };
    let header_rows = u16::from(guide.is_none());
    let [header, body, now, possible] = Layout::vertical([
        Constraint::Length(header_rows),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(main);
    Chrome {
        class,
        header: guide.is_none().then_some(header),
        body,
        guide,
        now,
        possible,
    }
}
