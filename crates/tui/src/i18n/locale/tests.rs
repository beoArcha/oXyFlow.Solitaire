use std::collections::BTreeSet;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Saved {
    locale: Locale,
}

fn locales_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../locales")
}

fn names_in(dir: &Path) -> BTreeSet<String> {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn the_list_holds_the_nine_locales_with_the_source_first() {
    let ids: Vec<&str> = Locale::ALL.iter().map(|locale| locale.id()).collect();
    assert_eq!(
        ids,
        [
            "en-US", "pl-PL", "de-DE", "fr-FR", "it-IT", "pt-BR", "uk-UA", "nl-NL", "es-ES"
        ]
    );
    for (index, locale) in Locale::ALL.iter().enumerate() {
        assert_eq!(locale.index(), index);
    }
}

#[test]
fn every_locale_is_saved_under_its_folder_name_and_read_back() {
    for locale in Locale::ALL {
        let text = toml::to_string(&Saved { locale }).unwrap();
        assert_eq!(text.trim(), format!("locale = \"{}\"", locale.id()));
        assert_eq!(toml::from_str::<Saved>(&text).unwrap(), Saved { locale });
    }
}

#[test]
fn every_folder_under_locales_is_a_listed_locale_with_meta_and_the_same_files() {
    let folders = names_in(&locales_dir());
    let listed: BTreeSet<String> = Locale::ALL.iter().map(|l| l.id().to_owned()).collect();
    assert_eq!(folders, listed);
    let files = names_in(&locales_dir().join("en-US"));
    assert_eq!(files.len(), AREA_COUNT + 1);
    assert!(files.contains("meta.toml"));
    for locale in Locale::ALL {
        assert_eq!(
            names_in(&locales_dir().join(locale.id())),
            files,
            "{}",
            locale.id()
        );
    }
}

#[test]
fn every_meta_file_parses_and_names_the_language_in_itself() {
    for locale in Locale::ALL {
        let meta = parse_meta(locale.sources().meta).unwrap();
        assert!(!meta.own_name.is_empty(), "{}", locale.id());
        assert_eq!(locale.own_name(), meta.own_name);
    }
    assert_eq!(Locale::DeDe.own_name(), "Deutsch");
    assert_eq!(Locale::UkUa.own_name(), "Українська");
    assert_eq!(Locale::PtBr.own_name(), "Português (Brasil)");
}

#[test]
fn a_meta_file_needs_both_fields_and_nothing_else() {
    let good = parse_meta("own_name = \"Deutsch\"\ncomplete = true\n").unwrap();
    assert_eq!(
        good,
        Meta {
            own_name: "Deutsch".to_owned(),
            complete: true
        }
    );
    assert!(parse_meta("own_name = \"Deutsch\"\n").is_err());
    assert!(parse_meta("own_name = \"Deutsch\"\ncomplete = true\nflag = \"DE\"\n").is_err());
}

#[test]
fn the_source_and_polish_are_complete() {
    assert!(Locale::EnUs.is_complete());
    assert!(Locale::PlPl.is_complete());
    let offered = Locale::offered();
    assert_eq!(&offered[..2], &[Locale::EnUs, Locale::PlPl]);
}

#[test]
fn only_complete_locales_are_offered_and_the_rest_fall_back_to_the_source() {
    for locale in Locale::ALL {
        assert_eq!(Locale::offered().contains(&locale), locale.is_complete());
        let expected = if locale.is_complete() {
            locale
        } else {
            Locale::EnUs
        };
        assert_eq!(locale.or_source(), expected, "{}", locale.id());
    }
}

#[test]
fn a_language_tag_is_matched_on_its_language_part() {
    let cases = [
        ("pl_PL.UTF-8", Some(Locale::PlPl)),
        ("en_GB", Some(Locale::EnUs)),
        ("de_AT.UTF-8", Some(Locale::DeDe)),
        ("de-CH", Some(Locale::DeDe)),
        ("fr_CA", Some(Locale::FrFr)),
        ("it", Some(Locale::ItIt)),
        ("pt_PT.UTF-8", Some(Locale::PtBr)),
        ("pt-BR", Some(Locale::PtBr)),
        ("uk_UA.UTF-8", Some(Locale::UkUa)),
        ("nl_BE@euro", Some(Locale::NlNl)),
        ("ES_mx", Some(Locale::EsEs)),
        ("C", None),
        ("cs_CZ", None),
        ("", None),
    ];
    for (tag, expected) in cases {
        assert_eq!(Locale::from_language_tag(tag), expected, "{tag}");
    }
}
