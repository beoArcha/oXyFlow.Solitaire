//! Plural forms of counted words, after the CLDR plural rules for whole numbers. A counted text is
//! a group of keys `<key>_one`, `<key>_few`, `<key>_many`, `<key>_other`; each locale has exactly
//! the forms its rule needs (see [`PluralRule::forms`]).

/// The CLDR plural category of a number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluralCategory {
    One,
    Few,
    Many,
    Other,
}

impl PluralCategory {
    /// Every category, in CLDR order.
    pub const ALL: [PluralCategory; 4] = [Self::One, Self::Few, Self::Many, Self::Other];

    /// The key suffix of the form: `one`, `few`, `many`, `other`.
    pub const fn suffix(self) -> &'static str {
        match self {
            Self::One => "one",
            Self::Few => "few",
            Self::Many => "many",
            Self::Other => "other",
        }
    }
}

/// A family of languages that pick plural forms the same way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluralRule {
    /// English, German, Dutch, Italian, Spanish: `one` for 1, else `other`.
    OneOther,
    /// French and Brazilian Portuguese: `one` for 0 and 1, else `other`.
    ZeroOneOther,
    /// Polish: `one` for 1; `few` for 2–4, 22–24, … but not 12–14; else `many`.
    Polish,
    /// Ukrainian: `one` for 1, 21, 31, … but not 11; `few` as Polish; else `many`.
    EastSlavic,
}

fn ends_in_two_to_four(count: usize) -> bool {
    (2..=4).contains(&(count % 10)) && !(12..=14).contains(&(count % 100))
}

impl PluralRule {
    /// The form used for a whole `count`. CLDR also calls whole millions `many` in French,
    /// Italian, Spanish and Portuguese, only for compact numbers; a game never counts that far, so
    /// those locales use `other` for them.
    pub fn category(self, count: usize) -> PluralCategory {
        match self {
            Self::OneOther if count == 1 => PluralCategory::One,
            Self::ZeroOneOther if count <= 1 => PluralCategory::One,
            Self::OneOther | Self::ZeroOneOther => PluralCategory::Other,
            Self::Polish if count == 1 => PluralCategory::One,
            Self::EastSlavic if count % 10 == 1 && count % 100 != 11 => PluralCategory::One,
            Self::Polish | Self::EastSlavic if ends_in_two_to_four(count) => PluralCategory::Few,
            Self::Polish | Self::EastSlavic => PluralCategory::Many,
        }
    }

    /// The forms a locale with this rule must have for every counted text.
    pub const fn forms(self) -> &'static [PluralCategory] {
        match self {
            Self::OneOther | Self::ZeroOneOther => &[PluralCategory::One, PluralCategory::Other],
            Self::Polish | Self::EastSlavic => &[
                PluralCategory::One,
                PluralCategory::Few,
                PluralCategory::Many,
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PluralCategory::{Few, Many, One, Other};
    use super::*;
    use crate::i18n::Locale;

    fn forms_of(locale: Locale, counts: &[usize]) -> Vec<PluralCategory> {
        counts
            .iter()
            .map(|count| locale.plural_rule().category(*count))
            .collect()
    }

    const COUNTS: [usize; 12] = [0, 1, 2, 4, 5, 11, 12, 21, 22, 25, 104, 1_000_000];

    #[test]
    fn english_german_dutch_italian_and_spanish_say_one_only_for_one() {
        let expected = [
            Other, One, Other, Other, Other, Other, Other, Other, Other, Other, Other, Other,
        ];
        for locale in [
            Locale::EnUs,
            Locale::DeDe,
            Locale::NlNl,
            Locale::ItIt,
            Locale::EsEs,
        ] {
            assert_eq!(forms_of(locale, &COUNTS), expected, "{}", locale.id());
        }
    }

    #[test]
    fn french_and_brazilian_portuguese_say_one_for_zero_and_one() {
        let expected = [
            One, One, Other, Other, Other, Other, Other, Other, Other, Other, Other, Other,
        ];
        for locale in [Locale::FrFr, Locale::PtBr] {
            assert_eq!(forms_of(locale, &COUNTS), expected, "{}", locale.id());
        }
    }

    #[test]
    fn polish_says_one_only_for_one_and_few_for_two_to_four_but_not_teens() {
        let expected = [
            Many, One, Few, Few, Many, Many, Many, Many, Few, Many, Few, Many,
        ];
        assert_eq!(forms_of(Locale::PlPl, &COUNTS), expected);
    }

    #[test]
    fn ukrainian_says_one_for_twenty_one_but_not_eleven() {
        let expected = [
            Many, One, Few, Few, Many, Many, Many, One, Few, Many, Few, Many,
        ];
        assert_eq!(forms_of(Locale::UkUa, &COUNTS), expected);
    }

    #[test]
    fn every_form_a_rule_picks_is_one_of_its_forms() {
        for locale in Locale::ALL {
            let rule = locale.plural_rule();
            for count in 0..2_000 {
                assert!(
                    rule.forms().contains(&rule.category(count)),
                    "{}",
                    locale.id()
                );
            }
        }
    }

    #[test]
    fn every_category_has_a_distinct_suffix() {
        let mut suffixes: Vec<&str> = PluralCategory::ALL.map(PluralCategory::suffix).to_vec();
        suffixes.dedup();
        assert_eq!(suffixes.len(), 4);
    }
}
