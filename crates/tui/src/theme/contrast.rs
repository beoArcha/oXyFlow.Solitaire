//! High-contrast color theme variant and WCAG contrast ratio calculations.

fn channel_to_linear(c: u8) -> f32 {
    let s = f32::from(c) / 255.0;
    if s <= 0.04045 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

fn relative_luminance(rgb: [u8; 3]) -> f32 {
    let r = channel_to_linear(rgb[0]);
    let g = channel_to_linear(rgb[1]);
    let b = channel_to_linear(rgb[2]);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Computes the WCAG 2 contrast ratio between two sRGB colors.
///
/// Returns a value in the range `1.0..=21.0`. The result is symmetric with
/// respect to the argument order.
#[must_use]
pub fn contrast_ratio(a: [u8; 3], b: [u8; 3]) -> f32 {
    let l1 = relative_luminance(a);
    let l2 = relative_luminance(b);
    let (lighter, darker) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (lighter + 0.05) / (darker + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_black_on_white() {
        let black = [0, 0, 0];
        let white = [255, 255, 255];
        let ratio = contrast_ratio(black, white);
        assert!((ratio - 21.0).abs() <= 0.01);
    }

    #[test]
    fn test_identical_colors() {
        let white = [255, 255, 255];
        let black = [0, 0, 0];
        let gray = [128, 128, 128];
        let color = [42, 100, 200];
        assert_eq!(contrast_ratio(white, white), 1.0);
        assert_eq!(contrast_ratio(black, black), 1.0);
        assert_eq!(contrast_ratio(gray, gray), 1.0);
        assert_eq!(contrast_ratio(color, color), 1.0);
    }

    #[test]
    fn test_symmetry() {
        let c1 = [12, 85, 200];
        let c2 = [220, 180, 50];
        assert_eq!(contrast_ratio(c1, c2), contrast_ratio(c2, c1));
    }

    #[test]
    fn test_card_mid_gray_on_white() {
        let mid_gray = [119, 119, 119];
        let white = [255, 255, 255];
        let ratio = contrast_ratio(mid_gray, white);
        assert!((ratio - 4.48).abs() <= 0.01);
    }
}
