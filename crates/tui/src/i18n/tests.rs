use super::*;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const AREAS: [&str; 9] = [
    "common", "menu", "board", "settings", "help", "cards", "cli", "audio", "rules",
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

#[test]
fn every_locale_has_the_same_keys_as_en_us() {
    let source = key_set(Locale::EnUs);
    assert!(!source.is_empty());
    for locale in Locale::ALL {
        assert_eq!(key_set(locale), source, "{}", locale.id());
    }
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
                known.contains(&key),
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
    assert_eq!(t("common.menu.quit"), "Quit");
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
    assert_eq!(Locale::from_id("de-DE"), None);
}

#[test]
fn scanner_finds_keys_but_not_file_names() {
    let found = keys_in(r#"t("common.menu.quit") include_str!("x/common.toml")"#);
    assert_eq!(found, vec!["common.menu.quit".to_owned()]);
}
