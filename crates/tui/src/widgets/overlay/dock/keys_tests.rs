use std::time::Instant;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use super::super::dock_view;
use super::super::{self as dock, DockState};
use crate::action::Action;
use crate::app::Modal;
use crate::audio::AudioStatus;
use crate::caps::ColorDepth;
use crate::glyphs::Glyphs;
use crate::i18n::{self, Locale, t};
use crate::input::primary_key;
use crate::screens::ViewCtx;
use crate::settings::{DockSide, Settings};
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, TRUE_COLOR};
use crate::theme::{Theme, ThemeId, Token};
use crate::widgets::overlay::help;
use crate::widgets::overlay::how_to_play::content::Kind;

const SIZES: [Rect; 4] = [
    Rect::new(0, 0, 130, 30),
    Rect::new(0, 0, 130, 40),
    Rect::new(0, 0, 160, 45),
    Rect::new(0, 0, 200, 60),
];
const KEY_COLUMN: usize = 12;
const KEY_GAP: usize = 2;

fn render_docked(
    screen: Rect,
    side: DockSide,
    locale: Locale,
    scroll: u16,
) -> (Terminal<TestBackend>, Rect) {
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
    let state = DockState {
        focused: true,
        scroll,
    };
    let mut terminal = Terminal::new(TestBackend::new(screen.width, screen.height))
        .unwrap_or_else(|never| match never {});
    terminal
        .draw(|frame| dock_view::draw(frame, rect, &Modal::help(), &state, side, &ctx))
        .unwrap_or_else(|never| match never {});
    (terminal, rect)
}

fn dock_lines(terminal: &Terminal<TestBackend>, rect: Rect) -> Vec<String> {
    let buffer = terminal.backend().buffer();
    (rect.y..rect.bottom())
        .map(|y| {
            (rect.x..rect.right())
                .map(|x| buffer[(x, y)].symbol())
                .collect()
        })
        .collect()
}

#[test]
fn short_keys_and_descriptions_share_one_row_while_long_keys_wrap() {
    i18n::set_locale(Locale::EnUs);
    let narrow = dock_view::body(&Modal::help(), 26);
    let stock_key = primary_key(Action::Stock).unwrap_or_default();
    let stock_desc = t(Action::Stock.label_key());
    let stock_row = narrow
        .iter()
        .find(|row| row.kind == Kind::Key && row.lead.trim_end() == stock_key)
        .expect("stock row");
    assert_eq!(stock_row.text, stock_desc.as_ref());

    let select_row = help::sections()
        .into_iter()
        .flat_map(|section| section.rows)
        .find(|row| row.text == t(Action::Select.label_key()).as_ref())
        .expect("select row");
    assert!(select_row.keys.chars().count() + KEY_GAP > KEY_COLUMN);
    let select_index = narrow
        .iter()
        .position(|row| row.kind == Kind::Key && row.lead.trim_end() == select_row.keys)
        .expect("select key line");
    assert!(narrow[select_index].text.is_empty());
    assert_eq!(narrow[select_index + 1].kind, Kind::Body);
    assert_eq!(narrow[select_index + 1].text, select_row.text);

    let zoom_key = t("help.display.terminal_keys");
    let zoom_index = narrow
        .iter()
        .position(|row| row.kind == Kind::Key && row.lead.trim_end() == zoom_key.as_ref())
        .expect("terminal zoom key");
    assert!(!narrow[zoom_index].text.is_empty());
    assert_eq!(narrow[zoom_index + 1].kind, Kind::Body);
    assert!(narrow[zoom_index + 1].lead.chars().all(|ch| ch == ' '));

    let wide = dock_view::body(&Modal::help(), 56);
    let wide_zoom = wide
        .iter()
        .find(|row| row.kind == Kind::Key && row.lead.trim_end() == zoom_key.as_ref())
        .expect("wide zoom key");
    assert_eq!(wide_zoom.text, t("help.display.terminal_text").as_ref());
}

#[test]
fn docked_keys_uses_fewer_rows_than_two_line_layout_in_all_locales() {
    for locale in Locale::offered() {
        i18n::set_locale(locale);
        let sections = help::sections();
        let two_line_rows: usize = sections
            .iter()
            .map(|section| 1 + section.rows.len() * 2)
            .sum::<usize>()
            + sections.len().saturating_sub(1);
        for width in [26_u16, 56] {
            let rows = dock_view::body(&Modal::help(), width);
            assert!(
                rows.len() < two_line_rows,
                "{} at width {width}: {} >= {two_line_rows}",
                locale.id(),
                rows.len()
            );
        }
    }
    i18n::set_locale(Locale::EnUs);
}

#[test]
fn every_docked_keys_row_fits_every_allowed_width_in_all_locales() {
    for locale in Locale::offered() {
        i18n::set_locale(locale);
        for width in 26_u16..=56 {
            for row in dock_view::body(&Modal::help(), width) {
                let used = row.lead.chars().count() + row.text.chars().count();
                assert!(
                    used <= usize::from(width),
                    "{} width {width}: {used} > {width} in {row:?}",
                    locale.id()
                );
            }
        }
    }
    i18n::set_locale(Locale::EnUs);
}

#[test]
fn all_locales_render_every_key_and_description_without_clipping_at_docked_sizes() {
    for locale in Locale::offered() {
        for screen in [SIZES[0], SIZES[3]] {
            for side in [DockSide::Left, DockSide::Right] {
                i18n::set_locale(locale);
                let rect = dock::split(screen, side).expect("dockable").0;
                let max = dock::max_scroll(&Modal::help(), rect);
                let mut seen = Vec::new();
                for scroll in 0..=max {
                    let (terminal, drawn) = render_docked(screen, side, locale, scroll);
                    seen.extend(dock_lines(&terminal, drawn));
                }
                let joined = seen.join("\n");
                for section in help::sections() {
                    assert!(
                        joined.contains(&section.title),
                        "{} {screen:?} {side:?}: missing section {}",
                        locale.id(),
                        section.title
                    );
                    for row in section.rows {
                        assert!(
                            joined.contains(&row.keys),
                            "{} {screen:?} {side:?}: missing key {}",
                            locale.id(),
                            row.keys
                        );
                        for word in row.text.split_whitespace() {
                            assert!(
                                joined.contains(word),
                                "{} {screen:?} {side:?}: clipped word {word:?} from {:?}",
                                locale.id(),
                                row.text
                            );
                        }
                    }
                }
            }
        }
    }
    i18n::set_locale(Locale::EnUs);
}

#[test]
fn rendered_row_uses_theme_tokens_and_places_short_key_and_description_together() {
    let theme = Theme::new(ThemeId::Dark, ColorDepth::TrueColor);
    let accent = theme.color(Token::Accent);
    let text = theme.color(Token::Text);
    for screen in SIZES {
        for side in [DockSide::Left, DockSide::Right] {
            let (terminal, rect) = render_docked(screen, side, Locale::EnUs, 0);
            let lines = dock_lines(&terminal, rect);
            let stock_key = primary_key(Action::Stock).unwrap_or_default();
            let stock_desc = t(Action::Stock.label_key());
            let row_index = lines
                .iter()
                .position(|line| line.contains(&stock_key) && line.contains(stock_desc.as_ref()))
                .expect("stock key and description on one rendered line");
            let line = &lines[row_index];
            let key_byte = line.find(&stock_key).expect("key col");
            let desc_byte = line.find(stock_desc.as_ref()).expect("desc col");
            let key_col = line[..key_byte].chars().count();
            let desc_col = line[..desc_byte].chars().count();
            let buffer = terminal.backend().buffer();
            let y = rect.y + u16::try_from(row_index).unwrap_or(0);
            assert_eq!(
                buffer[(rect.x + u16::try_from(key_col).unwrap_or(0), y)].fg,
                accent
            );
            assert_eq!(
                buffer[(rect.x + u16::try_from(desc_col).unwrap_or(0), y)].fg,
                text
            );
        }
    }
}

#[test]
fn docked_help_scrolls_by_keyboard_and_mouse_and_closes_at_130_by_30() {
    for (width, height) in [(130, 30), (200, 60)] {
        let mut h = Harness::with(MemoryStorage::new(), TRUE_COLOR, width, height);
        h.press(KeyCode::Enter);
        h.press(KeyCode::Enter);
        h.lines();
        h.type_char('?');
        h.type_char('d');
        h.press(KeyCode::F(6));
        assert!(h.text().contains(&*t("help.group.navigation")));
        h.press(KeyCode::End);
        assert!(h.text().contains(&*t("help.mouse_keys.drag")));
        h.press(KeyCode::Home);
        assert!(h.text().contains(&*t("help.group.navigation")));

        let rect = dock::split(Rect::new(0, 0, width, height), DockSide::Right)
            .expect("dock")
            .0;
        h.send(crate::input::Input::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: rect.x + 2,
            row: rect.y + 4,
            modifiers: ratatui::crossterm::event::KeyModifiers::NONE,
        }));
        assert!(h.text().contains(&*t("help.group.board")));
        let close = dock::close_rect(rect);
        h.click(close.x, close.y);
        assert!(h.app.modal().is_none());
    }
}
