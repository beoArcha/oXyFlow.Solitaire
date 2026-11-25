//! The preview pane of the settings screen (O-102): it follows every look setting and fits the
//! window beside the list, under it or not at all.

use ratatui::crossterm::event::KeyCode;

use super::super::options::Field;
use super::super::tabs::Tab;
use super::{canonical_tab, focus_in_tab, open_settings, switch_to_tab};
use crate::i18n::{Locale, set_locale, t};
use crate::test_support::Harness;

const LABELS: [&str; 4] = [
    "settings.pane.back",
    "settings.pane.focus",
    "settings.pane.carried",
    "settings.pane.target",
];

const PREVIEWED: [Field; 12] = [
    Field::Theme,
    Field::CardBack,
    Field::CardFace,
    Field::FrontStyle,
    Field::BackStyle,
    Field::Glyphs,
    Field::CardSize,
    Field::AuraColour,
    Field::AuraStyle,
    Field::AuraSize,
    Field::Aura,
    Field::TargetMarks,
];

fn side_by_side() -> Harness {
    let mut h = open_settings();
    h.resize(130, 40);
    h
}

fn focus(h: &mut Harness, field: Field) {
    focus_in_tab(h, canonical_tab(field), field);
}

#[test]
fn every_look_setting_changes_the_preview() {
    for field in PREVIEWED {
        let mut h = side_by_side();
        focus(&mut h, field);
        let before = h.pane_cells();
        assert!(!before.is_empty(), "{field:?}: no preview pane");
        h.press(KeyCode::Right);
        assert_ne!(h.pane_cells(), before, "{field:?}");
    }
}

#[test]
fn the_card_size_steps_each_change_the_preview() {
    let mut h = side_by_side();
    focus(&mut h, Field::CardSize);
    let mut seen = vec![h.pane_cells()];
    for _ in 0..3 {
        h.press(KeyCode::Right);
        seen.push(h.pane_cells());
    }
    for (index, cells) in seen.iter().enumerate() {
        assert!(
            !seen[..index].contains(cells),
            "card size step {index} looks like an earlier one"
        );
    }
}

#[test]
fn a_setting_outside_the_look_leaves_the_preview_alone() {
    let mut h = side_by_side();
    focus(&mut h, Field::Volume);
    let before = h.pane_cells();
    h.press(KeyCode::Left);
    assert_eq!(h.pane_cells(), before);
}

#[test]
fn the_card_back_row_no_longer_carries_a_swatch() {
    let mut h = side_by_side();
    let line = super::line_of(&mut h, &t(Field::CardBack.label_key()));
    let value = line.trim_end().trim_end_matches(['►', '◄']).trim_end();
    assert!(value.ends_with(&*t("settings.card_back.theme")), "{value}");
}

#[test]
fn the_preview_shows_the_cards_in_the_chosen_glyphs() {
    let mut h = side_by_side();
    let text = h.text();
    for card in ["Q♥", "J♠", "K♦"] {
        assert!(text.contains(card), "{card}");
    }
    focus(&mut h, Field::Glyphs);
    h.press(KeyCode::Right);
    let text = h.text();
    assert!(!text.contains('♥') && !text.contains('♠'), "{text}");
}

#[test]
fn the_preview_stands_beside_the_list_in_a_wide_window_in_every_language() {
    for locale in Locale::offered() {
        set_locale(locale);
        for tab in Tab::ALL {
            let mut h = side_by_side();
            switch_to_tab(&mut h, tab);
            for (index, field) in tab.fields().iter().enumerate() {
                let list = h.lines_without_pane();
                let marked = format!("► {}", t(field.label_key()));
                assert!(
                    list.iter().any(|line| line.contains(&marked)),
                    "{locale:?} {tab:?} row {index}"
                );
                h.press(KeyCode::Down);
            }
            let text = h.text();
            assert!(
                text.contains(&*t("settings.pane.title")),
                "{locale:?} {tab:?}"
            );
            for key in LABELS {
                assert!(text.contains(&*t(key)), "{locale:?} {tab:?} {key}");
            }
            assert!(
                !text.contains(&*t("settings.pane.hidden")),
                "{locale:?} {tab:?}"
            );
        }
    }
    set_locale(Locale::EnUs);
}

#[test]
fn a_narrow_low_window_keeps_the_long_list_and_says_the_preview_is_hidden() {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use std::time::Instant;

    use crate::action::Action;
    use crate::audio::AudioStatus;
    use crate::caps::ColorDepth;
    use crate::glyphs::Glyphs;
    use crate::screens::ViewCtx;
    use crate::screens::settings::SettingsScreen;
    use crate::settings::Settings;
    use crate::test_support::{TRUE_COLOR, sound_styles};
    use crate::theme::Theme;

    for locale in Locale::offered() {
        set_locale(locale);
        let settings = Settings {
            locale,
            ..Settings::default()
        };
        let theme = Theme::new(settings.theme, ColorDepth::TrueColor);
        let glyphs = Glyphs::new(settings.glyph_mode);
        let sounds = sound_styles();
        let ctx = ViewCtx {
            theme: &theme,
            glyphs: &glyphs,
            settings: &settings,
            caps: &TRUE_COLOR,
            audio_status: AudioStatus::Ready,
            sound_styles: &sounds,
            saved: None,
            pointer: None,
            now: Instant::now(),
            font: crate::app::FontFacts::default(),
        };
        let mut screen = SettingsScreen::new();
        screen.handle(
            &ratatui::crossterm::event::KeyEvent::new(
                KeyCode::BackTab,
                ratatui::crossterm::event::KeyModifiers::NONE,
            ),
            Some(Action::FocusPrevious),
            &ctx,
        );
        let mut terminal =
            Terminal::new(TestBackend::new(80, 30)).unwrap_or_else(|never| match never {});
        terminal
            .draw(|frame| screen.draw(frame, frame.area(), &ctx))
            .unwrap_or_else(|never| match never {});
        let buffer = terminal.backend().buffer();
        let text: String = (0..30)
            .map(|y| (0..80).map(|x| buffer[(x, y)].symbol()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains(&*t("settings.pane.hidden")), "{locale:?}");
        let title = t("settings.pane.title");
        assert!(text.lines().all(|line| line.trim() != title), "{locale:?}");
    }
    set_locale(Locale::EnUs);
}

#[test]
fn a_narrow_high_window_from_100_by_52_shows_the_preview_on_every_tab() {
    for locale in Locale::offered() {
        set_locale(locale);
        for tab in Tab::ALL {
            let mut h = open_settings();
            h.resize(100, 52);
            switch_to_tab(&mut h, tab);
            let last_field = *tab.fields().last().expect("non-empty tab");
            focus_in_tab(&mut h, tab, last_field);
            let lines = h.lines();
            let title = lines
                .iter()
                .position(|line| line.contains(&*t("settings.pane.title")));
            let last_row = lines
                .iter()
                .rposition(|line| line.contains(&*t(last_field.label_key())));
            assert!(title.is_some() && last_row.is_some(), "{locale:?} {tab:?}");
            if tab == Tab::Gameplay {
                assert!(title > last_row, "{locale:?} {tab:?}");
            }
            let text = lines.join("\n");
            assert!(
                !text.contains(&*t("settings.pane.hidden")),
                "{locale:?} {tab:?}"
            );
            for key in LABELS {
                assert!(text.contains(&*t(key)), "{locale:?} {tab:?} {key}");
            }
        }
    }
    set_locale(Locale::EnUs);
}
