use super::*;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use oxysol_core::{Card, CardId, PileRef, Rank, Suit};

use crate::announce::{card_name, pile_name};

const AREAS: [&str; 10] = [
    "common", "menu", "board", "settings", "help", "cards", "cli", "audio", "rules", "stats",
];

fn key_set(locale: Locale) -> BTreeSet<String> {
    load_catalog(locale)
        .map(|catalog| catalog.into_keys().collect())
        .unwrap_or_default()
}

fn rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for path in entries.flatten().map(|entry| entry.path()) {
        if path.is_dir() {
            rust_sources(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

fn is_key_text(key: &str) -> bool {
    let allowed = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.';
    key.chars().all(allowed) && !key.ends_with('.')
}

fn keys_in(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    for area in AREAS {
        let opener = format!("\"{area}.");
        for (start, _) in text.match_indices(&opener) {
            let key: String = text[start + 1..]
                .chars()
                .take_while(|c| *c != '"')
                .collect();
            if is_key_text(&key) {
                found.push(key);
            }
        }
    }
    found
}

#[test]
fn embedded_locales_parse() {
    for locale in Locale::ALL {
        assert!(load_catalog(locale).is_ok(), "{}", locale.id());
    }
}

fn diff_of(locale: Locale) -> diff::Diff {
    diff::diff(catalog(Locale::EnUs), catalog(locale), locale.plural_rule())
}

#[test]
fn every_locale_obeys_its_parity_rule_against_en_us() {
    assert!(key_set(Locale::EnUs).contains("board.announce.targets_one"));
    for locale in Locale::ALL {
        let result = diff_of(locale);
        assert!(
            result.passes(locale.is_complete()),
            "{} (complete: {}): {result:?}",
            locale.id(),
            locale.is_complete()
        );
    }
}

#[test]
fn every_offered_locale_mirrors_every_en_us_key() {
    for locale in Locale::offered() {
        assert!(diff_of(locale).is_clean(), "{}", locale.id());
    }
}

#[test]
fn a_counted_text_takes_the_form_of_the_locale() {
    assert_eq!(tn("board.announce.targets", 1, &[]), "1 target");
    assert_eq!(tn("board.announce.targets", 3, &[]), "3 targets");
    set_locale(Locale::PlPl);
    let polish = [1, 3, 5, 22].map(|count| tn("board.announce.targets", count, &[]));
    set_locale(Locale::EnUs);
    assert_eq!(polish, ["1 cel", "3 cele", "5 celów", "22 cele"]);
}

#[test]
fn every_key_used_in_the_sources_exists() {
    let mut files = Vec::new();
    rust_sources(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut files,
    );
    assert!(!files.is_empty());
    let known = key_set(Locale::EnUs);
    for file in files {
        let text = std::fs::read_to_string(&file).unwrap_or_default();
        for key in keys_in(&text) {
            assert!(
                known.contains(&key) || known.contains(&format!("{key}_other")),
                "{} uses unknown key {key}",
                file.display()
            );
        }
    }
}

#[test]
fn every_area_file_exists_in_every_locale() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../locales");
    for locale in Locale::ALL {
        for area in AREAS {
            let file = root.join(locale.id()).join(format!("{area}.toml"));
            assert!(file.is_file(), "{}", file.display());
        }
    }
}

#[test]
fn lookup_falls_back_to_the_key() {
    let missing = format!("{}.no.such.key", "common");
    assert_eq!(t(&missing), missing);
}

#[test]
fn lookup_uses_the_selected_locale() {
    set_locale(Locale::PlPl);
    assert_eq!(locale(), Locale::PlPl);
    assert_eq!(t("common.menu.quit"), "Zakończ");
    set_locale(Locale::EnUs);
}

#[test]
fn placeholders_are_replaced() {
    let text = tf("common.size.current", &[("width", "80"), ("height", "24")]);
    assert_eq!(text, "Current size: 80 x 24");
}

#[test]
fn locale_ids_round_trip() {
    for locale in Locale::ALL {
        assert_eq!(Locale::from_id(locale.id()), Some(locale));
    }
    assert_eq!(Locale::from_id("cs-CZ"), None);
}

#[test]
fn scanner_finds_keys_but_not_file_names() {
    let found = keys_in(r#"t("common.menu.quit") include_str!("x/common.toml")"#);
    assert_eq!(found, vec!["common.menu.quit".to_owned()]);
}

#[test]
fn a_pile_name_stands_at_the_start_after_a_colon_or_in_parentheses() {
    for locale in Locale::ALL {
        for (key, text) in catalog(locale) {
            for (at, _) in text.match_indices("{pile}") {
                let before = text[..at].trim_end();
                assert!(
                    before.is_empty() || before.ends_with(':') || before.ends_with('('),
                    "{} {key}: {text}",
                    locale.id()
                );
            }
        }
    }
}

fn all_cards() -> Vec<Card> {
    let mut cards = Vec::new();
    for suit in Suit::ALL {
        for rank in Rank::ALL {
            let id = CardId(u16::try_from(cards.len()).unwrap_or_default());
            cards.push(Card { id, suit, rank });
        }
    }
    cards
}

fn longest<I: IntoIterator<Item = String>>(names: I) -> String {
    names
        .into_iter()
        .max_by_key(|name| name.chars().count())
        .unwrap_or_default()
}

#[test]
fn now_row_names_fit_forty_columns_for_a_tableau_pile() {
    for locale in Locale::ALL {
        set_locale(locale);
        let pile = pile_name(PileRef::Tableau(9));
        let card = longest(all_cards().into_iter().map(card_name));
        let texts = [
            tf("board.action.browsing", &[("pile", &pile), ("what", &card)]),
            tf("board.action.hint_single", &[("what", &card)]),
        ];
        for text in texts {
            assert!(
                text.chars().count() <= 40,
                "{} is {} columns: {text}",
                locale.id(),
                text.chars().count()
            );
        }
    }
    set_locale(Locale::EnUs);
}
