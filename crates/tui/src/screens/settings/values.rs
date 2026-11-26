//! What a row of the settings screen shows and when it is unavailable.

pub use super::change::changed_for_facts;
#[cfg(test)]
pub use super::change::{changed, changed_with_custom};
use super::font_rows;
use super::game_option_rows;
use super::options::{FIELDS, Field};
use super::priority_row;
use crate::app::{CardFontState, FontFacts};
use crate::audio::{AudioStatus, StyleInfo};
use crate::card_style::CardStyles;
use crate::i18n::{t, tf};
use crate::settings::{CardBack, ColorMode, FullScreen, GlyphMode, Settings, TerminalKind};
use crate::theme::table::{felt, felt_for};
use crate::theme::{Palette, ThemeId};

const fn theme_key(theme: ThemeId) -> &'static str {
    match theme {
        ThemeId::Dark => "settings.theme.dark",
        ThemeId::Light => "settings.theme.light",
        ThemeId::HighContrast => "settings.theme.high_contrast",
        ThemeId::Monokai => "settings.theme.monokai",
        ThemeId::Dracula => "settings.theme.dracula",
        ThemeId::Custom => "settings.theme.custom",
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

const fn full_screen_key(mode: FullScreen) -> &'static str {
    match mode {
        FullScreen::Auto => "settings.full_screen.auto",
        FullScreen::On => "settings.value.on",
        FullScreen::Off => "settings.value.off",
    }
}

fn table_colour_text(settings: &Settings, custom: Option<&Palette>) -> String {
    let name = t(settings.table_colour.label_key()).into_owned();
    let choice = match (settings.theme, custom) {
        (ThemeId::Custom, Some(palette)) => felt(ThemeId::Custom, palette, settings.table_colour),
        _ => felt_for(settings.theme, settings.table_colour),
    };
    if choice.shifted {
        tf("settings.table.low_contrast", &[("name", &name)])
    } else {
        name
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

fn front_style_name(id: &str) -> String {
    CardStyles::embedded()
        .fronts()
        .iter()
        .find(|style| style.id == id)
        .map_or_else(|| id.to_owned(), |style| t(&style.name_key).into_owned())
}

fn back_style_name(id: &str) -> String {
    CardStyles::embedded()
        .backs()
        .iter()
        .find(|style| style.id == id)
        .map_or_else(|| id.to_owned(), |style| t(&style.name_key).into_owned())
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

/// The value of a row as the player reads it.
#[cfg(test)]
pub fn value_text(field: Field, settings: &Settings, sound_styles: &[StyleInfo]) -> String {
    value_text_with(field, settings, sound_styles, None)
}

fn value_text_with(
    field: Field,
    settings: &Settings,
    sound_styles: &[StyleInfo],
    custom: Option<&Palette>,
) -> String {
    match field {
        Field::Language => settings.locale.own_name().to_owned(),
        Field::LowVision => switch_text(settings.low_vision),
        Field::Theme => t(theme_key(settings.theme)).into_owned(),
        Field::ColorVision => t(settings.color_vision.label_key()).into_owned(),
        Field::CardBack => t(card_back_key(settings.card_back)).into_owned(),
        Field::CardFace => t(settings.card_face.label_key()).into_owned(),
        Field::FourColorDeck => switch_text(settings.four_color_deck),
        Field::TableColour => table_colour_text(settings, custom),
        Field::TableFeel => t(settings.table_feel.label_key()).into_owned(),
        Field::FrontStyle => front_style_name(&settings.front_style),
        Field::BackStyle => back_style_name(&settings.back_style),
        Field::Glyphs => t(glyph_key(settings.glyph_mode)).into_owned(),
        Field::ConsoleFont => switch_text(settings.console_font),
        Field::FontFace | Field::FontSize => font_rows::value_text(field, settings),
        Field::Colors => t(color_key(settings.color_mode)).into_owned(),
        Field::CardSize => t(settings.card_size.label_key()).into_owned(),
        Field::InterfaceSize => t(settings.interface_size.label_key()).into_owned(),
        Field::AuraColour => t(settings.aura_colour.label_key()).into_owned(),
        Field::AuraStyle => t(settings.aura_style.label_key()).into_owned(),
        Field::AuraSize => t(settings.aura_size.label_key()).into_owned(),
        Field::Priority => t(settings.process_priority.label_key()).into_owned(),
        Field::FullScreen => t(full_screen_key(settings.full_screen)).into_owned(),
        Field::WindowSize => t(settings.window_size.label_key()).into_owned(),
        Field::SoundStyle => sound_style_name(sound_styles, &settings.sound_style).to_owned(),
        Field::Cues => switch_text(settings.cues),
        Field::Music => switch_text(settings.music),
        Field::Volume => format!("{}%", settings.volume),
        Field::Aura
        | Field::TargetMarks
        | Field::GuidePanel
        | Field::MovesPanel
        | Field::AutoFinish
        | Field::QuickMove => aid_text(field, settings),
        Field::KlondikePasses
        | Field::KlondikeEmptyColumn
        | Field::SpiderDeal
        | Field::GolfWrap
        | Field::GolfKingStops => game_option_rows::value(field, settings.game_options),
    }
}

/// The value of the console font row: the option, and why it does not work when it does not.
fn console_font_value(settings: &Settings, facts: FontFacts) -> String {
    let option = switch_text(settings.console_font);
    let reason = match facts.card_font {
        CardFontState::Unavailable => return t("settings.value.unavailable").into_owned(),
        CardFontState::NotRegistered if settings.console_font => "settings.value.not_installed",
        CardFontState::Refused if settings.console_font => "settings.value.refused",
        _ => return option,
    };
    format!("{option} ({})", t(reason))
}

/// Whether a row cannot change now: a step on it does nothing. Such a row is drawn dimmed and without
/// arrows; its value says why (sound not available, a font the game cannot change).
pub(super) fn is_unavailable(field: Field, audio_available: bool, facts: FontFacts) -> bool {
    (field.is_audio() && !audio_available)
        || (field == Field::ConsoleFont && facts.card_font == CardFontState::Unavailable)
        || (field == Field::WindowSize
            && (facts.terminal != TerminalKind::ClassicConsole || !facts.owns_window))
        || font_rows::is_inert(field, facts)
}

/// The text of a row: the label, leaders of dots up to the value column, then the value. Sound rows
/// say when sound is unavailable, and a side panel row says which columns it needs when the screen
/// of `columns` is too narrow. `arrows` are the glyphs (left, right) that frame the value of the
/// focused row; every other row keeps the two columns free, so the values line up, and an
/// unavailable row takes no arrows at all.
#[cfg(test)]
pub fn line(
    field: Field,
    settings: &Settings,
    audio_available: bool,
    sound_styles: &[StyleInfo],
    (columns, facts): (u16, FontFacts),
    arrows: Option<(&str, &str)>,
) -> String {
    line_with_custom(
        field,
        settings,
        audio_available,
        sound_styles,
        (columns, facts),
        arrows,
        None,
    )
}

pub fn line_with_custom(
    field: Field,
    settings: &Settings,
    audio_available: bool,
    sound_styles: &[StyleInfo],
    (columns, facts): (u16, FontFacts),
    arrows: Option<(&str, &str)>,
    custom: Option<&Palette>,
) -> String {
    let mut value = value_text_with(field, settings, sound_styles, custom);
    if field == Field::ConsoleFont {
        value = console_font_value(settings, facts);
    }
    if matches!(field, Field::FontFace | Field::FontSize) {
        value = font_rows::value(field, settings, facts);
    }
    if field == Field::Priority {
        value = priority_row::value(settings, facts);
    }
    if field == Field::WindowSize && is_unavailable(field, audio_available, facts) {
        value = t("settings.value.unavailable").into_owned();
    }
    if field.is_audio() && !audio_available {
        value = format!("{value} ({})", t("settings.value.unavailable"));
    }
    if let Some(aid) = field.aid() {
        value = aid.value_for_size(&settings.aids(), columns, settings.interface_size);
    }
    let framed = match arrows {
        Some((left, right)) if !is_unavailable(field, audio_available, facts) => {
            format!("{left} {value} {right}")
        }
        _ => format!("  {value}"),
    };
    let name = t(field.label_key()).into_owned();
    let leaders = ".".repeat(label_width().saturating_sub(name.chars().count()) + 2);
    let label = format!("{name} {leaders}");
    tf(
        "settings.row.line",
        &[("label", label.as_str()), ("value", framed.as_str())],
    )
}
