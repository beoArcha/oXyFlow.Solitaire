//! Localization (decision T-06): TOML area files embedded at build time, flat string keys,
//! fallback to `en-US` and then to the key itself.
//!
//! A key is `<area>.<path>`: the area is the file name (`common.toml`), the path follows the TOML
//! tables, so `[menu] new_game = "…"` in `common.toml` is `common.menu.new_game`.

use std::borrow::Cow;
use std::cell::Cell;
use std::collections::HashMap;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A supported interface language. `en-US` is the source locale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Locale {
    #[serde(rename = "en-US")]
    EnUs,
    #[serde(rename = "pl-PL")]
    PlPl,
}

impl Locale {
    /// Every locale, source locale first.
    pub const ALL: [Locale; 2] = [Locale::EnUs, Locale::PlPl];

    /// The folder name under `locales/`.
    pub const fn id(self) -> &'static str {
        match self {
            Locale::EnUs => "en-US",
            Locale::PlPl => "pl-PL",
        }
    }

    /// The locale with the given folder name.
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|locale| locale.id() == id)
    }
}

/// A locale file that cannot be turned into a catalog. Embedded files are checked by tests, so
/// this only occurs through a broken build.
#[derive(Debug, Error)]
pub enum I18nError {
    #[error("locale file {area}.toml is not valid TOML")]
    Parse {
        area: &'static str,
        #[source]
        source: toml::de::Error,
    },
    #[error("locale key {key} is not a string")]
    NotAString { key: String },
}

type Catalog = HashMap<String, String>;

macro_rules! area {
    ($locale:literal, $area:literal) => {
        (
            $area,
            include_str!(concat!("../../../locales/", $locale, "/", $area, ".toml")),
        )
    };
}

macro_rules! areas {
    ($locale:literal) => {
        [
            area!($locale, "common"),
            area!($locale, "menu"),
            area!($locale, "board"),
            area!($locale, "settings"),
            area!($locale, "help"),
            area!($locale, "cards"),
            area!($locale, "cli"),
            area!($locale, "audio"),
            area!($locale, "rules"),
            area!($locale, "stats"),
        ]
    };
}

const EN_US: [(&str, &str); 10] = areas!("en-US");
const PL_PL: [(&str, &str); 10] = areas!("pl-PL");

fn sources(locale: Locale) -> &'static [(&'static str, &'static str); 10] {
    match locale {
        Locale::EnUs => &EN_US,
        Locale::PlPl => &PL_PL,
    }
}

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

pub(crate) fn load_catalog(locale: Locale) -> Result<Catalog, I18nError> {
    let mut catalog = Catalog::new();
    for (area, text) in sources(locale) {
        let table = toml::from_str::<toml::Table>(text)
            .map_err(|source| I18nError::Parse { area, source })?;
        flatten(area, &table, &mut catalog)?;
    }
    Ok(catalog)
}

fn catalog(locale: Locale) -> &'static Catalog {
    static EN: OnceLock<Catalog> = OnceLock::new();
    static PL: OnceLock<Catalog> = OnceLock::new();
    let cell = match locale {
        Locale::EnUs => &EN,
        Locale::PlPl => &PL,
    };
    cell.get_or_init(|| match load_catalog(locale) {
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

/// The text for `key` in the current locale, else in `en-US`, else the key itself.
pub fn t(key: &str) -> Cow<'_, str> {
    let found = catalog(locale())
        .get(key)
        .or_else(|| catalog(Locale::EnUs).get(key));
    match found {
        Some(text) => Cow::Borrowed(text.as_str()),
        None => Cow::Borrowed(key),
    }
}

/// Like [`t`], replacing every `{name}` with the matching value of `params`.
pub fn tf(key: &str, params: &[(&str, &str)]) -> String {
    let mut text = t(key).into_owned();
    for (name, value) in params {
        text = text.replace(&format!("{{{name}}}"), value);
    }
    text
}

#[cfg(test)]
mod tests;
