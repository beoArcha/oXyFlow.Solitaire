//! Localization (decision T-06): TOML area files embedded at build time, flat string keys,
//! fallback per key to `en-US` and then to the key itself (decision T-38).
//!
//! A key is `<area>.<path>`: the area is the file name (`common.toml`), the path follows the TOML
//! tables, so `[menu] new_game = "…"` in `common.toml` is `common.menu.new_game`.

use std::borrow::Cow;
use std::cell::Cell;
use std::collections::HashMap;
use std::sync::OnceLock;

use thiserror::Error;

pub mod diff;
pub mod flags;
mod locale;
pub mod plural;

pub use locale::{Locale, Meta};

use plural::PluralCategory;

/// A locale file that cannot be turned into a catalog. Embedded files are checked by tests, so
/// this only occurs through a broken build.
#[derive(Debug, Error)]
pub enum I18nError {
    #[error("locale file {area}.toml is not valid TOML")]
    Parse {
        area: String,
        #[source]
        source: toml::de::Error,
    },
    #[error("locale key {key} is not a string")]
    NotAString { key: String },
}

/// The flat key to text map of one locale.
pub type Catalog = HashMap<String, String>;

fn flatten(prefix: &str, table: &toml::Table, out: &mut Catalog) -> Result<(), I18nError> {
    for (name, value) in table {
        let key = format!("{prefix}.{name}");
        match value {
            toml::Value::String(text) => {
                out.insert(key, text.clone());
            }
            toml::Value::Table(inner) => flatten(&key, inner, out)?,
            _ => return Err(I18nError::NotAString { key }),
        }
    }
    Ok(())
}

/// Builds a catalog from `(area, TOML text)` pairs.
pub fn parse_catalog<'a>(
    areas: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Result<Catalog, I18nError> {
    let mut catalog = Catalog::new();
    for (area, text) in areas {
        let table = toml::from_str::<toml::Table>(text).map_err(|source| I18nError::Parse {
            area: area.to_owned(),
            source,
        })?;
        flatten(area, &table, &mut catalog)?;
    }
    Ok(catalog)
}

pub(crate) fn load_catalog(locale: Locale) -> Result<Catalog, I18nError> {
    parse_catalog(locale.sources().areas.iter().copied())
}

fn catalog(locale: Locale) -> &'static Catalog {
    static CATALOGS: [OnceLock<Catalog>; Locale::ALL.len()] =
        [const { OnceLock::new() }; Locale::ALL.len()];
    CATALOGS[locale.index()].get_or_init(|| match load_catalog(locale) {
        Ok(catalog) => catalog,
        Err(error) => {
            tracing::error!(locale = locale.id(), %error, "embedded locale is broken");
            Catalog::new()
        }
    })
}

thread_local! {
    static CURRENT: Cell<Locale> = const { Cell::new(Locale::EnUs) };
}

/// Selects the language used by [`t`] and [`tf`] on this thread. The TUI is single-threaded.
pub fn set_locale(locale: Locale) {
    CURRENT.with(|current| current.set(locale));
}

/// The language selected on this thread.
pub fn locale() -> Locale {
    CURRENT.with(Cell::get)
}

fn lookup<'a>(own: &'a Catalog, source: &'a Catalog, key: &str) -> Option<&'a str> {
    own.get(key).or_else(|| source.get(key)).map(String::as_str)
}

fn plural_lookup<'a>(
    own: &'a Catalog,
    source: &'a Catalog,
    key: &str,
    form: PluralCategory,
) -> Option<&'a str> {
    lookup(own, source, &format!("{key}_{}", form.suffix()))
        .or_else(|| lookup(own, source, &format!("{key}_other")))
}

fn substitute(mut text: String, params: &[(&str, &str)]) -> String {
    for (name, value) in params {
        text = text.replace(&format!("{{{name}}}"), value);
    }
    text
}

fn counted(
    own: &Catalog,
    source: &Catalog,
    key: &str,
    form: PluralCategory,
    params: &[(&str, &str)],
) -> String {
    let text = plural_lookup(own, source, key, form)
        .map_or_else(|| format!("{key}_{}", form.suffix()), str::to_owned);
    substitute(text, params)
}

/// The text for `key` in the current locale, else in `en-US`, else the key itself.
pub fn t(key: &str) -> Cow<'_, str> {
    match lookup(catalog(locale()), catalog(Locale::EnUs), key) {
        Some(text) => Cow::Borrowed(text),
        None => Cow::Borrowed(key),
    }
}

/// Like [`t`], replacing every `{name}` with the matching value of `params`.
pub fn tf(key: &str, params: &[(&str, &str)]) -> String {
    substitute(t(key).into_owned(), params)
}

/// The text counted by `count`: the plural form `<key>_<form>` the current locale's rule picks
/// (`board.announce.targets` gives `targets_one` or `targets_other` in English), with `{count}` and
/// every other `{name}` of `params` replaced. A form the locale lacks falls back to the `en-US`
/// text of the same form, then to the `_other` text of the locale, then of `en-US`.
pub fn tn(key: &str, count: usize, params: &[(&str, &str)]) -> String {
    let form = locale().plural_rule().category(count);
    let number = count.to_string();
    let mut all = vec![("count", number.as_str())];
    all.extend_from_slice(params);
    counted(catalog(locale()), catalog(Locale::EnUs), key, form, &all)
}

#[cfg(test)]
mod fallback_tests;
#[cfg(test)]
mod tests;
