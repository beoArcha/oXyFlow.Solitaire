use std::time::Instant;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;

use super::dock_view;
use super::{self as dock, DockState};
use crate::action::Action;
use crate::app::Modal;
use crate::audio::AudioStatus;
use crate::caps::ColorDepth;
use crate::glyphs::Glyphs;
use crate::i18n::{self, Locale, t, tf};
use crate::input::primary_key;
use crate::screens::ViewCtx;
use crate::settings::{DockSide, Settings};
use crate::test_support::TRUE_COLOR;
use crate::theme::{Theme, ThemeId};

const SCREEN_130: Rect = Rect::new(0, 0, 130, 40);
const SCREEN_200: Rect = Rect::new(0, 0, 200, 60);

fn screen_lines(terminal: &Terminal<TestBackend>) -> Vec<String> {
    let buffer = terminal.backend().buffer();
    let area = buffer.area;
    (0..area.height)
        .map(|y| (0..area.width).map(|x| buffer[(x, y)].symbol()).collect())
        .collect()
}

fn draw_docked(screen: Rect, side: DockSide, locale: Locale) -> (Vec<String>, Rect) {
    i18n::set_locale(locale);
    let caps = TRUE_COLOR;
    let settings = Settings::for_caps(&caps);
    let theme = Theme::new(ThemeId::Dark, ColorDepth::TrueColor);
    let glyphs = Glyphs::new(settings.glyph_mode);
    let ctx = ViewCtx {
        theme: &theme,
        glyphs: &glyphs,
        settings: &settings,
        caps: &caps,
        audio_status: AudioStatus::NoDevice,
        sound_styles: &[],
        saved: None,
        pointer: None,
        now: Instant::now(),
        font: crate::app::FontFacts::default(),
    };
    let rect = dock::split(screen, side).map_or(Rect::default(), |(rect, _)| rect);
    let mut terminal = Terminal::new(TestBackend::new(screen.width, screen.height))
        .unwrap_or_else(|never| match never {});
    terminal
        .draw(|frame| {
            dock_view::draw(
                frame,
                rect,
                &Modal::help(),
                &DockState::default(),
                side,
                &ctx,
            );
        })
        .unwrap_or_else(|never| match never {});
    (screen_lines(&terminal), rect)
}

fn row_of(lines: &[String], row: u16) -> &str {
    lines.get(usize::from(row)).map_or("", String::as_str)
}

#[test]
fn the_placement_label_names_the_dock_key_in_every_locale() {
    let key = primary_key(Action::Dock).unwrap_or_default();
    assert!(!key.is_empty());
    for locale in Locale::offered() {
        let (lines, rect) = draw_docked(SCREEN_130, DockSide::Right, locale);
        let label = tf(
            "help.dock.button",
            &[("side", &t("help.dock.side_right")), ("key", &key)],
        );
        let top = row_of(&lines, rect.y);
        assert!(top.contains(&label), "{}: {top:?}", locale.id());
        assert!(
            top.contains(&format!("[{key}]")),
            "{}: {top:?}",
            locale.id()
        );
    }
}

#[test]
fn the_docked_footer_shows_the_dock_and_focus_hints_at_docking_width() {
    let (lines, rect) = draw_docked(SCREEN_130, DockSide::Right, Locale::EnUs);
    let footer = row_of(&lines, dock::parts(rect).footer.y);
    let hints = dock_view::footer_hints();
    assert!(footer.contains(&hints), "{footer:?}");
    let focus = primary_key(Action::DockFocus).unwrap_or_default();
    assert!(footer.contains(&focus), "{footer:?}");
}

#[test]
fn the_docked_footer_shows_the_full_hints_in_every_locale_on_a_wide_screen() {
    for locale in Locale::offered() {
        let (lines, rect) = draw_docked(SCREEN_200, DockSide::Left, locale);
        let footer = row_of(&lines, dock::parts(rect).footer.y);
        let hints = dock_view::footer_hints();
        assert!(footer.contains(&hints), "{}: {footer:?}", locale.id());
    }
}

#[test]
fn the_close_button_stays_whole_beside_the_hints_at_the_narrowest_dock() {
    for locale in Locale::offered() {
        let (lines, rect) = draw_docked(SCREEN_130, DockSide::Right, locale);
        let footer = row_of(&lines, dock::parts(rect).footer.y);
        let close = dock::close_label();
        assert!(footer.contains(&close), "{}: {footer:?}", locale.id());
    }
}
