//! Player settings. Stored through `Storage` as one value; unknown or missing fields fall back to
//! their defaults so an older save keeps loading after the struct grows.

use serde::{Deserialize, Serialize};

mod aids;
mod card_back;
mod card_face;
mod card_size;
mod color_vision;
mod console_face;
mod dock;
mod full_screen;
mod interface;
mod priority;
mod table;
pub use aids::{Aid, Aids};
pub use card_back::CardBack;
pub use card_face::CardFaceTint;
pub use card_size::{CardSizeSetting, Zoom, Zoomed, step as zoom_step};
pub use color_vision::ColorVision;
pub use console_face::{CONSOLE_FONT_SIZES, ConsoleFace};
pub use dock::DockSide;
pub use full_screen::{Decision, FullScreen, TerminalKind, decide};
pub use interface::{AuraColour, AuraSize, AuraStyle};
pub use priority::ProcessPriority;
pub use table::{TableColour, TableFeel};

use crate::audio::AudioSettings;
use crate::caps::{ColorDepth, TerminalCaps};
use crate::card_style::{DEFAULT_BACK, DEFAULT_FRONT};
use crate::i18n::Locale;
use crate::theme::ThemeId;

/// The sound style used when none is saved or the saved one no longer exists.
pub const DEFAULT_SOUND_STYLE: &str = "default";

/// How cards and markers are drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GlyphMode {
    Font,
    Unicode,
    Ascii,
}

/// Color mode: detected from the terminal or forced by the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorMode {
    Auto,
    Ansi16,
    Ansi256,
    TrueColor,
}

/// Which halos the board draws: all three (focus, carried run, drop targets), or only the focus.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuraMode {
    #[default]
    Full,
    FocusOnly,
}

/// Everything the player can change in the settings screen. Styles are folder ids.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub locale: Locale,
    pub theme: ThemeId,
    pub front_style: String,
    pub back_style: String,
    pub glyph_mode: GlyphMode,
    /// Whether the player confirmed that the oXySol card font shows right (the graphics check).
    /// `Font` glyph mode draws the card font only with this set; without it the Unicode glyphs
    /// are drawn. An older saved value without the field loads as `false`.
    pub font_confirmed: bool,
    /// Whether the classic Windows console is asked to use the oXySol font ("Use the oXySol font in
    /// this console"). An older saved value without the field loads as `false`.
    pub console_font: bool,
    /// The font face asked of the classic Windows console. An older saved value without the field
    /// loads as `Default` (the face the console had).
    pub console_face: ConsoleFace,
    /// The font height in pixels asked of the classic Windows console; `0` is the size it had. An
    /// older saved value without the field loads as `0`.
    pub console_font_size: u8,
    pub color_mode: ColorMode,
    pub sound_style: String,
    pub cues: bool,
    pub music: bool,
    /// Volume in percent, `0..=100`.
    pub volume: u8,
    /// Whether the board shows the guide panel beside it (when the terminal is wide enough).
    pub guide_panel: bool,
    /// Whether the board shows the possible moves panel beside it (a window below 130 columns). An
    /// older saved value without the field loads as `true`.
    pub moves_panel: bool,
    /// Where the help window sits: centred, or docked to the right or left side (P3-28). An older
    /// saved value without the field loads as centred.
    pub help_dock: DockSide,
    /// The undo mode offered in the game selection and used for the next game: `true` allows undo
    /// and redo. An older saved value without the field loads as `true`.
    pub undo_allowed: bool,
    /// Whether a new game with a fresh seed searches for a deal the solver proves winnable.
    /// An older saved value without the field loads as `false`.
    pub winnable_only: bool,
    /// Whether a forced win is played out automatically move by move. An older saved value without
    /// the field loads as `true`.
    pub auto_finish: bool,
    /// Whether `a` and a double click move a card to its best target (quick move). When off they do
    /// nothing and the moves window says why. An older saved value without the field loads as `true`.
    pub quick_move: bool,
    /// Which halos the board draws. An older saved value without the field loads as `Full`.
    pub aura: AuraMode,
    /// Whether the board marks the places a carried run can go (`+` and the target halo). An older
    /// saved value without the field loads as `true`.
    pub target_marks: bool,
    /// Whether the game fills the whole screen. An older saved value without the field loads as
    /// `Auto`.
    pub full_screen: FullScreen,
    /// Which pattern and palette face-down cards use.
    pub card_back: CardBack,
    /// The color of face-up cards; the suit colors adapt to it. An older saved value without the
    /// field loads as `Theme`.
    pub card_face: CardFaceTint,
    /// Color vision mode: adapts meaning tokens to a palette safe for that deficiency. An older
    /// saved value without the field loads as `Normal`.
    pub color_vision: ColorVision,
    /// Whether face-up cards use four suit colors (spades black, hearts red, diamonds blue, clubs
    /// green) in every color vision mode. An older saved value without the field loads as `false`.
    pub four_color_deck: bool,
    /// The colour of the table the cards lie on. An older saved value without the field loads as
    /// `Theme`.
    pub table_colour: TableColour,
    /// The texture of the table. An older saved value without the field loads as `Plain`.
    pub table_feel: TableFeel,
    /// The card size on the board. An older saved value without the field loads as `Auto`.
    pub card_size: CardSizeSetting,
    /// The colour of the aura. An older saved value without the field loads as `Theme`.
    pub aura_colour: AuraColour,
    /// How the aura is drawn. An older saved value without the field loads as `Halo`.
    pub aura_style: AuraStyle,
    /// How wide the aura is. An older saved value without the field loads as `Thin`.
    pub aura_size: AuraSize,
    /// The process priority asked of the system. An older saved value without the field loads as
    /// `Normal`; `--priority` overrides it for one run.
    pub process_priority: ProcessPriority,
    pub favourites: Vec<oxysol_core::Variant>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            locale: Locale::EnUs,
            theme: ThemeId::Dark,
            front_style: DEFAULT_FRONT.to_owned(),
            back_style: DEFAULT_BACK.to_owned(),
            glyph_mode: GlyphMode::Unicode,
            font_confirmed: false,
            console_font: false,
            console_face: ConsoleFace::Default,
            console_font_size: 0,
            color_mode: ColorMode::Auto,
            sound_style: DEFAULT_SOUND_STYLE.to_owned(),
            cues: true,
            music: false,
            volume: 70,
            guide_panel: true,
            moves_panel: true,
            help_dock: DockSide::Centre,
            undo_allowed: true,
            winnable_only: false,
            auto_finish: true,
            quick_move: true,
            aura: AuraMode::Full,
            target_marks: true,
            full_screen: FullScreen::Auto,
            card_back: CardBack::Theme,
            card_face: CardFaceTint::Theme,
            color_vision: ColorVision::Normal,
            four_color_deck: false,
            table_colour: TableColour::Theme,
            table_feel: TableFeel::Plain,
            card_size: CardSizeSetting::Auto,
            aura_colour: AuraColour::Theme,
            aura_style: AuraStyle::Halo,
            aura_size: AuraSize::Thin,
            process_priority: ProcessPriority::Normal,
            favourites: Vec::new(),
        }
    }
}

impl Settings {
    /// Defaults adapted to a terminal: ASCII glyphs where Unicode is not available.
    pub fn for_caps(caps: &TerminalCaps) -> Self {
        let glyph_mode = if caps.unicode {
            GlyphMode::Unicode
        } else {
            GlyphMode::Ascii
        };
        Self {
            glyph_mode,
            ..Self::default()
        }
    }

    /// The glyph mode to draw with: `Font` only when the player confirmed the oXySol font or the
    /// console shows it (`console_shows_font`), else the Unicode glyphs, so the Private Use Area
    /// code points never show as squares.
    pub fn drawn_glyph_mode(&self, console_shows_font: bool) -> GlyphMode {
        match self.glyph_mode {
            GlyphMode::Font if !self.font_confirmed && !console_shows_font => GlyphMode::Unicode,
            mode => mode,
        }
    }

    /// The color depth to render with: the forced mode, or the detected one.
    pub fn color_depth(&self, caps: &TerminalCaps) -> ColorDepth {
        match self.color_mode {
            ColorMode::Auto => caps.color,
            ColorMode::Ansi16 => ColorDepth::Ansi16,
            ColorMode::Ansi256 => ColorDepth::Ansi256,
            ColorMode::TrueColor => ColorDepth::TrueColor,
        }
    }

    /// The part of the settings the audio output needs.
    pub fn audio(&self) -> AudioSettings {
        AudioSettings {
            style: self.sound_style.clone(),
            volume: self.volume.min(100),
            cues: self.cues,
            music: self.music,
        }
    }
}

#[cfg(test)]
mod tests;
