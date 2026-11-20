use super::*;

fn catalog(pairs: &[(&str, &str)]) -> Catalog {
    pairs
        .iter()
        .map(|(key, text)| ((*key).to_owned(), (*text).to_owned()))
        .collect()
}

fn english() -> Catalog {
    catalog(&[
        ("a.hello", "Hello {name}"),
        ("a.bye", "Bye"),
        ("a.items_one", "{count} item"),
        ("a.items_other", "{count} items in {pile}"),
    ])
}

fn polish_full() -> Catalog {
    catalog(&[
        ("a.hello", "Cześć {name}"),
        ("a.bye", "Pa"),
        ("a.items_one", "{count} rzecz"),
        ("a.items_few", "{count} rzeczy"),
        ("a.items_many", "{count} rzeczy w {pile}"),
    ])
}

#[test]
fn a_full_translation_is_clean() {
    let result = diff(&english(), &polish_full(), PluralRule::Polish);
    assert!(result.is_clean(), "{result:?}");
}

#[test]
fn a_missing_plain_key_is_reported() {
    let mut polish = polish_full();
    polish.remove("a.bye");
    let result = diff(&english(), &polish, PluralRule::Polish);
    assert_eq!(result.missing, vec!["a.bye".to_owned()]);
    assert!(result.extra.is_empty() && result.mismatched.is_empty());
}

#[test]
fn an_extra_key_is_reported() {
    let mut polish = polish_full();
    polish.insert("a.surplus".to_owned(), "x".to_owned());
    let result = diff(&english(), &polish, PluralRule::Polish);
    assert_eq!(result.extra, vec!["a.surplus".to_owned()]);
}

#[test]
fn a_placeholder_mismatch_is_reported() {
    let mut polish = polish_full();
    polish.insert("a.hello".to_owned(), "Cześć {nazwa}".to_owned());
    let result = diff(&english(), &polish, PluralRule::Polish);
    assert_eq!(result.mismatched.len(), 1);
    assert_eq!(result.mismatched[0].key, "a.hello");
    assert!(result.mismatched[0].expected.contains("name"));
    assert!(result.mismatched[0].found.contains("nazwa"));
}

#[test]
fn a_missing_plural_form_of_the_rule_is_reported() {
    let mut polish = polish_full();
    polish.remove("a.items_few");
    let result = diff(&english(), &polish, PluralRule::Polish);
    assert_eq!(result.missing, vec!["a.items_few".to_owned()]);
}

#[test]
fn a_plural_form_the_rule_does_not_use_is_extra() {
    let mut german = catalog(&[
        ("a.hello", "Hallo {name}"),
        ("a.bye", "Tschüss"),
        ("a.items_one", "{count} Ding"),
        ("a.items_other", "{count} Dinge"),
    ]);
    assert!(diff(&english(), &german, PluralRule::OneOther).is_clean());
    german.insert("a.items_few".to_owned(), "{count}".to_owned());
    let result = diff(&english(), &german, PluralRule::OneOther);
    assert_eq!(result.extra, vec!["a.items_few".to_owned()]);
}

#[test]
fn a_plural_form_may_use_fewer_placeholders_than_other_but_not_new_ones() {
    let mut polish = polish_full();
    polish.insert("a.items_few".to_owned(), "{count} i {nowy}".to_owned());
    let result = diff(&english(), &polish, PluralRule::Polish);
    assert_eq!(result.mismatched.len(), 1);
    assert_eq!(result.mismatched[0].key, "a.items_few");
}

#[test]
fn a_complete_locale_must_be_clean_and_an_incomplete_one_may_miss_keys() {
    let mut polish = polish_full();
    polish.remove("a.bye");
    polish.remove("a.items_many");
    let result = diff(&english(), &polish, PluralRule::Polish);
    assert!(!result.passes(true));
    assert!(result.passes(false));

    polish.insert("a.surplus".to_owned(), "x".to_owned());
    let result = diff(&english(), &polish, PluralRule::Polish);
    assert!(!result.passes(false));

    let mut wrong = polish_full();
    wrong.insert("a.hello".to_owned(), "Cześć".to_owned());
    assert!(!diff(&english(), &wrong, PluralRule::Polish).passes(false));
}

#[test]
fn placeholders_ignore_text_in_braces_that_is_not_a_name() {
    let found = placeholders("{a} {B} {} {c_d} {open");
    let expected: BTreeSet<String> = ["a", "c_d"].map(str::to_owned).into();
    assert_eq!(found, expected);
}
