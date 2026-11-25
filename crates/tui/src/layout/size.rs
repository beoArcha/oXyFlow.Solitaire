//! Card sizes and the spacing rules that grow with the window.

/// The size of one card in terminal cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardSize {
    pub width: u16,
    pub height: u16,
}

/// Every card size, largest first. A board uses the largest one that fits the window.
pub const SIZES: [CardSize; 4] = [
    CardSize::new(13, 11),
    CardSize::new(11, 9),
    CardSize::new(9, 7),
    CardSize::new(7, 5),
];

/// Rows a column of face-up cards may need below its first card without losing a rank. The size
/// of the board is chosen so that a column of this length fits (a king-to-ace run plus a margin).
pub const FAN_BUDGET: u16 = 14;

const MAX_GAP: u16 = 3;

impl CardSize {
    pub const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    /// The smallest size, used when nothing fits.
    pub const fn smallest() -> Self {
        SIZES[SIZES.len() - 1]
    }

    /// The largest size for which `fits` holds, else the smallest.
    pub fn largest_fit(fits: impl Fn(CardSize) -> bool) -> Self {
        Self::largest_fit_within(None, fits)
    }

    /// The largest size for which `fits` holds that is no larger than `limit` (any size when
    /// `limit` is `None`), else the smallest. A limit that does not fit falls back to the largest
    /// smaller size that does.
    pub fn largest_fit_within(limit: Option<CardSize>, fits: impl Fn(CardSize) -> bool) -> Self {
        SIZES
            .into_iter()
            .filter(|size| limit.is_none_or(|limit| size.width <= limit.width))
            .find(|size| fits(*size))
            .unwrap_or_else(Self::smallest)
    }

    /// Empty rows between the top row and the tableau.
    pub const fn row_gap(self) -> u16 {
        if self.height >= 9 { 2 } else { 1 }
    }

    /// Natural distance between two face-up cards of a column.
    pub const fn up_step(self) -> u16 {
        let half = self.height / 2;
        if half < 2 { 2 } else { half }
    }

    /// Natural distance between two face-down cards of a column.
    pub const fn down_step(self) -> u16 {
        let half = self.up_step() / 2;
        if half < 1 { 1 } else { half }
    }

    /// Distance between two overlapping rows of the pyramid.
    pub const fn pyramid_step(self) -> u16 {
        self.up_step()
    }

    /// Horizontal distance between cards of a fanned waste. Wide enough to keep the rank and the
    /// suit of the covered cards readable.
    pub const fn waste_step(self) -> u16 {
        self.width.div_ceil(2)
    }

    /// Wider face-up step used in low-vision mode when room permits.
    pub const fn wide_up_step(self) -> u16 {
        let max = self.height.saturating_sub(1);
        let next = self.up_step() + 1;
        if next < max { next } else { max }
    }

    /// Wider face-down step used in low-vision mode when room permits.
    pub const fn wide_down_step(self) -> u16 {
        self.down_step() + 1
    }

    /// Wider pyramid row step used in low-vision mode when height permits.
    pub const fn wide_pyramid_step(self) -> u16 {
        self.wide_up_step()
    }

    /// Wider waste fan step used in low-vision mode when width permits.
    pub const fn wide_waste_step(self) -> u16 {
        let max = self.width.saturating_sub(1);
        let next = self.waste_step() + 1;
        if next < max { next } else { max }
    }
}

/// The gap between `count` cards in a row of width `available` and the left margin that centers
/// the row. The gap is at least 1 and grows with the window up to 3.
pub fn spacing(available: u16, count: u16, card_width: u16) -> (u16, u16) {
    let gap = match count {
        0 | 1 => 1,
        _ => (available.saturating_sub(count * card_width) / (count - 1)).clamp(1, MAX_GAP),
    };
    let content = (count * card_width + count.saturating_sub(1) * gap).min(available);
    (gap, (available - content) / 2)
}
