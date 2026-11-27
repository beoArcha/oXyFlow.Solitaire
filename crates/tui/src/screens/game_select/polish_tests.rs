//! The wording, the empty rows, the winnable-deals key and the footer of the game selection screen.
//! The expected text is read from the locale; only the en-US spelling check names English words.

use crate::action::Action;
use crate::app::Screen;
use crate::i18n::{Locale, set_locale, t, tf};
use crate::input::primary_key;
use crate::screens::Effect;
use crate::settings::Settings;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{Harness, TRUE_COLOR};

const SIZE: (u16, u16) = (100, 30);

fn select(locale: Locale) -> Harness {
    set_locale(locale);
    let storage = MemoryStorage {
        settings: Some(Settings {
            locale,
            ..Settings::for_caps(&TRUE_COLOR)
        }),
        ..MemoryStorage::new()
    };
    let mut h = Harness::with(storage, TRUE_COLOR, SIZE.0, SIZE.1);
    h.app.apply(Effect::Goto(Screen::GameSelect));
    assert_eq!(h.app.screen(), Screen::GameSelect);
    h
}

fn lines_of(h: &mut Harness) -> Vec<String> {
    h.text().lines().map(str::to_owned).collect()
}

fn description_of_first_variant() -> String {
    let level = t("menu.difficulty.easy").into_owned();
    let description = t("menu.game_hint.klondike").into_owned();
    tf(
        "menu.variant_hint",
        &[
            ("level", level.as_str()),
            ("description", description.as_str()),
        ],
    )
}

fn winnable_button(state: &str) -> String {
    tf("menu.winnable.button", &[("key", "w"), ("state", state)])
}

fn footer_entries() -> Vec<String> {
    vec![
        format!(
            "{} {}",
            primary_key(Action::Select).unwrap_or_default(),
            t("menu.footer.play")
        ),
        format!(
            "{} {}",
            primary_key(Action::Back).unwrap_or_default(),
            t("help.action.back")
        ),
        format!(
            "{} {}",
            primary_key(Action::Finish).unwrap_or_default(),
            t("menu.footer.favorite")
        ),
        format!(
            "{} {}",
            primary_key(Action::Undo).unwrap_or_default(),
            t("help.action.undo")
        ),
        format!("/ {}", t("menu.search.label")),
        format!("w {}", t("menu.footer.winnable")),
    ]
}

#[test]
fn en_us_spells_favorites_with_the_american_spelling() {
    let mut h = select(Locale::EnUs);
    assert_eq!(t("menu.favourites"), "Favorites");
    assert!(t("menu.legend").contains("favorite ({key})"));
    assert!(h.text().contains("favorite (f)"));
}

#[test]
fn the_description_has_an_empty_row_above_it() {
    let mut h = select(Locale::EnUs);
    let lines = lines_of(&mut h);
    let description = description_of_first_variant();
    let at = lines
        .iter()
        .position(|line| line.contains(&description))
        .unwrap_or_default();
    assert!(at >= 2, "the description is not drawn: {lines:?}");
    assert!(lines[at - 1].trim().is_empty(), "no empty row above it");
    assert!(
        !lines[at - 2].trim().is_empty(),
        "the empty row is not below the list"
    );
}

#[test]
fn an_empty_row_separates_the_favorites_from_the_first_family() {
    let mut h = select(Locale::EnUs);
    h.type_char('f');
    fn row_text(line: &str) -> &str {
        let trimmed = line.trim();
        trimmed
            .strip_suffix(['█', '░', '#', ':'])
            .map_or(trimmed, str::trim_end)
    }
    let lines = lines_of(&mut h);
    let heading = lines
        .iter()
        .position(|line| row_text(line) == format!("[-] {}", t("menu.favourites")))
        .unwrap_or_default();
    assert!(heading > 0, "the favorites heading is not drawn: {lines:?}");
    assert!(!row_text(&lines[heading + 1]).is_empty());
    assert!(
        row_text(&lines[heading + 2]).is_empty(),
        "no empty row below"
    );
    assert_eq!(
        row_text(&lines[heading + 3]),
        format!("[-] {}", t("menu.family.klondike")),
        "the first family does not follow the empty row"
    );
}

#[test]
fn the_w_key_switches_the_winnable_deals_option() {
    let mut h = select(Locale::EnUs);
    let off = t("common.select.off").into_owned();
    let on = t("common.select.on").into_owned();
    assert!(h.text().contains(&winnable_button(&off)));
    assert!(h.text().contains(t("menu.winnable.off_hint").as_ref()));
    h.type_char('w');
    assert!(h.app.settings().winnable_only);
    assert!(h.text().contains(&winnable_button(&on)));
    assert!(h.text().contains(t("menu.winnable.on_hint").as_ref()));
    assert!(
        h.storage
            .borrow()
            .settings
            .as_ref()
            .is_some_and(|settings| settings.winnable_only)
    );
    h.type_char('w');
    assert!(!h.app.settings().winnable_only);
    assert!(h.text().contains(&winnable_button(&off)));
}

#[test]
fn the_footer_names_every_key_of_the_screen() {
    let mut h = select(Locale::EnUs);
    let text = h.text();
    for entry in footer_entries() {
        assert!(text.contains(&entry), "{entry:?} is not drawn in {text}");
    }
}

#[test]
fn the_w_key_is_typed_into_an_open_search() {
    let mut h = select(Locale::EnUs);
    h.type_char('/');
    h.type_char('w');
    assert!(!h.app.settings().winnable_only);
    assert!(h.text().contains("[ / Search: w_ ]"));
}

#[test]
fn the_screen_fits_at_100_by_30_in_every_locale() {
    for locale in Locale::offered() {
        let mut h = select(locale);
        let text = h.text();
        let mut needles = footer_entries();
        needles.push(winnable_button(&t("common.select.off")));
        needles.push(t("menu.winnable.off_hint").into_owned());
        needles.push(description_of_first_variant());
        for needle in needles {
            assert!(
                text.contains(&needle),
                "{locale:?}: {needle:?} is not drawn"
            );
        }
    }
    set_locale(Locale::EnUs);
}
