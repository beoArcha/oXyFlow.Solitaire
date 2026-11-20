use std::collections::BTreeSet;

use oxysol_tui::i18n::diff::Mismatch;

use super::*;

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

fn words(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

fn options(lang: Option<&str>, git_ref: Option<&str>) -> Options {
    Options {
        lang: lang.map(str::to_owned),
        git_ref: git_ref.map(str::to_owned),
    }
}

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

#[test]
fn flags_name_an_optional_language_and_an_optional_ref() {
    assert_eq!(parse_options(&[]), Some(options(None, None)));
    assert_eq!(
        parse_options(&strings(&["de-DE"])),
        Some(options(Some("de-DE"), None))
    );
    assert_eq!(
        parse_options(&strings(&["--ref", "main", "de-DE"])),
        Some(options(Some("de-DE"), Some("main")))
    );
    assert_eq!(parse_options(&strings(&["--ref"])), None);
    assert_eq!(parse_options(&strings(&["a", "b"])), None);
    assert_eq!(parse_options(&strings(&["--quick"])), None);
}

#[test]
fn the_report_lists_missing_extra_and_mismatched_keys() {
    let result = Diff {
        missing: strings(&["a.one_few"]),
        extra: strings(&["a.surplus"]),
        mismatched: vec![Mismatch {
            key: "a.hello".to_owned(),
            expected: words(&["name"]),
            found: words(&["nazwa"]),
        }],
    };
    assert_eq!(
        render(Locale::PlPl, &result),
        strings(&[
            "pl-PL: 1 missing, 1 extra, 1 placeholder mismatch(es)",
            "  missing  a.one_few",
            "  extra    a.surplus",
            "  placeholders  a.hello: expected [{name}], found [{nazwa}]",
        ])
    );
}

#[test]
fn a_clean_diff_is_one_line() {
    assert_eq!(
        render(Locale::DeDe, &Diff::default()),
        strings(&["de-DE: clean"])
    );
}

#[test]
fn the_nine_locales_in_the_working_tree_are_clean() {
    assert!(run(&workspace(), &options(None, None)).is_ok());
    assert!(run(&workspace(), &options(Some("pl-PL"), None)).is_ok());
}

#[test]
fn an_unknown_language_is_an_error() {
    let result = run(&workspace(), &options(Some("xx-XX"), None));
    assert!(matches!(result, Err(LocaleDiffError::UnknownLocale(..))));
}

#[test]
fn an_unknown_ref_is_an_error() {
    let result = run(
        &workspace(),
        &options(Some("pl-PL"), Some("no-such-ref-for-locale-diff")),
    );
    assert!(matches!(result, Err(LocaleDiffError::GitShow { .. })));
}

fn write_locale(root: &Path, locale: &str, extra: &str) {
    let dir = root.join("locales").join(locale);
    assert!(fs::create_dir_all(&dir).is_ok());
    for area in Locale::area_names() {
        let text = if area == "common" {
            format!("[x]\nhello = \"Hello {{name}}\"\n{extra}")
        } else {
            String::new()
        };
        assert!(fs::write(dir.join(format!("{area}.toml")), text).is_ok());
    }
}

#[test]
fn a_locale_that_differs_makes_the_run_fail() {
    let root = std::env::temp_dir().join(format!("oxysol-locale-diff-{}", std::process::id()));
    write_locale(&root, "en-US", "bye = \"Bye\"\n");
    write_locale(&root, "de-DE", "");
    let result = run(&root, &options(Some("de-DE"), None));
    assert!(fs::remove_dir_all(&root).is_ok());
    assert!(matches!(result, Err(LocaleDiffError::Differs { count: 1 })));
}
