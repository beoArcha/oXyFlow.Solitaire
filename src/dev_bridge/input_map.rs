//! Mapping JSON protocol fields (game names, colors, locales, themes, keys, mouse) to TUI types.

use oxysol_core::{DrawCount, PyramidRules, SuitCount, Variant, YukonRules};
use oxysol_tui::caps::{ColorDepth, TerminalCaps};
use oxysol_tui::crossterm::event::{
    KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use oxysol_tui::i18n::Locale;
use oxysol_tui::settings::{ColorMode, GlyphMode, Settings};
use oxysol_tui::theme::ThemeId;
use oxysol_tui::{HeadlessConfig, Input};

use super::ProtocolError;
use super::protocol::StartArgs;

/// What a `start` command opens: the main menu or a specific dealt variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartTarget {
    Menu,
    Game(Variant),
}

/// Builds a [`HeadlessConfig`] and [`StartTarget`] from validated `start` arguments.
pub fn build_start_config(
    args: &StartArgs,
    default_seed: Option<u64>,
) -> Result<(HeadlessConfig, StartTarget), ProtocolError> {
    if args.cols == 0 || args.rows == 0 {
        return Err(ProtocolError::InvalidDimensions {
            cols: args.cols,
            rows: args.rows,
        });
    }
    let target = parse_game(args.game.as_deref())?;
    let (color_depth, color_mode) = parse_colors(&args.colors)?;
    let locale = parse_locale(&args.lang)?;
    let theme = parse_theme(&args.theme)?;
    let caps = TerminalCaps {
        color: color_depth,
        unicode: !args.ascii,
        remote: false,
    };
    let mut settings = Settings::for_caps(&caps);
    settings.locale = locale;
    settings.theme = theme;
    settings.color_mode = color_mode;
    settings.glyph_mode = if args.ascii {
        GlyphMode::Ascii
    } else {
        GlyphMode::Unicode
    };
    let config = HeadlessConfig {
        cols: args.cols,
        rows: args.rows,
        caps,
        settings: Some(settings),
        seed_start: args.seed.or(default_seed).unwrap_or(1),
        ..HeadlessConfig::default()
    };
    Ok((config, target))
}

/// Parses a `game` name (`"menu"`, `None`, or any variant slug).
pub fn parse_game(game: Option<&str>) -> Result<StartTarget, ProtocolError> {
    let Some(raw) = game else {
        return Ok(StartTarget::Menu);
    };
    let normalized = raw.trim().to_ascii_lowercase();
    let variant = match normalized.as_str() {
        "menu" | "" => return Ok(StartTarget::Menu),
        "klondike" | "klondike_one" | "klondike_1" => Variant::Klondike(DrawCount::One),
        "klondike_three" | "klondike_3" => Variant::Klondike(DrawCount::Three),
        "spider" | "spider_one" | "spider_1" => Variant::Spider(SuitCount::One),
        "spider_two" | "spider_2" => Variant::Spider(SuitCount::Two),
        "spider_four" | "spider_4" => Variant::Spider(SuitCount::Four),
        "freecell" | "free_cell" => Variant::FreeCell,
        "pyramid" | "pyramid_relaxed" => Variant::Pyramid(PyramidRules::Relaxed),
        "pyramid_classic" => Variant::Pyramid(PyramidRules::Classic),
        "yukon" | "yukon_classic" => Variant::Yukon(YukonRules::Classic),
        "yukon_russian" | "russian" => Variant::Yukon(YukonRules::Russian),
        "yukon_alaska" | "alaska" => Variant::Yukon(YukonRules::Alaska),
        "yukon_moosehide" | "moosehide" => Variant::Yukon(YukonRules::Moosehide),
        "yukon_australian" | "australian" => Variant::Yukon(YukonRules::Australian),
        "golf" => Variant::Golf,
        "tripeaks" | "tri_peaks" => Variant::TriPeaks,
        _ => return Err(ProtocolError::InvalidGame(raw.to_owned())),
    };
    Ok(StartTarget::Game(variant))
}

fn parse_colors(colors: &str) -> Result<(ColorDepth, ColorMode), ProtocolError> {
    match colors.trim().to_ascii_lowercase().as_str() {
        "16" | "ansi16" => Ok((ColorDepth::Ansi16, ColorMode::Ansi16)),
        "256" | "ansi256" => Ok((ColorDepth::Ansi256, ColorMode::Ansi256)),
        "truecolor" | "true_color" => Ok((ColorDepth::TrueColor, ColorMode::TrueColor)),
        "auto" => Ok((ColorDepth::TrueColor, ColorMode::Auto)),
        _ => Err(ProtocolError::InvalidColors(colors.to_owned())),
    }
}

fn parse_locale(lang: &str) -> Result<Locale, ProtocolError> {
    if let Some(exact) = Locale::ALL
        .into_iter()
        .find(|locale| locale.id().eq_ignore_ascii_case(lang))
    {
        return Ok(exact);
    }
    Locale::from_language_tag(lang).ok_or_else(|| ProtocolError::InvalidLocale(lang.to_owned()))
}

fn parse_theme(theme: &str) -> Result<ThemeId, ProtocolError> {
    match theme.trim().to_ascii_lowercase().as_str() {
        "dark" => Ok(ThemeId::Dark),
        "light" => Ok(ThemeId::Light),
        "high_contrast" | "highcontrast" => Ok(ThemeId::HighContrast),
        "monokai" => Ok(ThemeId::Monokai),
        "dracula" => Ok(ThemeId::Dracula),
        _ => Err(ProtocolError::InvalidTheme(theme.to_owned())),
    }
}

/// Parses modifier strings (`"shift"`, `"ctrl"`, `"alt"`).
pub fn parse_modifiers(names: &[String]) -> Result<KeyModifiers, ProtocolError> {
    let mut mods = KeyModifiers::NONE;
    for name in names {
        match name.trim().to_ascii_lowercase().as_str() {
            "shift" => mods |= KeyModifiers::SHIFT,
            "ctrl" | "control" => mods |= KeyModifiers::CONTROL,
            "alt" => mods |= KeyModifiers::ALT,
            _ => return Err(ProtocolError::InvalidModifier(name.clone())),
        }
    }
    Ok(mods)
}

/// Maps a key name and modifier list to a crossterm [`KeyEvent`].
pub fn parse_key_event(key: &str, modifier_names: &[String]) -> Result<KeyEvent, ProtocolError> {
    let modifiers = parse_modifiers(modifier_names)?;
    let code = parse_key_code(key, modifiers)?;
    Ok(KeyEvent::new(code, modifiers))
}

fn parse_key_code(key: &str, modifiers: KeyModifiers) -> Result<KeyCode, ProtocolError> {
    if let Some(ch) = single_char(key) {
        let adjusted = if modifiers.contains(KeyModifiers::SHIFT) && ch.is_ascii_lowercase() {
            ch.to_ascii_uppercase()
        } else {
            ch
        };
        return Ok(KeyCode::Char(adjusted));
    }
    match key.trim().to_ascii_lowercase().as_str() {
        "enter" | "return" => Ok(KeyCode::Enter),
        "esc" | "escape" => Ok(KeyCode::Esc),
        "tab" if modifiers.contains(KeyModifiers::SHIFT) => Ok(KeyCode::BackTab),
        "tab" => Ok(KeyCode::Tab),
        "backtab" => Ok(KeyCode::BackTab),
        "space" => Ok(KeyCode::Char(' ')),
        "backspace" => Ok(KeyCode::Backspace),
        "up" => Ok(KeyCode::Up),
        "down" => Ok(KeyCode::Down),
        "left" => Ok(KeyCode::Left),
        "right" => Ok(KeyCode::Right),
        "home" => Ok(KeyCode::Home),
        "end" => Ok(KeyCode::End),
        "pageup" | "page_up" | "pgup" => Ok(KeyCode::PageUp),
        "pagedown" | "page_down" | "pgdn" => Ok(KeyCode::PageDown),
        "delete" | "del" => Ok(KeyCode::Delete),
        "insert" | "ins" => Ok(KeyCode::Insert),
        other => parse_function_key(other).ok_or_else(|| ProtocolError::InvalidKey(key.to_owned())),
    }
}

fn single_char(key: &str) -> Option<char> {
    let mut chars = key.chars();
    let first = chars.next()?;
    chars.next().is_none().then_some(first)
}

fn parse_function_key(lower: &str) -> Option<KeyCode> {
    let num = lower.strip_prefix('f')?.parse::<u8>().ok()?;
    (1..=12).contains(&num).then_some(KeyCode::F(num))
}

/// Converts a mouse command into the sequence of [`Input`] events sent to `Headless`.
pub fn parse_mouse_inputs(
    action: &str,
    x: u16,
    y: u16,
    modifier_names: &[String],
) -> Result<Vec<Input>, ProtocolError> {
    let modifiers = parse_modifiers(modifier_names)?;
    let event = |kind| {
        Input::Mouse(MouseEvent {
            kind,
            column: x,
            row: y,
            modifiers,
        })
    };
    let inputs = match action.trim().to_ascii_lowercase().as_str() {
        "move" => vec![event(MouseEventKind::Moved)],
        "down" => vec![event(MouseEventKind::Down(MouseButton::Left))],
        "up" => vec![event(MouseEventKind::Up(MouseButton::Left))],
        "click" => vec![
            event(MouseEventKind::Down(MouseButton::Left)),
            event(MouseEventKind::Up(MouseButton::Left)),
        ],
        "double_click" => vec![
            event(MouseEventKind::Down(MouseButton::Left)),
            event(MouseEventKind::Up(MouseButton::Left)),
            event(MouseEventKind::Down(MouseButton::Left)),
            event(MouseEventKind::Up(MouseButton::Left)),
        ],
        "right_click" => vec![
            event(MouseEventKind::Down(MouseButton::Right)),
            event(MouseEventKind::Up(MouseButton::Right)),
        ],
        _ => return Err(ProtocolError::InvalidMouseAction(action.to_owned())),
    };
    Ok(inputs)
}
