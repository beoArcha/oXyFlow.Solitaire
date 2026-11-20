//! What a row of the settings screen shows and how one step changes [`Settings`].

use super::options::{
    BACK_STYLES, CARD_BACKS, COLOR_MODES, FIELDS, FRONT_STYLES, FULL_SCREEN_MODES, Field,
    GLYPH_MODES, Step,
};
use crate::audio::{AudioStatus, StyleInfo};
use crate::i18n::{Locale, t, tf};
use crate::settings::{CardBack, CardSizeSetting, ColorMode, FullScreen, GlyphMode, Settings};
use crate::theme::ThemeId;
use crate::widgets::card::back;

const VOLUME_STEP: u8 = 10;

const fn theme_key(theme: ThemeId) -> &'static str {
    match theme {
        ThemeId::Dark => "settings.theme.dark",
        ThemeId::Light => "settings.theme.light",
        ThemeId::HighContrast => "settings.theme.high_contrast",
        ThemeId::Monokai => "settings.theme.monokai",
        ThemeId::Dracula => "settings.theme.dracula",
    }
}

const fn card_back_key(card_back: CardBack) -> &'static str {
    match card_back {
        CardBack::Theme => "settings.card_back.theme",
        CardBack::Monokai => "settings.card_back.monokai",
        CardBack::Dracula => "settings.card_back.dracula",
    }
}

const fn glyph_key(mode: GlyphMode) -> &'static str {
    match mode {
        GlyphMode::Font | GlyphMode::Unicode => "settings.glyph.unicode",
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

const fn full_screen_key(mode: FullScreen) -> &'static str {
    match mode {
        FullScreen::Auto => "settings.full_screen.auto",
        FullScreen::On => "settings.value.on",
        FullScreen::Off => "settings.value.off",
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

/// The name of a card style: its localized name for the ids offered today, its id otherwise.
fn card_style_name(id: &str) -> String {
    match id {
        "international" => t("settings.style.international").into_owned(),
        "default" => t("settings.style.default").into_owned(),
        other => other.to_owned(),
    }
}

/// The widest label of the rows, in characters: the column the values start after.
fn label_width() -> usize {
    FIELDS
        .iter()
        .map(|field| t(field.label_key()).chars().count())
        .max()
        .unwrap_or_default()
}

fn aid_text(field: Field, settings: &Settings) -> String {
    field
        .aid()
        .map_or_else(String::new, |aid| aid.value_text(&settings.aids()))
}

fn toggled_aid(field: Field, settings: &Settings) -> Settings {
    field.aid().map_or_else(
        || settings.clone(),
        |aid| settings.with_aids(aid.toggled(&settings.aids())),
    )
}

/// The value of a row as the player reads it.
pub fn value_text(field: Field, settings: &Settings, sound_styles: &[StyleInfo]) -> String {
    match field {
        Field::Language => settings.locale.own_name().to_owned(),
        Field::Theme => t(theme_key(settings.theme)).into_owned(),
        Field::CardBack => format!(
            "{} {}",
            t(card_back_key(settings.card_back)),
            back::preview(settings.card_back, settings.glyph_mode)
        ),
        Field::FrontStyle => card_style_name(&settings.front_style),
        Field::BackStyle => card_style_name(&settings.back_style),
        Field::Glyphs => t(glyph_key(settings.glyph_mode)).into_owned(),
        Field::Colors => t(color_key(settings.color_mode)).into_owned(),
        Field::CardSize => t(settings.card_size.label_key()).into_owned(),
        Field::FullScreen => t(full_screen_key(settings.full_screen)).into_owned(),
        Field::SoundStyle => sound_style_name(sound_styles, &settings.sound_style).to_owned(),
        Field::Cues => switch_text(settings.cues),
        Field::Music => switch_text(settings.music),
        Field::Volume => format!("{}%", settings.volume),
        Field::Aura
        | Field::TargetMarks
        | Field::GuidePanel
        | Field::MovesPanel
        | Field::AutoFinish => aid_text(field, settings),
    }
}

/// The text of a row: the label, leaders of dots up to the value column, then the value. Sound rows
/// say when sound is unavailable, and a side panel row says which columns it needs when the screen
/// of `columns` is too narrow. `arrows` are the glyphs (left, right) that frame the value of the
/// focused row; every other row keeps the two columns free, so the values line up.
pub fn line(
    field: Field,
    settings: &Settings,
    audio_available: bool,
    sound_styles: &[StyleInfo],
    columns: u16,
    arrows: Option<(&str, &str)>,
) -> String {
    let mut value = value_text(field, settings, sound_styles);
    if field.is_audio() && !audio_available {
        value = format!("{value} ({})", t("settings.value.unavailable"));
    }
    if let Some(aid) = field.aid() {
        value = aid.value_for(&settings.aids(), columns);
    }
    let framed = match arrows {
        Some((left, right)) => format!("{left} {value} {right}"),
        None => format!("  {value}"),
    };
    let name = t(field.label_key()).into_owned();
    let leaders = ".".repeat(label_width().saturating_sub(name.chars().count()) + 2);
    let label = format!("{name} {leaders}");
    tf(
        "settings.row.line",
        &[("label", label.as_str()), ("value", framed.as_str())],
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
        Field::Language => next.locale = cycle(&Locale::offered(), settings.locale, step),
        Field::Theme => next.theme = cycle(&ThemeId::ALL, settings.theme, step),
        Field::CardBack => next.card_back = cycle(&CARD_BACKS, settings.card_back, step),
        Field::FrontStyle => {
            next.front_style = cycle_str(&FRONT_STYLES, &settings.front_style, step);
        }
        Field::BackStyle => {
            next.back_style = cycle_str(&BACK_STYLES, &settings.back_style, step);
        }
        Field::Glyphs => next.glyph_mode = cycle(&GLYPH_MODES, settings.glyph_mode, step),
        Field::Colors => next.color_mode = cycle(&COLOR_MODES, settings.color_mode, step),
        Field::CardSize => {
            next.card_size = cycle(&CardSizeSetting::ALL, settings.card_size, step);
        }
        Field::FullScreen => {
            next.full_screen = cycle(&FULL_SCREEN_MODES, settings.full_screen, step);
        }
        Field::SoundStyle => {
            let ids: Vec<&str> = sound_styles.iter().map(|style| style.id.as_str()).collect();
            next.sound_style = cycle_str(&ids, &settings.sound_style, step);
        }
        Field::Cues => next.cues = !settings.cues,
        Field::Music => next.music = !settings.music,
        Field::Volume => next.volume = volume_step(settings.volume, step),
        Field::Aura
        | Field::TargetMarks
        | Field::GuidePanel
        | Field::MovesPanel
        | Field::AutoFinish => return toggled_aid(field, settings),
    }
    next
}
