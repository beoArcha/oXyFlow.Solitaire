//! Comparison of a locale with `en-US` (decision T-38): which keys it lacks, which it adds and where
//! its `{placeholders}` differ. Plural groups are compared by the forms of the locale's rule. A
//! complete locale must have an empty [`Diff`]; an incomplete one may only miss keys.

use std::collections::BTreeSet;

use super::Catalog;
use super::plural::{PluralCategory, PluralRule};

/// A key whose placeholders differ from the English text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mismatch {
    pub key: String,
    pub expected: BTreeSet<String>,
    pub found: BTreeSet<String>,
}

/// What a locale lacks, adds and writes differently, all sorted by key.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Diff {
    pub missing: Vec<String>,
    pub extra: Vec<String>,
    pub mismatched: Vec<Mismatch>,
}

impl Diff {
    /// Nothing missing, extra or mismatched.
    pub fn is_clean(&self) -> bool {
        self.missing.is_empty() && self.extra.is_empty() && self.mismatched.is_empty()
    }

    /// Whether the locale obeys its parity rule: a complete locale must be clean, an incomplete
    /// one may miss keys but has no extra key and no placeholder mismatch.
    pub fn passes(&self, complete: bool) -> bool {
        if complete {
            self.is_clean()
        } else {
            self.extra.is_empty() && self.mismatched.is_empty()
        }
    }
}

/// The `{name}` placeholders of a text.
pub fn placeholders(text: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        let inner = &rest[open + 1..];
        let Some(close) = inner.find('}') else {
            break;
        };
        let name = &inner[..close];
        if !name.is_empty() && name.bytes().all(|b| b.is_ascii_lowercase() || b == b'_') {
            found.insert(name.to_owned());
        }
        rest = &inner[close + 1..];
    }
    found
}

fn plural_bases(source: &Catalog) -> BTreeSet<&str> {
    source
        .keys()
        .filter_map(|key| key.strip_suffix("_other"))
        .filter(|base| source.contains_key(&format!("{base}_one")))
        .collect()
}

fn plural_form_of<'a>(key: &'a str, bases: &BTreeSet<&str>) -> Option<(&'a str, &'a str)> {
    let (base, suffix) = key.rsplit_once('_')?;
    let known = PluralCategory::ALL
        .iter()
        .any(|form| form.suffix() == suffix);
    (known && bases.contains(base)).then_some((base, suffix))
}

fn mismatch(key: &str, expected: &str, found: &str) -> Option<Mismatch> {
    let (expected, found) = (placeholders(expected), placeholders(found));
    (expected != found).then(|| Mismatch {
        key: key.to_owned(),
        expected,
        found,
    })
}

fn plain_keys(source: &Catalog, translated: &Catalog, bases: &BTreeSet<&str>, diff: &mut Diff) {
    for (key, text) in source {
        if plural_form_of(key, bases).is_some() {
            continue;
        }
        match translated.get(key) {
            None => diff.missing.push(key.clone()),
            Some(found) => diff.mismatched.extend(mismatch(key, text, found)),
        }
    }
    for key in translated.keys() {
        if plural_form_of(key, bases).is_none() && !source.contains_key(key) {
            diff.extra.push(key.clone());
        }
    }
}

fn plural_keys(
    source: &Catalog,
    translated: &Catalog,
    rule: PluralRule,
    bases: &BTreeSet<&str>,
    diff: &mut Diff,
) {
    for base in bases {
        let reference = source
            .get(&format!("{base}_other"))
            .map_or("", String::as_str);
        for form in PluralCategory::ALL {
            let key = format!("{base}_{}", form.suffix());
            match translated.get(&key) {
                None if rule.forms().contains(&form) => diff.missing.push(key),
                None => {}
                Some(_) if !rule.forms().contains(&form) => diff.extra.push(key),
                Some(found) => {
                    let allowed = placeholders(reference);
                    let used = placeholders(found);
                    if !used.is_subset(&allowed) {
                        diff.mismatched.push(Mismatch {
                            key,
                            expected: allowed,
                            found: used,
                        });
                    }
                }
            }
        }
    }
}

/// Compares `translated`, a locale with plural `rule`, with the source locale `source`.
pub fn diff(source: &Catalog, translated: &Catalog, rule: PluralRule) -> Diff {
    let bases = plural_bases(source);
    let mut result = Diff::default();
    plain_keys(source, translated, &bases, &mut result);
    plural_keys(source, translated, rule, &bases, &mut result);
    result.missing.sort();
    result.extra.sort();
    result.mismatched.sort_by(|a, b| a.key.cmp(&b.key));
    result
}

#[cfg(test)]
mod tests;
