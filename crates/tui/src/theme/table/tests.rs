use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::{felt_for, hex, mix, passes, preset};
use crate::caps::ColorDepth;
use crate::settings::{TableColour, TableFeel};
use crate::theme::table_feel;
use crate::theme::{Theme, ThemeId, Token};

const AREA: Rect = Rect::new(0, 0, 40, 12);

fn drawn(id: ThemeId, colour: TableColour, feel: TableFeel) -> Buffer {
    let theme = Theme::new(id, ColorDepth::TrueColor).with_table(colour);
    let mut buffer = Buffer::empty(AREA);
    table_feel::draw(&mut buffer, AREA, &theme, feel, true);
    buffer
}

fn count_symbol(buffer: &Buffer, symbol: &str) -> usize {
    buffer
        .content()
        .iter()
        .filter(|cell| cell.symbol() == symbol)
        .count()
}

#[test]
fn every_colour_in_every_theme_is_drawn_in_a_felt_that_passes_the_table_tests() {
    for id in ThemeId::ALL {
        for colour in TableColour::ALL {
            let chosen = felt_for(id, colour).swatch.rgb;
            assert!(
                passes(id, &id.palette(), chosen),
                "{id:?} {colour:?} is drawn in {}, which fails",
                hex(chosen)
            );
        }
    }
}

#[test]
fn a_preset_is_shifted_exactly_when_it_fails_and_the_theme_felt_never_is() {
    for id in ThemeId::ALL {
        assert!(!felt_for(id, TableColour::Theme).shifted);
        for colour in TableColour::ALL {
            let Some(requested) = preset(colour) else {
                continue;
            };
            let shifted = felt_for(id, colour).shifted;
            assert_eq!(
                shifted,
                !passes(id, &id.palette(), requested.rgb),
                "{id:?} {colour:?}"
            );
        }
    }
}

#[test]
fn some_preset_fails_in_some_theme_so_the_warning_is_reachable() {
    let shifted = ThemeId::ALL.into_iter().any(|id| {
        TableColour::ALL
            .into_iter()
            .any(|colour| felt_for(id, colour).shifted)
    });
    assert!(shifted);
}

#[test]
fn a_shifted_felt_keeps_the_preset_when_no_shade_is_needed() {
    for id in ThemeId::ALL {
        for colour in TableColour::ALL {
            let felt = felt_for(id, colour);
            if !felt.shifted {
                assert_eq!(
                    Some(felt.swatch),
                    preset(colour).or(Some(id.palette().table))
                );
            }
        }
    }
}

#[test]
fn mixing_moves_each_channel_toward_its_target() {
    assert_eq!(mix([0, 100, 200], [255, 100, 0], 50), [127, 100, 100]);
}

#[test]
fn hex_names_the_colour_in_upper_case() {
    assert_eq!(hex([1, 171, 255]), "#01ABFF");
}

#[test]
fn with_table_replaces_only_the_felt() {
    for id in ThemeId::ALL {
        let theme = Theme::new(id, ColorDepth::TrueColor);
        assert_eq!(theme.with_table(TableColour::Theme), theme);
        let green = theme.with_table(TableColour::Green);
        let expected = felt_for(id, TableColour::Green).swatch;
        assert_eq!(
            green.color(Token::Table),
            expected.resolve(ColorDepth::TrueColor)
        );
        assert_eq!(green.color(Token::TableText), theme.color(Token::TableText));
    }
}

#[test]
fn plain_draws_no_texture_in_any_theme_or_colour() {
    for id in ThemeId::ALL {
        for colour in TableColour::ALL {
            assert_eq!(drawn(id, colour, TableFeel::Plain), Buffer::empty(AREA));
        }
    }
}

#[test]
fn felt_and_lined_draw_their_texture_in_every_theme_and_colour() {
    for id in ThemeId::ALL {
        for colour in TableColour::ALL {
            let felt = drawn(id, colour, TableFeel::Felt);
            assert!(count_symbol(&felt, "·") > 0, "{id:?} {colour:?} felt");
            let lined = drawn(id, colour, TableFeel::Lined);
            assert_eq!(count_symbol(&lined, "─"), 4 * usize::from(AREA.width));
        }
    }
}

#[test]
fn the_texture_never_replaces_text_or_card_cells() {
    let text = vec!["X".repeat(usize::from(AREA.width)); usize::from(AREA.height)];
    for id in ThemeId::ALL {
        for feel in [TableFeel::Felt, TableFeel::Lined] {
            let theme = Theme::new(id, ColorDepth::TrueColor);
            let mut buffer = Buffer::with_lines(text.clone());
            table_feel::draw(&mut buffer, AREA, &theme, feel, true);
            assert_eq!(buffer, Buffer::with_lines(text.clone()), "{id:?} {feel:?}");
        }
    }
}

#[test]
fn ascii_texture_uses_plain_characters() {
    let theme = Theme::new(ThemeId::Dark, ColorDepth::TrueColor);
    let mut buffer = Buffer::empty(AREA);
    table_feel::draw(&mut buffer, AREA, &theme, TableFeel::Lined, false);
    assert_eq!(count_symbol(&buffer, "-"), 4 * usize::from(AREA.width));
    assert_eq!(count_symbol(&buffer, "─"), 0);
}
