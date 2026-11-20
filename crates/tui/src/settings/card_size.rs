//! The card-size setting and the pure step of the zoom keys. `Auto` takes the largest size that fits
//! the window; a named size is the largest the player wants, and a window too small for it falls
//! back to the largest that fits.

use serde::{Deserialize, Serialize};

use crate::layout::{CardSize, SIZES};

/// The card size the player chose.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardSizeSetting {
    /// The largest size that fits the window.
    #[default]
    Auto,
    /// 7 × 5.
    Small,
    /// 9 × 7.
    Medium,
    /// 11 × 9.
    Large,
    /// 13 × 11.
    Huge,
}

impl CardSizeSetting {
    /// Every value in the order the settings row cycles through them.
    pub const ALL: [CardSizeSetting; 5] = [
        CardSizeSetting::Auto,
        CardSizeSetting::Small,
        CardSizeSetting::Medium,
        CardSizeSetting::Large,
        CardSizeSetting::Huge,
    ];

    /// The size this value asks for; `None` for `Auto`.
    pub const fn size(self) -> Option<CardSize> {
        match self {
            Self::Auto => None,
            Self::Small => Some(SIZES[3]),
            Self::Medium => Some(SIZES[2]),
            Self::Large => Some(SIZES[1]),
            Self::Huge => Some(SIZES[0]),
        }
    }

    /// The value that asks for exactly `size`; `Auto` for a size the table does not know.
    pub fn of(size: CardSize) -> Self {
        Self::ALL
            .into_iter()
            .find(|setting| setting.size() == Some(size))
            .unwrap_or(Self::Auto)
    }

    /// Localization key of the name shown in the settings and in the game information.
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::Auto => "settings.card_size.auto",
            Self::Small => "settings.card_size.small",
            Self::Medium => "settings.card_size.medium",
            Self::Large => "settings.card_size.large",
            Self::Huge => "settings.card_size.huge",
        }
    }
}

/// Which way a zoom key moves the card size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zoom {
    In,
    Out,
}

/// What a zoom key does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zoomed {
    /// The setting becomes this value.
    To(CardSizeSetting),
    /// The board already shows the largest size that fits (zoom in) or the smallest (zoom out).
    AtLimit,
}

/// One step of a zoom key. `shown` is the size on the board now, `largest` the largest size that
/// fits the window. Zooming in goes one size up from what is shown, never past what fits; zooming
/// out goes one size down, never below the smallest.
pub fn step(shown: CardSize, largest: CardSize, zoom: Zoom) -> Zoomed {
    let Some(at) = SIZES.iter().position(|size| *size == shown) else {
        return Zoomed::AtLimit;
    };
    let target = match zoom {
        Zoom::In if shown.width < largest.width => at.checked_sub(1),
        Zoom::In => None,
        Zoom::Out => Some(at + 1),
    };
    match target.and_then(|index| SIZES.get(index)) {
        Some(size) => Zoomed::To(CardSizeSetting::of(*size)),
        None => Zoomed::AtLimit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HUGE: CardSize = SIZES[0];
    const LARGE: CardSize = SIZES[1];
    const MEDIUM: CardSize = SIZES[2];
    const SMALL: CardSize = SIZES[3];

    #[test]
    fn auto_is_the_default() {
        assert_eq!(CardSizeSetting::default(), CardSizeSetting::Auto);
        assert_eq!(CardSizeSetting::Auto.size(), None);
    }

    #[test]
    fn every_named_value_maps_to_its_size_and_back() {
        let sizes = [
            (CardSizeSetting::Small, (7, 5)),
            (CardSizeSetting::Medium, (9, 7)),
            (CardSizeSetting::Large, (11, 9)),
            (CardSizeSetting::Huge, (13, 11)),
        ];
        for (setting, (width, height)) in sizes {
            let size = CardSize::new(width, height);
            assert_eq!(setting.size(), Some(size));
            assert_eq!(CardSizeSetting::of(size), setting);
        }
    }

    #[test]
    fn zooming_in_steps_up_until_the_largest_that_fits() {
        assert_eq!(
            step(SMALL, LARGE, Zoom::In),
            Zoomed::To(CardSizeSetting::Medium)
        );
        assert_eq!(
            step(MEDIUM, LARGE, Zoom::In),
            Zoomed::To(CardSizeSetting::Large)
        );
        assert_eq!(step(LARGE, LARGE, Zoom::In), Zoomed::AtLimit);
        assert_eq!(step(HUGE, HUGE, Zoom::In), Zoomed::AtLimit);
    }

    #[test]
    fn zooming_out_steps_down_until_the_smallest() {
        assert_eq!(
            step(HUGE, HUGE, Zoom::Out),
            Zoomed::To(CardSizeSetting::Large)
        );
        assert_eq!(
            step(MEDIUM, HUGE, Zoom::Out),
            Zoomed::To(CardSizeSetting::Small)
        );
        assert_eq!(step(SMALL, HUGE, Zoom::Out), Zoomed::AtLimit);
    }

    #[test]
    fn the_setting_round_trips_as_snake_case_and_old_values_load_as_auto() {
        let text = toml::to_string(&crate::settings::Settings {
            card_size: CardSizeSetting::Large,
            ..crate::settings::Settings::default()
        })
        .unwrap_or_default();
        assert!(text.contains("card_size = \"large\""));
        let back = toml::from_str::<crate::settings::Settings>(&text);
        assert!(matches!(back, Ok(ref s) if s.card_size == CardSizeSetting::Large));
        let old = toml::from_str::<crate::settings::Settings>("volume = 10");
        assert!(matches!(old, Ok(ref s) if s.card_size == CardSizeSetting::Auto));
    }
}
