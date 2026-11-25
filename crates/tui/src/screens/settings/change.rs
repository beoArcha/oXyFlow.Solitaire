//! Stepping a setting one option forward or back.

use super::options::{
    AURA_COLOURS, AURA_SIZES, AURA_STYLES, CARD_BACKS, COLOR_MODES, FULL_SCREEN_MODES, Field,
    GLYPH_MODES, Step, WINDOW_SIZES, back_styles, front_styles,
};
use crate::app::FontFacts;
use crate::audio::StyleInfo;
use crate::i18n::Locale;
use crate::settings::{
    CONSOLE_FONT_SIZES, CardFaceTint, CardSizeSetting, ColorVision, ConsoleFace, ProcessPriority,
    Settings, TableColour, TableFeel, TerminalKind,
};
use crate::theme::ThemeId;

const VOLUME_STEP: u8 = 10;

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

fn toggled_aid(field: Field, settings: &Settings) -> Settings {
    field.aid().map_or_else(
        || settings.clone(),
        |aid| settings.with_aids(aid.toggled(&settings.aids())),
    )
}

/// The settings after one step on `field`. Everything else stays as it was.
#[cfg(test)]
pub fn changed(
    field: Field,
    settings: &Settings,
    step: Step,
    sound_styles: &[StyleInfo],
) -> Settings {
    changed_for_facts(field, settings, step, sound_styles, FontFacts::default())
}

/// The settings after one step on `field`, taking terminal/console `facts` into account for
/// low-vision console font sizing.
pub fn changed_for_facts(
    field: Field,
    settings: &Settings,
    step: Step,
    sound_styles: &[StyleInfo],
    facts: FontFacts,
) -> Settings {
    let mut next = settings.clone();
    match field {
        Field::Language => next.locale = cycle(&Locale::offered(), settings.locale, step),
        Field::LowVision => {
            let classic = facts.terminal == TerminalKind::ClassicConsole && facts.owns_window;
            return settings.with_low_vision(!settings.low_vision, classic);
        }
        Field::Theme => next.theme = cycle(&ThemeId::ALL, settings.theme, step),
        Field::ColorVision => {
            next.color_vision = cycle(&ColorVision::ALL, settings.color_vision, step);
        }
        Field::CardBack => next.card_back = cycle(&CARD_BACKS, settings.card_back, step),
        Field::CardFace => next.card_face = cycle(&CardFaceTint::ALL, settings.card_face, step),
        Field::FourColorDeck => next.four_color_deck = !settings.four_color_deck,
        Field::TableColour => {
            next.table_colour = cycle(&TableColour::ALL, settings.table_colour, step)
        }
        Field::TableFeel => next.table_feel = cycle(&TableFeel::ALL, settings.table_feel, step),
        Field::FrontStyle => {
            next.front_style = cycle_str(&front_styles(), &settings.front_style, step);
        }
        Field::BackStyle => {
            next.back_style = cycle_str(&back_styles(), &settings.back_style, step);
        }
        Field::Glyphs => next.glyph_mode = cycle(&GLYPH_MODES, settings.glyph_mode, step),
        Field::ConsoleFont => next.console_font = !settings.console_font,
        Field::Colors => next.color_mode = cycle(&COLOR_MODES, settings.color_mode, step),
        Field::CardSize => {
            next.card_size = cycle(&CardSizeSetting::ALL, settings.card_size, step);
        }
        Field::FontFace => {
            next.console_face = cycle(&ConsoleFace::ALL, settings.console_face, step)
        }
        Field::FontSize => {
            next.console_font_size = cycle(&CONSOLE_FONT_SIZES, settings.console_font_size, step);
        }
        Field::AuraColour => {
            next.aura_colour = cycle(&AURA_COLOURS, settings.aura_colour, step);
        }
        Field::AuraStyle => next.aura_style = cycle(&AURA_STYLES, settings.aura_style, step),
        Field::AuraSize => next.aura_size = cycle(&AURA_SIZES, settings.aura_size, step),
        Field::Priority => {
            next.process_priority = cycle(&ProcessPriority::ALL, settings.process_priority, step);
        }
        Field::FullScreen => {
            next.full_screen = cycle(&FULL_SCREEN_MODES, settings.full_screen, step);
        }
        Field::WindowSize => {
            next.window_size = cycle(&WINDOW_SIZES, settings.window_size, step);
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
        | Field::AutoFinish
        | Field::QuickMove => return toggled_aid(field, settings),
    }
    next
}
