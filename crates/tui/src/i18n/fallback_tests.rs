use super::plural::PluralCategory::{Few, Many, One, Other};
use super::*;

fn catalog(pairs: &[(&str, &str)]) -> Catalog {
    pairs
        .iter()
        .map(|(key, text)| ((*key).to_owned(), (*text).to_owned()))
        .collect()
}

fn english() -> Catalog {
    catalog(&[
        ("a.hello", "Hello"),
        ("a.bye", "Bye"),
        ("a.items_one", "{count} item"),
        ("a.items_other", "{count} items"),
    ])
}

#[test]
fn a_key_the_locale_has_wins_over_english() {
    let own = catalog(&[("a.hello", "Cześć")]);
    assert_eq!(lookup(&own, &english(), "a.hello"), Some("Cześć"));
}

#[test]
fn a_missing_key_falls_back_to_english_and_then_to_nothing() {
    let own = catalog(&[("a.hello", "Cześć")]);
    assert_eq!(lookup(&own, &english(), "a.bye"), Some("Bye"));
    assert_eq!(lookup(&own, &english(), "a.nope"), None);
}

#[test]
fn a_missing_plural_form_falls_back_to_the_english_form_of_the_same_category() {
    let own = catalog(&[("a.items_few", "{count} rzeczy")]);
    let source = english();
    assert_eq!(
        plural_lookup(&own, &source, "a.items", One),
        Some("{count} item")
    );
    assert_eq!(
        plural_lookup(&own, &source, "a.items", Few),
        Some("{count} rzeczy")
    );
}

#[test]
fn a_plural_form_english_does_not_have_falls_back_to_other() {
    let own = Catalog::new();
    let source = english();
    assert_eq!(
        plural_lookup(&own, &source, "a.items", Many),
        Some("{count} items")
    );
    assert_eq!(
        plural_lookup(&own, &source, "a.items", Other),
        Some("{count} items")
    );
}

#[test]
fn a_missing_plural_group_gives_the_key_with_the_form() {
    let text = counted(&Catalog::new(), &english(), "a.nope", One, &[]);
    assert_eq!(text, "a.nope_one");
}

#[test]
fn a_counted_text_from_the_fallback_still_gets_its_placeholders() {
    let own = Catalog::new();
    let text = counted(&own, &english(), "a.items", Many, &[("count", "5")]);
    assert_eq!(text, "5 items");
}
