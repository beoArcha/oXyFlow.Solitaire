//! The options of the settings screen: their order, what each row shows, and how one step changes
//! [`Settings`]. Front and back styles are fixed to the ids that exist today; the sound styles are the
//! list the application passes in.

use crate::audio::{AudioStatus, StyleInfo};
use crate::i18n::{Locale, t, tf};
use crate::settings::{ColorMode, GlyphMode, Settings};
use crate::theme::ThemeId;

/// One row of the settings screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Language,
    Theme,
    FrontStyle,
    BackStyle,
    Glyphs,
    Colors,
    SoundStyle,
    Cues,
    Music,
    Volume,
    GuidePanel,
    AutoFinish,
}

/// Every row, top to bottom.
pub const FIELDS: [Field; 12] = [
    Field::Language,
    Field::Theme,
    Field::FrontStyle,
    Field::BackStyle,
    Field::Glyphs,
    Field::Colors,
    Field::SoundStyle,
    Field::Cues,
    Field::Music,
    Field::Volume,
    Field::GuidePanel,
    Field::AutoFinish,
];

/// Which way a change moves through the options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Next,
    Previous,
}

/// Front style ids offered today.
pub const FRONT_STYLES: [&str; 1] = ["international"];
/// Back style ids offered today.
pub const BACK_STYLES: [&str; 1] = ["default"];

const GLYPH_MODES: [GlyphMode; 3] = [GlyphMode::Font, GlyphMode::Unicode, GlyphMode::Ascii];
const COLOR_MODES: [ColorMode; 4] = [
    ColorMode::Auto,
    ColorMode::Ansi16,
    ColorMode::Ansi256,
    ColorMode::TrueColor,
];
const VOLUME_STEP: u8 = 10;

impl Field {
    /// Whether the row needs the sound output.
    pub const fn is_audio(self) -> bool {
        matches!(
            self,
            Field::SoundStyle | Field::Cues | Field::Music | Field::Volume
        )
    }

    /// Localization key of the row label.
    pub const fn label_key(self) -> &'static str {
        match self {
            Field::Language => "settings.row.language",
            Field::Theme => "settings.row.theme",
            Field::FrontStyle => "settings.row.front_style",
            Field::BackStyle => "settings.row.back_style",
            Field::Glyphs => "settings.row.glyphs",
            Field::Colors => "settings.row.colors",
            Field::SoundStyle => "settings.row.sound_style",
            Field::Cues => "settings.row.cues",
            Field::Music => "settings.row.music",
            Field::Volume => "settings.row.volume",
            Field::GuidePanel => "settings.row.guide_panel",
            Field::AutoFinish => "settings.row.auto_finish",
        }
    }
}

const fn locale_key(locale: Locale) -> &'static str {
    match locale {
        Locale::EnUs => "settings.locale.en_us",
        Locale::PlPl => "settings.locale.pl_pl",
    }
}

const fn theme_key(theme: ThemeId) -> &'static str {
    match theme {
        ThemeId::Dark => "settings.theme.dark",
        ThemeId::Light => "settings.theme.light",
        ThemeId::HighContrast => "settings.theme.high_contrast",
    }
}

const fn glyph_key(mode: GlyphMode) -> &'static str {
    match mode {
        GlyphMode::Font => "settings.glyph.font",
        GlyphMode::Unicode => "settings.glyph.unicode",
        GlyphMode::Ascii => "settings.glyph.ascii",
    }
}

const fn color_key(mode: ColorMode) -> &'static str {
    match mode {
        ColorMode::Auto => "settings.color.auto",
        ColorMode::Ansi16 => "settings.color.ansi16",
        ColorMode::Ansi256 => "settings.color.ansi256",
        ColorMode::TrueColor => "settings.color.true_color",
    }
}

fn switch_text(on: bool) -> String {
    let key = if on {
        "settings.value.on"
    } else {
        "settings.value.off"
    };
    t(key).into_owned()
}

/// Localization key of the reason why sound is unavailable; `None` when it is available.
pub const fn unavailable_reason_key(status: AudioStatus) -> Option<&'static str> {
    match status {
        AudioStatus::Ready => None,
        AudioStatus::RemoteSession => Some("settings.audio.remote"),
        AudioStatus::NotBuilt => Some("settings.audio.not_built"),
        AudioStatus::NoDevice => Some("settings.audio.no_device"),
        AudioStatus::StyleFailed => Some("settings.audio.style_failed"),
    }
}

/// The name of a sound style: the name its `style.toml` gives, or its id when the list lacks it.
pub fn sound_style_name<'a>(sound_styles: &'a [StyleInfo], id: &'a str) -> &'a str {
    sound_styles
        .iter()
        .find(|style| style.id == id)
        .map_or(id, |style| style.name.as_str())
}

/// The value of a row as the player reads it.
pub fn value_text(field: Field, settings: &Settings, sound_styles: &[StyleInfo]) -> String {
    match field {
        Field::Language => t(locale_key(settings.locale)).into_owned(),
        Field::Theme => t(theme_key(settings.theme)).into_owned(),
        Field::FrontStyle => settings.front_style.clone(),
        Field::BackStyle => settings.back_style.clone(),
        Field::Glyphs => t(glyph_key(settings.glyph_mode)).into_owned(),
        Field::Colors => t(color_key(settings.color_mode)).into_owned(),
        Field::SoundStyle => sound_style_name(sound_styles, &settings.sound_style).to_owned(),
        Field::Cues => switch_text(settings.cues),
        Field::Music => switch_text(settings.music),
        Field::Volume => format!("{}%", settings.volume),
        Field::GuidePanel => switch_text(settings.guide_panel),
        Field::AutoFinish => switch_text(settings.auto_finish),
    }
}

/// The text of a row: `label: value`, with a marker on sound rows when sound is unavailable.
pub fn line(
    field: Field,
    settings: &Settings,
    audio_available: bool,
    sound_styles: &[StyleInfo],
) -> String {
    let mut value = value_text(field, settings, sound_styles);
    if field.is_audio() && !audio_available {
        value = format!("{value} ({})", t("settings.value.unavailable"));
    }
    let label = t(field.label_key()).into_owned();
    tf(
        "settings.row.line",
        &[("label", label.as_str()), ("value", value.as_str())],
    )
}

fn cycle<T: Copy + PartialEq>(options: &[T], current: T, step: Step) -> T {
    let Some(position) = options.iter().position(|option| *option == current) else {
        return options.first().copied().unwrap_or(current);
    };
    let len = options.len();
    let index = match step {
        Step::Next => (position + 1) % len,
        Step::Previous => (position + len - 1) % len,
    };
    options.get(index).copied().unwrap_or(current)
}

fn cycle_str(options: &[&str], current: &str, step: Step) -> String {
    cycle(options, current, step).to_owned()
}

fn volume_step(volume: u8, step: Step) -> u8 {
    let volume = volume.min(100);
    match step {
        Step::Next => volume.saturating_add(VOLUME_STEP).min(100),
        Step::Previous => volume.saturating_sub(VOLUME_STEP),
    }
}

/// The settings after one step on `field`. Everything else stays as it was.
pub fn changed(
    field: Field,
    settings: &Settings,
    step: Step,
    sound_styles: &[StyleInfo],
) -> Settings {
    let mut next = settings.clone();
    match field {
        Field::Language => next.locale = cycle(&Locale::ALL, settings.locale, step),
        Field::Theme => next.theme = cycle(&ThemeId::ALL, settings.theme, step),
        Field::FrontStyle => {
            next.front_style = cycle_str(&FRONT_STYLES, &settings.front_style, step);
        }
        Field::BackStyle => {
            next.back_style = cycle_str(&BACK_STYLES, &settings.back_style, step);
        }
        Field::Glyphs => next.glyph_mode = cycle(&GLYPH_MODES, settings.glyph_mode, step),
        Field::Colors => next.color_mode = cycle(&COLOR_MODES, settings.color_mode, step),
        Field::SoundStyle => {
            let ids: Vec<&str> = sound_styles.iter().map(|style| style.id.as_str()).collect();
            next.sound_style = cycle_str(&ids, &settings.sound_style, step);
        }
        Field::Cues => next.cues = !settings.cues,
        Field::Music => next.music = !settings.music,
        Field::Volume => next.volume = volume_step(settings.volume, step),
        Field::GuidePanel => next.guide_panel = !settings.guide_panel,
        Field::AutoFinish => next.auto_finish = !settings.auto_finish,
    }
    next
}
