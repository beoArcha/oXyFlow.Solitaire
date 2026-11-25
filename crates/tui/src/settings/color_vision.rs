//! Color vision deficiency modes for theme token remapping and four-color deck adaptation.

use serde::{Deserialize, Serialize};

/// Color vision mode: normal, three dichromacy modes (red-blind, green-blind, blue-blind), or
/// monochrome.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorVision {
    #[default]
    Normal,
    Protanopia,
    Deuteranopia,
    Tritanopia,
    Monochrome,
}

impl ColorVision {
    /// Every mode, in the order of the settings screen.
    pub const ALL: [ColorVision; 5] = [
        ColorVision::Normal,
        ColorVision::Protanopia,
        ColorVision::Deuteranopia,
        ColorVision::Tritanopia,
        ColorVision::Monochrome,
    ];

    /// Localization key of the mode name.
    pub const fn label_key(self) -> &'static str {
        match self {
            ColorVision::Normal => "settings.color_vision.normal",
            ColorVision::Protanopia => "settings.color_vision.protanopia",
            ColorVision::Deuteranopia => "settings.color_vision.deuteranopia",
            ColorVision::Tritanopia => "settings.color_vision.tritanopia",
            ColorVision::Monochrome => "settings.color_vision.monochrome",
        }
    }
}
