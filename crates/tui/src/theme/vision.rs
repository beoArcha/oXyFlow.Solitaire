//! Color vision deficiency remapping (Okabe-Ito based palettes) and Machado (2009) simulation
//! with CIELAB color-difference metrics for testing.

use ratatui::style::Color;

use super::ThemeId;
use super::palette::{Palette, Swatch, swatch};
use crate::settings::ColorVision;

/// Converts an sRGB byte `0..=255` to linear light `0.0..=1.0`.
pub(super) fn srgb_to_linear(c: u8) -> f32 {
    let s = f32::from(c) / 255.0;
    if s <= 0.04045 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

/// Converts a linear light value `0.0..=1.0` to an sRGB byte `0..=255`.
fn linear_to_srgb(v: f32) -> u8 {
    let clamped = v.clamp(0.0, 1.0);
    let s = if clamped <= 0.003_130_8 {
        12.92 * clamped
    } else {
        1.055 * clamped.powf(1.0 / 2.4) - 0.055
    };
    (s * 255.0).round().clamp(0.0, 255.0) as u8
}

/// Simulates how `rgb` is perceived under `vision` using Machado et al. (2009) matrices (severity
/// 1.0) in linear sRGB, or perceptual luminance for [`ColorVision::Monochrome`].
#[must_use]
pub fn simulate(rgb: [u8; 3], vision: ColorVision) -> [u8; 3] {
    if vision == ColorVision::Normal {
        return rgb;
    }
    let r = srgb_to_linear(rgb[0]);
    let g = srgb_to_linear(rgb[1]);
    let b = srgb_to_linear(rgb[2]);
    let matrix: [[f32; 3]; 3] = match vision {
        ColorVision::Normal => return rgb,
        ColorVision::Protanopia => [
            [0.152_286, 1.052_583, -0.204_868],
            [0.114_503, 0.786_281, 0.099_216],
            [-0.003_882, -0.048_116, 1.051_998],
        ],
        ColorVision::Deuteranopia => [
            [0.367_322, 0.860_646, -0.227_968],
            [0.280_085, 0.672_501, 0.047_413],
            [-0.011_820, 0.042_940, 0.968_881],
        ],
        ColorVision::Tritanopia => [
            [1.255_528, -0.076_749, -0.178_779],
            [-0.078_411, 0.930_809, 0.147_602],
            [0.004_733, 0.691_367, 0.303_900],
        ],
        ColorVision::Monochrome => {
            let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
            let gray = linear_to_srgb(y);
            return [gray, gray, gray];
        }
    };
    let sr = matrix[0][0] * r + matrix[0][1] * g + matrix[0][2] * b;
    let sg = matrix[1][0] * r + matrix[1][1] * g + matrix[1][2] * b;
    let sb = matrix[2][0] * r + matrix[2][1] * g + matrix[2][2] * b;
    [linear_to_srgb(sr), linear_to_srgb(sg), linear_to_srgb(sb)]
}

fn lab_f(t: f32) -> f32 {
    if t > 0.008_856 {
        t.cbrt()
    } else {
        7.787 * t + 16.0 / 116.0
    }
}

fn rgb_to_lab(rgb: [u8; 3]) -> [f32; 3] {
    let r = srgb_to_linear(rgb[0]);
    let g = srgb_to_linear(rgb[1]);
    let b = srgb_to_linear(rgb[2]);
    let x = (0.412_456_4 * r + 0.357_576_1 * g + 0.180_437_5 * b) / 0.950_47;
    let y = 0.212_672_9 * r + 0.715_152_2 * g + 0.072_175_0 * b;
    let z = (0.019_333_9 * r + 0.119_192 * g + 0.950_304_1 * b) / 1.088_83;
    let fx = lab_f(x);
    let fy = lab_f(y);
    let fz = lab_f(z);
    [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
}

/// CIE76 perceptual color difference ($\Delta E^*_{ab}$) between two sRGB colors.
#[must_use]
pub fn delta_e(a: [u8; 3], b: [u8; 3]) -> f32 {
    let [l1, a1, b1] = rgb_to_lab(a);
    let [l2, a2, b2] = rgb_to_lab(b);
    let dl = l1 - l2;
    let da = a1 - a2;
    let db = b1 - b2;
    (dl * dl + da * da + db * db).sqrt()
}

fn to_gray(s: Swatch) -> Swatch {
    let [g, _, _] = simulate(s.rgb, ColorVision::Monochrome);
    let ansi16 = match g {
        0..=55 => Color::Black,
        56..=127 => Color::DarkGray,
        128..=205 => Color::Gray,
        _ => Color::White,
    };
    swatch(g, g, g, ansi16)
}

fn remap_red_green(id: ThemeId, palette: Palette) -> Palette {
    if id == ThemeId::Light {
        return Palette {
            accent: swatch(0, 102, 166, Color::Blue),
            border_focus: swatch(250, 240, 100, Color::Yellow),
            focus_fg: swatch(255, 255, 255, Color::White),
            focus_bg: swatch(0, 96, 152, Color::Blue),
            error: swatch(176, 56, 0, Color::Red),
            success: swatch(0, 96, 152, Color::Blue),
            card_red: swatch(176, 56, 0, Color::Red),
            target: swatch(250, 240, 100, Color::Yellow),
            selection: swatch(140, 225, 255, Color::LightCyan),
            hint: swatch(255, 195, 230, Color::White),
            aura_focus: swatch(255, 252, 210, Color::LightYellow),
            aura_carry: swatch(232, 255, 255, Color::LightCyan),
            ..palette
        };
    }
    let card_red = if id == ThemeId::HighContrast {
        swatch(155, 42, 0, Color::Red)
    } else {
        swatch(176, 56, 0, Color::Red)
    };
    Palette {
        accent: swatch(120, 210, 255, Color::LightCyan),
        border_focus: swatch(245, 232, 80, Color::Yellow),
        focus_fg: swatch(16, 18, 24, Color::Black),
        focus_bg: swatch(245, 232, 80, Color::Yellow),
        error: swatch(255, 168, 92, Color::LightRed),
        success: swatch(110, 205, 255, Color::LightCyan),
        card_red,
        target: swatch(250, 240, 100, Color::Yellow),
        selection: swatch(140, 225, 255, Color::LightCyan),
        hint: swatch(255, 195, 230, Color::LightMagenta),
        aura_focus: swatch(255, 252, 210, Color::LightYellow),
        aura_carry: swatch(232, 255, 255, Color::LightCyan),
        ..palette
    }
}

fn remap_tritanopia(id: ThemeId, palette: Palette) -> Palette {
    if id == ThemeId::Light {
        return Palette {
            accent: swatch(0, 110, 104, Color::Cyan),
            border_focus: swatch(255, 178, 170, Color::LightRed),
            focus_fg: swatch(255, 255, 255, Color::White),
            focus_bg: swatch(0, 110, 104, Color::Cyan),
            error: swatch(184, 24, 48, Color::Red),
            success: swatch(0, 118, 96, Color::Green),
            card_red: swatch(180, 20, 40, Color::Red),
            target: swatch(255, 192, 182, Color::LightRed),
            selection: swatch(130, 240, 228, Color::LightCyan),
            hint: swatch(255, 228, 232, Color::White),
            aura_focus: swatch(255, 250, 252, Color::White),
            aura_carry: swatch(230, 255, 252, Color::LightCyan),
            ..palette
        };
    }
    let card_red = if id == ThemeId::HighContrast {
        swatch(160, 0, 20, Color::Red)
    } else {
        swatch(180, 20, 40, Color::Red)
    };
    Palette {
        accent: swatch(95, 230, 220, Color::LightCyan),
        border_focus: swatch(255, 192, 182, Color::LightRed),
        focus_fg: swatch(16, 18, 24, Color::Black),
        focus_bg: swatch(255, 192, 182, Color::LightRed),
        error: swatch(255, 158, 158, Color::LightRed),
        success: swatch(80, 235, 195, Color::LightGreen),
        card_red,
        target: swatch(255, 192, 182, Color::LightRed),
        selection: swatch(130, 240, 228, Color::LightCyan),
        hint: swatch(255, 228, 232, Color::White),
        aura_focus: swatch(255, 250, 252, Color::White),
        aura_carry: swatch(230, 255, 252, Color::LightCyan),
        ..palette
    }
}

fn remap_monochrome(id: ThemeId, palette: Palette) -> Palette {
    let mut gray = Palette {
        background: to_gray(palette.background),
        surface: to_gray(palette.surface),
        text: to_gray(palette.text),
        text_muted: to_gray(palette.text_muted),
        accent: to_gray(palette.accent),
        border: to_gray(palette.border),
        border_focus: to_gray(palette.border_focus),
        focus_fg: to_gray(palette.focus_fg),
        focus_bg: to_gray(palette.focus_bg),
        error: to_gray(palette.error),
        success: to_gray(palette.success),
        table: to_gray(palette.table),
        table_text: to_gray(palette.table_text),
        slot: to_gray(palette.slot),
        card_face: to_gray(palette.card_face),
        card_back: to_gray(palette.card_back),
        card_back_mark: to_gray(palette.card_back_mark),
        card_red: swatch(88, 88, 88, Color::DarkGray),
        card_black: swatch(0, 0, 0, Color::Black),
        target: swatch(255, 255, 255, Color::White),
        selection: swatch(231, 231, 231, Color::Gray),
        hint: swatch(208, 208, 208, Color::Gray),
        aura_focus: swatch(255, 255, 255, Color::White),
        aura_carry: swatch(250, 250, 250, Color::White),
        field_edge: palette.field_edge,
    };
    if id == ThemeId::Light {
        gray.error = swatch(64, 64, 64, Color::DarkGray);
        gray.success = swatch(108, 108, 108, Color::DarkGray);
    } else {
        gray.error = swatch(186, 186, 186, Color::Gray);
        gray.success = swatch(238, 238, 238, Color::White);
    }
    gray
}

/// Remaps `palette` of theme `id` for `vision`.
#[must_use]
pub fn remap(id: ThemeId, palette: Palette, vision: ColorVision) -> Palette {
    match vision {
        ColorVision::Normal => palette,
        ColorVision::Protanopia | ColorVision::Deuteranopia => remap_red_green(id, palette),
        ColorVision::Tritanopia => remap_tritanopia(id, palette),
        ColorVision::Monochrome => remap_monochrome(id, palette),
    }
}
