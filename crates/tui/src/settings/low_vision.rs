//! Low-vision mode bundle: turns on a readable preset (High Contrast, largest card size that
//! fits, thick aura, and at least 24 px console font in an owned classic Windows console) while
//! remembering the previous values so turning it off restores them exactly.

use serde::{Deserialize, Serialize};

use super::{AuraSize, CardSizeSetting, Settings};
use crate::theme::ThemeId;

/// Minimum classic Windows console font height (in pixels) asked when low-vision mode turns on.
pub const LOW_VISION_MIN_FONT_SIZE: u8 = 24;

/// Snapshot of the settings overridden by turning low-vision mode on, kept so turning it off
/// restores the player's previous choices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LowVisionBackup {
    pub theme: ThemeId,
    pub card_size: CardSizeSetting,
    pub aura_size: AuraSize,
    pub console_font_size: u8,
}

impl Settings {
    /// Returns a copy with low-vision mode set to `on`. Turning it on saves the current bundle
    /// fields and applies the low-vision preset (raising `console_font_size` to at least 24 px
    /// only when `classic_console` is true); turning it off restores the saved values.
    #[must_use]
    pub fn with_low_vision(&self, on: bool, classic_console: bool) -> Self {
        if on == self.low_vision {
            return self.clone();
        }
        if on {
            let backup = LowVisionBackup {
                theme: self.theme,
                card_size: self.card_size,
                aura_size: self.aura_size,
                console_font_size: self.console_font_size,
            };
            let console_font_size = if classic_console {
                self.console_font_size.max(LOW_VISION_MIN_FONT_SIZE)
            } else {
                self.console_font_size
            };
            return Self {
                low_vision: true,
                low_vision_saved: Some(backup),
                theme: ThemeId::HighContrast,
                card_size: CardSizeSetting::Auto,
                aura_size: AuraSize::Thick,
                console_font_size,
                ..self.clone()
            };
        }
        let Some(backup) = self.low_vision_saved else {
            return Self {
                low_vision: false,
                ..self.clone()
            };
        };
        Self {
            low_vision: false,
            low_vision_saved: None,
            theme: backup.theme,
            card_size: backup.card_size,
            aura_size: backup.aura_size,
            console_font_size: backup.console_font_size,
            ..self.clone()
        }
    }
}
