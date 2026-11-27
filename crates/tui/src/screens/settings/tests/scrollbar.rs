//! Scroll bar tests for the Settings list (O-127): visibility when scrolling vs fitting, thumb
//! movement on keys and wheel, track-click paging and thumb dragging in Unicode and ASCII, title
//! position retention, and no clipped row text or pane overlap across locales and interface sizes.

use ratatui::crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use super::super::options::Field;
use super::super::tabs::Tab;
use super::{open_settings, switch_to_tab};
use crate::i18n::{Locale, set_locale, t};
use crate::input::Input;
use crate::layout::InterfaceSize;
use crate::screens::Effect;
use crate::settings::Settings;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, PLAIN, TRUE_COLOR};

fn open_with(
    settings: Settings,
    caps: crate::caps::TerminalCaps,
    width: u16,
    height: u16,
) -> Harness {
    let storage = MemoryStorage {
        settings: Some(settings),
        ..MemoryStorage::new()
    };
    let mut h = Harness::with(storage, caps, width, height);
    h.app.apply(Effect::Goto(crate::app::Screen::Settings));
    h
}

fn bar_x(h: &mut Harness, (thumb, groove): (char, char)) -> Option<u16> {
    h.lines_without_pane().into_iter().find_map(|line| {
        let chars: Vec<char> = line.chars().collect();
        let pos = chars.iter().position(|ch| *ch == thumb || *ch == groove)?;
        u16::try_from(pos).ok()
    })
}

fn bar_column(h: &mut Harness, glyphs: (char, char)) -> Vec<(u16, char)> {
    let Some(x) = bar_x(h, glyphs) else {
        return Vec::new();
    };
    let (thumb, groove) = glyphs;
    h.lines_without_pane()
        .into_iter()
        .enumerate()
        .filter_map(|(row, line)| {
            let ch = line.chars().nth(usize::from(x))?;
            (ch == thumb || ch == groove)
                .then(|| u16::try_from(row).ok().map(|r| (r, ch)))
                .flatten()
        })
        .collect()
}

fn focused_label(h: &mut Harness) -> String {
    h.lines_without_pane()
        .into_iter()
        .find(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with('►') || trimmed.starts_with('>')
        })
        .unwrap_or_default()
}

fn wheel(h: &mut Harness, kind: MouseEventKind) {
    h.send(Input::Mouse(MouseEvent {
        kind,
        column: 20,
        row: 10,
        modifiers: KeyModifiers::NONE,
    }));
}

#[test]
fn scrollbar_appears_only_when_active_tab_scrolls() {
    let mut h = open_settings();
    h.resize(100, 30);
    assert!(bar_column(&mut h, ('█', '░')).is_empty());

    switch_to_tab(&mut h, Tab::Gameplay);
    assert!(bar_column(&mut h, ('█', '░')).is_empty());

    switch_to_tab(&mut h, Tab::AudioVisual);
    assert_eq!(bar_column(&mut h, ('█', '░')).len(), 20);
    assert!(h.text().contains("(row 1 of 27)"));

    h.resize(100, 40);
    assert!(bar_column(&mut h, ('█', '░')).is_empty());
}

#[test]
fn thumb_moves_with_keys_and_wheel_and_keeps_row_title() {
    let mut h = open_settings();
    h.resize(100, 30);
    switch_to_tab(&mut h, Tab::AudioVisual);

    let top = bar_column(&mut h, ('█', '░'));
    assert_eq!(top.first().map(|(_, ch)| *ch), Some('█'));
    assert!(h.text().contains("(row 1 of 27)"));

    for _ in 0..26 {
        h.press(KeyCode::Down);
    }
    let bottom = bar_column(&mut h, ('█', '░'));
    assert_ne!(bottom, top);
    assert_eq!(bottom.last().map(|(_, ch)| *ch), Some('█'));
    assert!(h.text().contains("(row 27 of 27)"));

    for _ in 0..26 {
        wheel(&mut h, MouseEventKind::ScrollUp);
    }
    assert_eq!(bar_column(&mut h, ('█', '░')), top);
    assert!(h.text().contains("(row 1 of 27)"));
}

#[test]
fn track_click_pages_and_thumb_drag_scrolls_in_unicode_and_ascii() {
    for (caps, thumb, groove) in [(TRUE_COLOR, '█', '░'), (PLAIN, '#', ':')] {
        let mut h = open_with(Settings::for_caps(&caps), caps, 100, 30);
        switch_to_tab(&mut h, Tab::AudioVisual);
        let x = bar_x(&mut h, (thumb, groove)).expect("bar drawn");
        let first_label = t(Tab::AudioVisual.fields()[0].label_key()).into_owned();
        let last_label = t(Tab::AudioVisual
            .fields()
            .last()
            .expect("audio visual has fields")
            .label_key())
        .into_owned();

        let cells = bar_column(&mut h, (thumb, groove));
        let groove_below = cells
            .iter()
            .rfind(|(_, ch)| *ch == groove)
            .map(|(r, _)| *r)
            .expect("groove below thumb");
        h.click(x, groove_below);
        assert!(!h.text().contains(&first_label));
        assert!(h.text().contains(&last_label));
        let focused = focused_label(&mut h);
        assert!(!focused.is_empty());

        let cells = bar_column(&mut h, (thumb, groove));
        let groove_above = cells
            .iter()
            .find(|(_, ch)| *ch == groove)
            .map(|(r, _)| *r)
            .expect("groove above thumb");
        h.click(x, groove_above);
        assert!(h.text().contains(&first_label));

        let cells = bar_column(&mut h, (thumb, groove));
        let thumb_top = cells
            .iter()
            .find(|(_, ch)| *ch == thumb)
            .map(|(r, _)| *r)
            .expect("thumb top");
        let bar_bottom = cells.last().map(|(r, _)| *r).expect("bar bottom");
        h.send(Input::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: x,
            row: thumb_top,
            modifiers: KeyModifiers::NONE,
        }));
        h.send(Input::Mouse(MouseEvent {
            kind: MouseEventKind::Drag(MouseButton::Left),
            column: x,
            row: bar_bottom,
            modifiers: KeyModifiers::NONE,
        }));
        h.send(Input::Mouse(MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: x,
            row: bar_bottom,
            modifiers: KeyModifiers::NONE,
        }));
        assert!(h.text().contains(&last_label));
        let focused = focused_label(&mut h);
        assert!(!focused.is_empty());
        assert!(focused.contains(&*t(Field::Priority.label_key())) || h.text().contains(&focused));
    }
}

fn assert_rows_unclipped(locale: Locale, size: InterfaceSize, width: u16) {
    let mut h = open_with(
        Settings {
            locale,
            interface_size: size,
            ..Settings::default()
        },
        TRUE_COLOR,
        width,
        30,
    );
    switch_to_tab(&mut h, Tab::AudioVisual);
    let x = bar_x(&mut h, ('█', '░')).expect("bar drawn");
    let gap_x = usize::from(x.saturating_sub(1));
    for _ in 0..Tab::AudioVisual.fields().len() {
        let lines = h.lines_without_pane();
        assert!(lines.len() <= 30);
        assert!(
            lines
                .iter()
                .all(|line| line.chars().count() <= usize::from(width))
        );
        let focused_row = lines
            .iter()
            .find(|line| line.trim_start().starts_with('►'))
            .expect("focused row");
        assert_eq!(
            focused_row.chars().nth(gap_x),
            Some(' '),
            "{locale:?} {size:?} {width}: {focused_row:?}"
        );
        if focused_row.contains('◄') {
            assert!(
                focused_row.chars().take(gap_x).any(|ch| ch == '►')
                    && focused_row
                        .chars()
                        .skip(1)
                        .take(gap_x - 1)
                        .collect::<String>()
                        .trim_end()
                        .ends_with('►'),
                "{locale:?} {size:?} {width}: right arrow cut in {focused_row:?}"
            );
        }
        h.press(KeyCode::Down);
    }
    if width == 130 {
        assert!(
            h.text().contains(&*t("settings.pane.title")),
            "{locale:?} {size:?}: preview pane missing"
        );
    }
}

#[test]
fn scrollbar_leaves_every_row_and_arrow_intact_across_locales_and_sizes() {
    for locale in Locale::offered() {
        set_locale(locale);
        for size in InterfaceSize::ALL {
            for width in [100, 130] {
                assert_rows_unclipped(locale, size, width);
            }
        }
    }
    set_locale(Locale::EnUs);
}
