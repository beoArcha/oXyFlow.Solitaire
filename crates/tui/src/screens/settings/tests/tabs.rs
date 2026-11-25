//! Tests for the three settings tabs (O-103): Popular active on open, canonical tabs,
//! alias <-> canonical synchronization, Tab/Shift+Tab vs Left/Right, mouse and keyboard reachability,
//! active-tab styling, and 100 x 30 fit in all nine locales.

use std::collections::BTreeSet;

use ratatui::crossterm::event::KeyCode;
use ratatui::style::Modifier;

use super::super::options::{FIELDS, Field};
use super::super::tabs::Tab;
use super::{bare, canonical_tab, focus_in_tab, line_of, open_settings, switch_to_tab};
use crate::caps::ColorDepth;
use crate::glyphs::is_wgl4;
use crate::i18n::{Locale, set_locale, t};
use crate::settings::Settings;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, PLAIN, TRUE_COLOR};
use crate::theme::{Theme, ThemeId};

fn active_tab_label(tab: Tab) -> String {
    format!("[{}]", t(tab.label_key()))
}

fn tab_bar_arrows(h: &mut Harness) -> Option<((u16, u16), (u16, u16))> {
    let lines = h.lines();
    let needle = active_tab_label(Tab::Popular);
    let row = lines.iter().position(|line| {
        Tab::ALL
            .iter()
            .any(|tab| line.contains(&active_tab_label(*tab)))
    })?;
    let _ = needle;
    let chars: Vec<char> = lines.get(row)?.chars().collect();
    let left = chars.iter().position(|c| *c == '◄')?;
    let right = chars.iter().rposition(|c| *c == '►')?;
    Some((
        (u16::try_from(left).ok()?, u16::try_from(row).ok()?),
        (u16::try_from(right).ok()?, u16::try_from(row).ok()?),
    ))
}

#[test]
fn popular_is_active_on_open_in_the_middle() {
    set_locale(Locale::EnUs);
    let mut h = open_settings();
    let text = h.text();
    let line = text
        .lines()
        .find(|line| line.contains("[Popular]"))
        .expect("tab bar with [Popular]");
    assert!(
        line.contains("◄  Audio & Visual  │ [Popular] │  Gameplay  ►"),
        "{line}"
    );
}

#[test]
fn tab_and_shift_tab_switch_tabs_and_left_right_never_do() {
    set_locale(Locale::EnUs);
    let mut h = open_settings();
    assert!(h.text().contains(&active_tab_label(Tab::Popular)));

    h.press(KeyCode::Tab);
    assert!(h.text().contains(&active_tab_label(Tab::Gameplay)));

    h.press(KeyCode::Tab);
    assert!(h.text().contains(&active_tab_label(Tab::AudioVisual)));

    h.press(KeyCode::Tab);
    assert!(h.text().contains(&active_tab_label(Tab::Popular)));

    h.press(KeyCode::BackTab);
    assert!(h.text().contains(&active_tab_label(Tab::AudioVisual)));

    h.press(KeyCode::BackTab);
    assert!(h.text().contains(&active_tab_label(Tab::Gameplay)));

    h.press(KeyCode::BackTab);
    assert!(h.text().contains(&active_tab_label(Tab::Popular)));

    h.type_char(']');
    assert!(h.text().contains(&active_tab_label(Tab::Gameplay)));
    h.type_char('[');
    assert!(h.text().contains(&active_tab_label(Tab::Popular)));

    let before = h.app.settings().clone();
    h.press(KeyCode::Right);
    assert_ne!(h.app.settings(), &before);
    assert!(h.text().contains(&active_tab_label(Tab::Popular)));

    h.press(KeyCode::Left);
    assert_eq!(h.app.settings(), &before);
    assert!(h.text().contains(&active_tab_label(Tab::Popular)));
}

#[test]
fn every_setting_is_reachable_in_its_canonical_tab_by_keyboard_and_mouse() {
    set_locale(Locale::EnUs);
    for expected_av in [Field::ColorVision, Field::FourColorDeck, Field::Priority] {
        assert!(Tab::AudioVisual.fields().contains(&expected_av));
    }
    assert!(Tab::Gameplay.fields().contains(&Field::QuickMove));

    let mut seen = BTreeSet::new();
    let mut h = open_settings();
    h.resize(130, 40);
    for tab in [Tab::AudioVisual, Tab::Gameplay] {
        switch_to_tab(&mut h, tab);
        for field in tab.fields() {
            let marked = format!("► {}", t(field.label_key()));
            let list = h.lines_without_pane();
            assert!(
                list.iter().any(|line| line.contains(&marked)),
                "{tab:?} {field:?}"
            );
            seen.insert(t(field.label_key()).into_owned());
            h.press(KeyCode::Down);
        }
    }
    assert_eq!(seen.len(), FIELDS.len());

    let mut mouse = open_settings();
    mouse.resize(130, 40);
    for tab in Tab::ALL {
        let label = format!(" {} ", t(tab.label_key()));
        if let Some((col, row)) = mouse.find(&label) {
            mouse.click(col + 1, row);
        }
        assert!(
            mouse.text().contains(&active_tab_label(tab)),
            "clicking {tab:?}"
        );
    }

    let (left, _) = tab_bar_arrows(&mut mouse).expect("tab arrows");
    mouse.click(left.0, left.1);
    assert!(mouse.text().contains(&active_tab_label(Tab::Popular)));
    let (_, right) = tab_bar_arrows(&mut mouse).expect("tab arrows");
    mouse.click(right.0, right.1);
    assert!(mouse.text().contains(&active_tab_label(Tab::Gameplay)));
}

#[test]
fn popular_aliases_and_canonical_rows_sync_both_ways() {
    set_locale(Locale::EnUs);
    for field in Tab::Popular.fields().iter().copied() {
        let canonical = canonical_tab(field);
        let mut h = open_settings();
        h.resize(130, 40);

        focus_in_tab(&mut h, Tab::Popular, field);
        let label = t(field.label_key()).into_owned();
        let initial_line = bare(&line_of(&mut h, &label)).trim_start().to_owned();
        h.press(KeyCode::Right);
        let updated_popular = bare(&line_of(&mut h, &label)).trim_start().to_owned();
        assert_ne!(updated_popular, initial_line, "{field:?} popular changed");

        focus_in_tab(&mut h, canonical, field);
        let canonical_line = bare(&line_of(&mut h, &label)).trim_start().to_owned();
        assert_eq!(
            canonical_line, updated_popular,
            "{field:?} popular -> canonical"
        );

        h.press(KeyCode::Left);
        let reverted_canonical = bare(&line_of(&mut h, &label)).trim_start().to_owned();
        assert_eq!(
            reverted_canonical, initial_line,
            "{field:?} canonical reverted"
        );

        focus_in_tab(&mut h, Tab::Popular, field);
        let popular_after = bare(&line_of(&mut h, &label)).trim_start().to_owned();
        assert_eq!(
            popular_after, initial_line,
            "{field:?} canonical -> popular"
        );
        set_locale(Locale::EnUs);
    }
}

fn check_locale_fit_at_100_by_30(locale: Locale, caps: crate::caps::TerminalCaps) {
    let mut storage = MemoryStorage::new();
    storage.settings = Some(Settings {
        locale,
        ..Settings::for_caps(&caps)
    });
    let mut h = Harness::with(storage, caps, 100, 30);
    for _ in 0..super::settings_menu_index() {
        h.press(KeyCode::Down);
    }
    h.press(KeyCode::Enter);
    for tab in Tab::ALL {
        switch_to_tab(&mut h, tab);
        for field in tab.fields() {
            let lines = h.lines();
            assert!(lines.len() <= 30, "{locale:?} {tab:?}");
            assert!(
                lines.iter().all(|line| line.chars().count() <= 100),
                "{locale:?} {tab:?} overflow"
            );
            let text = lines.join("\n");
            assert!(
                text.contains(&active_tab_label(tab)),
                "{locale:?} {tab:?} missing active tab"
            );
            assert!(
                text.contains(&*t(field.label_key())),
                "{locale:?} {tab:?} missing {field:?}"
            );
            h.press(KeyCode::Down);
        }
    }
}

#[test]
fn all_nine_locales_fit_at_100_by_30_across_all_three_tabs() {
    assert_eq!(Locale::ALL.len(), 9);
    for locale in Locale::ALL {
        set_locale(locale);
        for caps in [TRUE_COLOR, PLAIN] {
            check_locale_fit_at_100_by_30(locale, caps);
        }
    }
    set_locale(Locale::EnUs);
}

fn check_active_tab_style(theme_id: ThemeId, unicode: bool) {
    let caps = if unicode { TRUE_COLOR } else { PLAIN };
    let mut storage = MemoryStorage::new();
    storage.settings = Some(Settings {
        theme: theme_id,
        ..Settings::for_caps(&caps)
    });
    let mut h = Harness::with(storage, caps, 100, 30);
    for _ in 0..super::settings_menu_index() {
        h.press(KeyCode::Down);
    }
    h.press(KeyCode::Enter);

    let lines = h.lines();
    let needle = "[Popular]";
    let (col, row) = h.find(needle).expect("[Popular] drawn");
    let buf = h.terminal.backend().buffer();
    let theme = Theme::new(theme_id, ColorDepth::TrueColor);
    let expected = theme.active_tab();
    for offset in 0..u16::try_from(needle.len()).unwrap_or(0) {
        let cell = &buf[(col + offset, row)];
        assert!(
            cell.modifier
                .contains(Modifier::BOLD | Modifier::UNDERLINED)
        );
        assert!(!cell.modifier.contains(Modifier::REVERSED));
        if unicode {
            assert_eq!(Some(cell.fg), expected.fg);
            assert_eq!(Some(cell.bg), expected.bg);
        }
    }
    for ch in lines.iter().flat_map(|line| line.chars()) {
        if unicode {
            assert!(is_wgl4(ch), "{theme_id:?}: non-WGL4 {ch:?}");
        } else {
            assert!(ch.is_ascii(), "{theme_id:?}: non-ASCII {ch:?}");
        }
    }
}

#[test]
fn active_tab_uses_active_tab_style_without_color_across_themes_and_glyphs() {
    set_locale(Locale::EnUs);
    for theme_id in ThemeId::ALL {
        for unicode in [true, false] {
            check_active_tab_style(theme_id, unicode);
        }
    }
}
