//! The text of `--version`: the version, then the copyright and licence from the package metadata.

use oxysol_tui::i18n;

/// Year of the copyright line.
pub const COPYRIGHT_YEAR: &str = "2026";

/// The two lines printed by `--version`, in the current locale.
pub fn version_text() -> String {
    version_text_for(
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS"),
        env!("CARGO_PKG_LICENSE"),
    )
}

/// Like [`version_text`] for explicit values. `authors` is the Cargo form, names joined with `:`.
pub fn version_text_for(version: &str, authors: &str, license: &str) -> String {
    let names = authors.replace(':', ", ");
    let first = i18n::tf("cli.version", &[("version", version)]);
    let second = i18n::tf(
        "cli.version_legal",
        &[
            ("year", COPYRIGHT_YEAR),
            ("authors", names.as_str()),
            ("license", license),
        ],
    );
    format!("{first}\n{second}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxysol_tui::i18n::Locale;

    const AUTHORS: &str = "Marcin \"beoArcha\" Stefan";

    #[test]
    fn the_second_line_has_the_copyright_and_the_licence() {
        i18n::set_locale(Locale::EnUs);
        let text = version_text_for("1.2.3", AUTHORS, "MIT");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(
            lines,
            [
                "oXySol 1.2.3",
                "Copyright (c) 2026 Marcin \"beoArcha\" Stefan. MIT License."
            ]
        );
    }

    #[test]
    fn several_authors_are_listed_with_commas() {
        i18n::set_locale(Locale::EnUs);
        let text = version_text_for("1.0.0", "A:B", "MIT");
        assert!(text.contains("2026 A, B."), "{text}");
    }

    #[test]
    fn the_polish_text_names_the_licence_too() {
        i18n::set_locale(Locale::PlPl);
        let text = version_text_for("1.2.3", AUTHORS, "MIT");
        i18n::set_locale(Locale::EnUs);
        assert!(text.contains(AUTHORS), "{text}");
        assert!(text.contains("MIT"), "{text}");
        assert!(text.contains("Licencja"), "{text}");
    }

    #[test]
    fn the_package_metadata_gives_the_owner_and_the_licence() {
        i18n::set_locale(Locale::EnUs);
        let text = version_text();
        assert!(text.contains(AUTHORS), "{text}");
        assert!(text.contains("MIT"), "{text}");
        assert!(!text.contains('@'), "{text}");
    }
}
