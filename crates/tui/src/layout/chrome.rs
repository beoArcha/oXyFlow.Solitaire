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

/// Rows of the possible moves panel when it shares the side column with the guide panel: every
/// entry of the list plus the frame.
pub const MOVES_ROWS: u16 = 12;
/// Height of the side column from which the shared moves panel is tall enough to show a reason
/// under each inactive entry.
pub const MOVES_ROWS_WITH_REASONS_FROM: u16 = 56;
/// Rows of the moves panel when the screen is tall enough for the reasons.
pub const MOVES_ROWS_TALL: u16 = 22;

/// The parts of the board screen. `header` is the game-information row, present exactly when
/// there is no guide panel; `guide` is the framed guide panel and `moves` the framed possible
/// moves panel, both in the side column (alone, either takes the full height; together the guide
/// is above); `now` and `possible` are the two rows of the controls strip, as wide as the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chrome {
    pub class: Class,
    pub header: Option<Rect>,
    pub body: Rect,
    pub guide: Option<Rect>,
    pub moves: Option<Rect>,
    pub now: Rect,
    pub possible: Rect,
}

/// Splits the screen without a side panel: a header row, the board and the controls strip. `body`
/// is the area to pass to `compute`.
pub fn chrome(area: Rect) -> Chrome {
    chrome_with(area, false)
}

/// Like [`chrome`], and with the guide panel at the right when `guide` is set and the screen is
/// Standard or Wide.
pub fn chrome_with(area: Rect, guide: bool) -> Chrome {
    chrome_panels(area, guide, false)
}

/// Splits the side column between the panels that are on: one takes it all, two share it with
/// the guide above and the moves panel below.
fn stack(column: Rect, guide: bool, moves: bool) -> (Option<Rect>, Option<Rect>) {
    match (guide, moves) {
        (true, true) => {
            let wanted = if column.height >= MOVES_ROWS_WITH_REASONS_FROM {
                MOVES_ROWS_TALL
            } else {
                MOVES_ROWS
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
pub fn chrome_panels(area: Rect, guide: bool, moves: bool) -> Chrome {
    let class = Class::of(area);
    let width = class.panel_width();
    let (main, column) = if (guide || moves) && width > 0 {
        let [main, side] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(width)]).areas(area);
        (main, Some(side))
    } else {
        (area, None)
    };
    let (guide, moves) = column.map_or((None, None), |column| stack(column, guide, moves));
    let [header, body, now, possible] = Layout::vertical([
        Constraint::Length(u16::from(guide.is_none())),
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
        moves,
        now,
        possible,
    }
}
