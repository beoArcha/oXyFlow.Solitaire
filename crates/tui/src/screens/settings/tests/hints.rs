//! The Gameplay tab: Language and the board aids a new game starts with.

use ratatui::crossterm::event::KeyCode;

use super::super::options::Field;
use super::super::tabs::Tab;
use super::{bare, fitting_size, focus_in_tab, line_of, open_settings, switch_to_tab};
use crate::i18n::{Locale, set_locale, t};
use crate::screens::Effect;
use crate::settings::{Aid, Settings};

#[test]
fn the_aid_rows_and_then_the_game_options_sit_in_the_gameplay_tab_in_this_order() {
    let gameplay = Tab::Gameplay.fields();
    let aids: Vec<Aid> = gameplay.iter().filter_map(|field| field.aid()).collect();
    assert_eq!(aids, Aid::ALL);
    assert_eq!(gameplay.first(), Some(&Field::Language));
    let game_options = [
        Field::QuickMove,
        Field::KlondikePasses,
        Field::KlondikeEmptyColumn,
        Field::SpiderDeal,
        Field::GolfWrap,
        Field::GolfKingStops,
    ];
    assert!(gameplay.ends_with(&game_options));
    assert!(
        Tab::AudioVisual
            .fields()
            .iter()
            .all(|field| field.aid().is_none())
    );
}

#[test]
fn the_gameplay_tab_is_named_in_both_locales_and_the_old_board_heading_is_gone() {
    let mut h = open_settings();
    let (width, height) = fitting_size();
    h.resize(width, height);
    switch_to_tab(&mut h, Tab::Gameplay);
    let text = h.text();
    assert!(text.contains("[Gameplay]"), "{text}");
    assert!(!text.contains("Board"), "{text}");
    h.app.apply(Effect::Apply(Settings {
        locale: Locale::PlPl,
        ..Settings::default()
    }));
    assert!(h.text().contains("[Rozgrywka]"), "{}", h.text());
    set_locale(Locale::EnUs);
}

#[test]
fn every_aid_row_switches_by_key_and_by_click_and_is_saved() {
    for aid in Aid::ALL {
        let expected = aid.toggled(&Settings::default().aids());
        let mut h = open_settings();
        h.resize(140, 40);
        let field = Tab::Gameplay
            .fields()
            .iter()
            .copied()
            .find(|f| f.aid() == Some(aid))
            .expect("aid in Gameplay tab");
        focus_in_tab(&mut h, Tab::Gameplay, field);
        h.press(KeyCode::Enter);
        assert_eq!(h.app.settings().aids(), expected, "{aid:?} by key");
        let saved = h.storage.borrow().settings.clone();
        assert!(saved.is_some_and(|s| s.aids() == expected), "{aid:?} saved");
        h.press(KeyCode::Left);
        assert_eq!(
            h.app.settings().aids(),
            Settings::default().aids(),
            "{aid:?}"
        );

        let label = t(aid.label_key()).into_owned();
        let Some((column, row)) = h.find(&format!("{label} .")) else {
            panic!("{label} is not drawn");
        };
        h.click(column + 1, row);
        assert_eq!(h.app.settings().aids(), expected, "{aid:?} by click");
    }
}

#[test]
fn the_focused_aid_row_is_explained_in_every_locale() {
    for locale in Locale::offered() {
        for aid in Aid::ALL {
            let mut h = open_settings();
            h.app.apply(Effect::Apply(Settings {
                locale,
                ..Settings::default()
            }));
            let field = Tab::Gameplay
                .fields()
                .iter()
                .copied()
                .find(|f| f.aid() == Some(aid))
                .expect("aid in Gameplay tab");
            focus_in_tab(&mut h, Tab::Gameplay, field);
            let text = h.text();
            assert!(
                text.contains(t(aid.description_key()).as_ref()),
                "{locale:?} {aid:?}"
            );
        }
    }
    set_locale(Locale::EnUs);
}

#[test]
fn target_marks_and_the_moves_panel_have_their_own_rows() {
    let mut h = open_settings();
    h.resize(140, 46);
    switch_to_tab(&mut h, Tab::Gameplay);
    assert!(bare(&line_of(&mut h, "Target marks")).ends_with("On"));
    assert!(bare(&line_of(&mut h, "Possible moves panel")).ends_with("On"));
    focus_in_tab(&mut h, Tab::Gameplay, Field::TargetMarks);
    h.press(KeyCode::Enter);
    assert!(!h.app.settings().target_marks);
    assert!(bare(&line_of(&mut h, "Target marks")).ends_with("Off"));
}
