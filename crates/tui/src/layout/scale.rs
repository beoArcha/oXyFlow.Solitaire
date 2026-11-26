//! The interface size scale: one table per step (`S`, `M`, `L`, `XL`) that coordinates card caps,
//! spacing, fan steps, side panel widths, screen classes and overlay window sizes.

use serde::{Deserialize, Serialize};

use super::scale_table::SCALES;
use super::size::CardSize;

/// The four interface scale steps selectable by the user. `M` reproduces the built-in layout;
/// steps that do not fit the current terminal fall back to the largest step that fits.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum InterfaceSize {
    S,
    #[default]
    M,
    L,
    XL,
}

impl InterfaceSize {
    /// Every scale step from smallest to largest.
    pub const ALL: [Self; 4] = [Self::S, Self::M, Self::L, Self::XL];

    /// Steps ordered from largest to smallest for fallback matching.
    const DESCENDING: [Self; 4] = [Self::XL, Self::L, Self::M, Self::S];

    /// The step table entry for this interface size.
    pub const fn scale(self) -> &'static Scale {
        match self {
            Self::S => &SCALES[0],
            Self::M => &SCALES[1],
            Self::L => &SCALES[2],
            Self::XL => &SCALES[3],
        }
    }

    /// Every step from `self` down to [`Self::S`], largest first.
    pub fn downward(self) -> impl Iterator<Item = Self> {
        let start = Self::DESCENDING
            .iter()
            .position(|step| *step == self)
            .unwrap_or(Self::DESCENDING.len() - 1);
        Self::DESCENDING[start..].iter().copied()
    }

    /// Resolves the largest step no larger than `self` for which `fits` holds, else [`Self::S`].
    pub fn fit(self, fits: impl Fn(&Scale) -> bool) -> Self {
        self.downward()
            .find(|step| fits(step.scale()))
            .unwrap_or(Self::S)
    }

    /// Localization key for the step name (`S`, `M`, `L`, `XL`).
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::S => "settings.interface_size.s",
            Self::M => "settings.interface_size.m",
            Self::L => "settings.interface_size.l",
            Self::XL => "settings.interface_size.xl",
        }
    }
}

/// Complete geometry parameters for one interface scale step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scale {
    pub size: InterfaceSize,
    pub card_cap: CardSize,
    pub gaps: GapsScale,
    pub fan: FanScale,
    pub chrome: ChromeScale,
    pub dialog: DialogScale,
    pub help: HelpScale,
    pub how_to_play: HowToPlayScale,
    pub guide_window: GuideWindowScale,
    pub moves_window: MovesWindowScale,
    pub quick_options: QuickOptionsScale,
    pub dock: DockScale,
}

/// Horizontal and group gaps between piles on the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GapsScale {
    pub min_gap: u16,
    pub max_gap: u16,
    pub group_gap: u16,
    pub tripeaks_base_gap: u16,
}

/// Tableau and waste fan steps relative to a card size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FanScale {
    pub up_delta: i16,
    pub down_delta: i16,
    pub waste_delta: i16,
    pub small_row_gap: u16,
    pub tall_row_gap: u16,
}

impl FanScale {
    /// Empty rows between the top row and the tableau for cards of `card`.
    pub const fn row_gap(self, card: CardSize) -> u16 {
        if card.height >= 9 {
            self.tall_row_gap
        } else {
            self.small_row_gap
        }
    }

    /// Natural face-up card step for `card` at this scale.
    pub const fn up_step(self, card: CardSize) -> u16 {
        let half = card.height / 2;
        let base = if half < 2 { 2 } else { half };
        let max = if card.height > 2 { card.height - 1 } else { 2 };
        shift_clamped(base, self.up_delta, 2, max)
    }

    /// Natural face-down card step for `card` at this scale.
    pub const fn down_step(self, card: CardSize) -> u16 {
        let half_h = card.height / 2;
        let base_up = if half_h < 2 { 2 } else { half_h };
        let half = base_up / 2;
        let base = if half < 1 { 1 } else { half };
        let max = if card.height > 2 { card.height - 2 } else { 1 };
        shift_clamped(base, self.down_delta, 1, max)
    }

    /// Pyramid row overlap step for `card` at this scale.
    pub const fn pyramid_step(self, card: CardSize) -> u16 {
        self.up_step(card)
    }

    /// Horizontal step between fanned waste cards for `card` at this scale.
    pub const fn waste_step(self, card: CardSize) -> u16 {
        let base = card.width.div_ceil(2);
        let max = if card.width > 4 { card.width - 1 } else { 4 };
        shift_clamped(base, self.waste_delta, 4, max)
    }

    /// Wider face-up step used in low-vision mode when room permits.
    pub const fn wide_up_step(self, card: CardSize) -> u16 {
        let max = card.height.saturating_sub(1);
        let next = self.up_step(card) + 1;
        if next < max { next } else { max }
    }

    /// Wider face-down step used in low-vision mode when room permits.
    pub const fn wide_down_step(self, card: CardSize) -> u16 {
        let max = card.height.saturating_sub(1);
        let next = self.down_step(card) + 1;
        if next < max { next } else { max }
    }

    /// Wider pyramid row step used in low-vision mode when height permits.
    pub const fn wide_pyramid_step(self, card: CardSize) -> u16 {
        self.wide_up_step(card)
    }

    /// Wider waste fan step used in low-vision mode when width permits.
    pub const fn wide_waste_step(self, card: CardSize) -> u16 {
        let max = card.width.saturating_sub(1);
        let next = self.waste_step(card) + 1;
        if next < max { next } else { max }
    }
}

const fn shift_clamped(base: u16, delta: i16, min: u16, max: u16) -> u16 {
    let shifted = (base as i32) + (delta as i32);
    let upper = if max < min { min } else { max };
    if shifted < (min as i32) {
        min
    } else if shifted > (upper as i32) {
        upper
    } else {
        shifted as u16
    }
}

/// Side panel widths, class thresholds and moves panel row counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChromeScale {
    pub standard_min_width: u16,
    pub wide_min_width: u16,
    pub guide_width: u16,
    pub guide_width_wide: u16,
    pub moves_rows: u16,
    pub moves_rows_tall: u16,
    pub moves_reasons_from: u16,
}

/// Shared modal dialog box constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DialogScale {
    pub max_text_columns: u16,
    pub min_columns: u16,
    pub chrome_columns: u16,
    pub button_gap: u16,
    pub marker_columns: u16,
}

/// Help window geometry parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HelpScale {
    pub min_width: u16,
    pub gap: u16,
    pub key_padding: usize,
    pub share_percent: u16,
    pub min_frame_rows: u16,
    pub max_frame_rows: u16,
}

/// Rules ("How to play") tab geometry and text wrapping width.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HowToPlayScale {
    pub text_width: usize,
    pub arrow_columns: u16,
    pub tab_padding: u16,
    pub tab_gap: u16,
}

/// Guide popup window dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuideWindowScale {
    pub box_width: u16,
    pub text_width: usize,
    pub text_rows: usize,
}

/// Possible-moves popup window dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MovesWindowScale {
    pub box_width: u16,
}

/// Quick-options popup dialog dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuickOptionsScale {
    pub box_width: u16,
}

/// Docked help window width limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DockScale {
    pub min_width: u16,
    pub max_width: u16,
}
