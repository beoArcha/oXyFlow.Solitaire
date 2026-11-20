//! Tests of the style manifests, the registry checks and the embedded styles.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use oxysol_core::{Rank, Suit};

use super::*;
use crate::glyphs::is_wgl4;

const REGISTRY: &str = r#"
[glyphs]
"fronts/test/a" = "E6BC"
"fronts/test/b" = "E6BD"
"fronts/test/c" = "E6BE"
"fronts/test/d" = "E6BF"
"backs/test/tile" = "E6D8"
"#;

const PLAIN_FRONT: &str = r#"
name = "settings.style.international"

[suits.clubs]
name = "board.word.clubs"
unicode = "♣"
ascii = "c"
color = "card_black"

[suits.diamonds]
name = "board.word.diamonds"
unicode = "♦"
ascii = "d"
color = "card_red"

[suits.hearts]
name = "board.word.hearts"
unicode = "♥"
ascii = "h"
color = "card_red"

[suits.spades]
name = "board.word.spades"
unicode = "♠"
ascii = "s"
color = "card_black"
"#;

const BACK: &str = r##"
name = "settings.style.lattice"
fg = "card_back_mark"
bg = "card_back"

[tile]
font = "backs/test/tile"
unicode = "▒"
ascii = "#"

[mosaic]
unicode = ["▓▒", "▒▓"]
ascii = ["@#", "#@"]
"##;

fn embedded() -> Loaded {
    CardStyles::load(
        embedded::EMBEDDED_FRONTS,
        embedded::EMBEDDED_BACKS,
        embedded::CODEPOINTS,
    )
}

fn registry() -> manifest::Registry {
    manifest::Registry::parse(REGISTRY).unwrap_or_default()
}

fn folders(kind: &str) -> Vec<String> {
    let cards = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/cards");
    let mut ids: Vec<String> = fs::read_dir(cards.join(kind))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.path().join("style.toml").is_file())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    ids.sort();
    ids
}

fn sorted(mut ids: Vec<&str>) -> Vec<&str> {
    ids.sort_unstable();
    ids
}

#[test]
fn every_embedded_manifest_passes_its_checks() {
    let loaded = embedded();
    assert!(loaded.problems.is_empty(), "{:?}", loaded.problems);
}

#[test]
fn the_fronts_and_backs_of_the_card_are_found_the_default_first() {
    let loaded = embedded();
    let fronts: Vec<&str> = loaded
        .styles
        .fronts()
        .iter()
        .map(|s| s.id.as_str())
        .collect();
    let backs: Vec<&str> = loaded
        .styles
        .backs()
        .iter()
        .map(|s| s.id.as_str())
        .collect();
    assert_eq!(fronts, ["international", "french", "german", "italian"]);
    assert_eq!(backs, ["lattice", "ornament"]);
}

#[test]
fn the_embedded_styles_follow_the_folders_so_a_new_folder_needs_no_code_change() {
    let loaded = embedded();
    let fronts: Vec<&str> = loaded
        .styles
        .fronts()
        .iter()
        .map(|s| s.id.as_str())
        .collect();
    let backs: Vec<&str> = loaded
        .styles
        .backs()
        .iter()
        .map(|s| s.id.as_str())
        .collect();
    assert_eq!(sorted(fronts), folders("fronts"));
    assert_eq!(sorted(backs), folders("backs"));
}

#[test]
fn a_style_given_as_a_manifest_is_listed_and_found_by_its_id() {
    let loaded = CardStyles::load(
        &[("zebra", PLAIN_FRONT), (DEFAULT_FRONT, PLAIN_FRONT)],
        &[("stripes", BACK)],
        REGISTRY,
    );
    assert!(loaded.problems.is_empty(), "{:?}", loaded.problems);
    let ids: Vec<&str> = loaded
        .styles
        .fronts()
        .iter()
        .map(|s| s.id.as_str())
        .collect();
    assert_eq!(ids, [DEFAULT_FRONT, "zebra"]);
    assert!(loaded.styles.has_front("zebra"));
    assert!(loaded.styles.has_back("stripes"));
}

#[test]
fn an_unknown_id_falls_back_to_the_default_style() {
    let styles = embedded().styles;
    let front = styles
        .front_or_default("removed")
        .map(|style| style.id.as_str());
    let back = styles
        .back_or_default("removed")
        .map(|style| style.id.as_str());
    assert_eq!(front, Some(DEFAULT_FRONT));
    assert_eq!(back, Some(DEFAULT_BACK));
}

#[test]
fn a_front_without_font_keys_is_accepted_and_draws_unicode_in_font_mode() {
    let front = manifest::front("swiss", PLAIN_FRONT, &registry());
    let Ok(front) = front else {
        panic!("a front without font keys must be accepted: {front:?}");
    };
    assert!(!front.has_font());
    assert_eq!(
        front.suit_symbols(GlyphMode::Font),
        front.suit_symbols(GlyphMode::Unicode)
    );
    assert_eq!(front.court_emblem(Rank::King), None);
    assert_eq!(front.suit_glyph(Suit::Hearts), None);
}

#[test]
fn a_font_key_missing_from_the_registry_is_rejected() {
    let text = PLAIN_FRONT.replacen(
        "unicode = \"♣\"",
        "font = \"fronts/test/missing\"\nunicode = \"♣\"",
        1,
    );
    let result = manifest::front("test", &text, &registry());
    assert!(
        matches!(result, Err(CardStyleError::UnknownGlyph { .. })),
        "{result:?}"
    );
}

#[test]
fn some_suits_with_a_font_key_and_some_without_is_rejected() {
    let text = PLAIN_FRONT.replacen(
        "unicode = \"♣\"",
        "font = \"fronts/test/a\"\nunicode = \"♣\"",
        1,
    );
    let result = manifest::front("test", &text, &registry());
    assert!(
        matches!(result, Err(CardStyleError::Invalid { .. })),
        "{result:?}"
    );
}

#[test]
fn a_registry_code_point_outside_the_font_range_is_rejected() {
    for code in ["E6BB", "E700", "0041", "zzzz"] {
        let text = format!("[glyphs]\n\"fronts/test/a\" = \"{code}\"\n");
        let result = manifest::Registry::parse(&text);
        assert!(
            matches!(result, Err(CardStyleError::RegistryCode { .. })),
            "{code}"
        );
    }
}

#[test]
fn a_color_that_is_no_card_token_is_rejected() {
    for color in ["red", "text", "card_face", "#ff0000"] {
        let text = PLAIN_FRONT.replacen("card_black", color, 1);
        let result = manifest::front("test", &text, &registry());
        assert!(
            matches!(result, Err(CardStyleError::Invalid { .. })),
            "{color}"
        );
    }
    let back = BACK.replacen("card_back_mark", "pink", 1);
    assert!(manifest::back("test", &back, &registry()).is_err());
}

#[test]
fn symbols_outside_the_glyph_sets_are_rejected() {
    let not_wgl4 = PLAIN_FRONT.replacen("♣", "🂡", 1);
    assert!(manifest::front("test", &not_wgl4, &registry()).is_err());
    let upper = PLAIN_FRONT.replacen("ascii = \"c\"", "ascii = \"C\"", 1);
    assert!(manifest::front("test", &upper, &registry()).is_err());
    let twin = PLAIN_FRONT.replacen("ascii = \"d\"", "ascii = \"c\"", 1);
    assert!(manifest::front("test", &twin, &registry()).is_err());
    let wide_back = BACK.replacen("unicode = \"▒\"", "unicode = \"▒▒\"", 1);
    assert!(manifest::back("test", &wide_back, &registry()).is_err());
}

#[test]
fn an_unknown_field_is_rejected() {
    let text = format!("{PLAIN_FRONT}\n[extra]\nvalue = 1\n");
    assert!(matches!(
        manifest::front("test", &text, &registry()),
        Err(CardStyleError::Toml { .. })
    ));
}

#[test]
fn a_failing_style_is_left_out_and_reported() {
    let loaded = CardStyles::load(&[("good", PLAIN_FRONT), ("bad", "name = 1")], &[], REGISTRY);
    assert_eq!(loaded.styles.fronts().len(), 1);
    assert_eq!(loaded.problems.len(), 1);
}

#[test]
fn every_font_key_of_an_embedded_manifest_is_in_the_registry_inside_the_font_range() {
    let registry = manifest::Registry::parse(embedded::CODEPOINTS);
    assert!(registry.is_ok(), "{registry:?}");
    let mut keys = HashSet::new();
    let sources = embedded::EMBEDDED_FRONTS
        .iter()
        .chain(embedded::EMBEDDED_BACKS);
    for (id, text) in sources {
        let mut found = 0;
        for line in text.lines() {
            for quoted in line.split('"').skip(1).step_by(2) {
                if quoted.starts_with("fronts/") || quoted.starts_with("backs/") {
                    keys.insert(quoted.to_owned());
                    found += 1;
                }
            }
        }
        assert!(found > 0, "{id} names no font glyph");
    }
    let glyphs = embedded::CODEPOINTS;
    for key in keys {
        let line = glyphs
            .lines()
            .find(|line| line.starts_with(&format!("\"{key}\" = ")));
        let code = line
            .and_then(|line| line.split('"').nth(3))
            .and_then(|code| u32::from_str_radix(code, 16).ok());
        let Some(code) = code else {
            panic!("`{key}` is not in codepoints.toml");
        };
        assert!((FONT_FIRST..=FONT_LAST).contains(&code), "{key}");
    }
}

#[test]
fn the_suits_of_every_front_differ_in_every_glyph_mode() {
    let styles = embedded().styles;
    for front in styles.fronts() {
        for mode in [GlyphMode::Font, GlyphMode::Unicode, GlyphMode::Ascii] {
            let symbols: HashSet<&str> = front.suit_symbols(mode).into_iter().collect();
            assert_eq!(symbols.len(), 4, "{} {mode:?}", front.id);
        }
    }
}

#[test]
fn every_embedded_unicode_symbol_is_wgl4_and_every_ascii_symbol_is_ascii() {
    let styles = embedded().styles;
    for front in styles.fronts() {
        let unicode = front.suit_symbols(GlyphMode::Unicode);
        assert!(
            unicode.iter().all(|s| s.chars().all(is_wgl4)),
            "{}",
            front.id
        );
        let ascii = front.suit_symbols(GlyphMode::Ascii);
        assert!(ascii.iter().all(|s| s.is_ascii()), "{}", front.id);
    }
    for back in styles.backs() {
        assert!(
            back.tile(GlyphMode::Unicode).chars().all(is_wgl4),
            "{}",
            back.id
        );
        assert!(back.tile(GlyphMode::Ascii).is_ascii(), "{}", back.id);
        for row in back.mosaic_rows(GlyphMode::Unicode) {
            assert!(row.chars().all(is_wgl4), "{}", back.id);
        }
        for row in back.mosaic_rows(GlyphMode::Ascii) {
            assert!(row.is_ascii(), "{}", back.id);
        }
    }
}

#[test]
fn the_font_glyphs_of_the_embedded_styles_lie_in_the_private_use_area_range() {
    let styles = embedded().styles;
    let in_range = |glyph: &str| {
        glyph
            .chars()
            .all(|c| (FONT_FIRST..=FONT_LAST).contains(&u32::from(c)))
    };
    for front in styles.fronts() {
        assert!(front.has_font(), "{}", front.id);
        assert!(
            front
                .suit_symbols(GlyphMode::Font)
                .iter()
                .all(|s| in_range(s))
        );
        assert!(front.court_emblem(Rank::Queen).is_some_and(in_range));
    }
    for back in styles.backs() {
        assert!(in_range(back.tile(GlyphMode::Font)), "{}", back.id);
        for row in back.mosaic_rows(GlyphMode::Font) {
            assert_eq!(row.chars().count(), 2);
            assert!(in_range(&row), "{}", back.id);
        }
    }
}

#[test]
fn the_court_labels_of_the_skins_and_the_localized_labels_of_international() {
    let styles = embedded().styles;
    let label = |id: &str, rank| {
        styles
            .front_or_default(id)
            .and_then(|front| front.rank_label(rank))
            .map(str::to_owned)
    };
    assert_eq!(label("international", Rank::Queen), None);
    assert_eq!(label("french", Rank::Jack).as_deref(), Some("V"));
    assert_eq!(label("german", Rank::King).as_deref(), Some("K"));
    assert_eq!(label("italian", Rank::Jack).as_deref(), Some("F"));
    assert_eq!(label("french", Rank::Seven), None);
}

#[test]
fn german_and_italian_use_the_international_court_emblems_and_french_its_suits() {
    let styles = embedded().styles;
    let emblem = |id: &str, rank| {
        styles
            .front_or_default(id)
            .and_then(|front| front.court_emblem(rank))
            .map(str::to_owned)
    };
    for rank in [Rank::Jack, Rank::Queen, Rank::King] {
        assert!(emblem("international", rank).is_some());
        assert_eq!(emblem("german", rank), emblem("international", rank));
        assert_eq!(emblem("italian", rank), emblem("international", rank));
    }
    let suits = |id: &str| {
        styles
            .front_or_default(id)
            .map(|front| front.suit_symbols(GlyphMode::Font).map(str::to_owned))
    };
    assert_eq!(suits("french"), suits("international"));
    assert_ne!(suits("german"), suits("international"));
}

#[test]
fn every_style_name_and_suit_name_is_a_localized_key_in_every_locale() {
    use crate::i18n::{Locale, set_locale, t};
    let styles = embedded().styles;
    let mut keys = Vec::new();
    for front in styles.fronts() {
        keys.push(front.name_key.clone());
        keys.extend(Suit::ALL.map(|suit| front.suit(suit).name_key.clone()));
    }
    keys.extend(styles.backs().iter().map(|back| back.name_key.clone()));
    for locale in Locale::offered() {
        set_locale(locale);
        for key in &keys {
            assert_ne!(t(key), key.as_str(), "{key} in {}", locale.id());
        }
    }
    set_locale(Locale::EnUs);
}
