//! The list of interface languages. Each line of [`locales!`] names a locale, its folder under
//! `locales/`, its plural rule and its flag; everything else about a language is data in that
//! folder, including `meta.toml` with its own name and whether its translation is complete. Only
//! complete locales are offered to the player (decision T-36).

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use super::flags::{self, Flag};
use super::plural::PluralRule;

/// The embedded text of one locale: its `meta.toml` and its area files.
pub(crate) struct Sources {
    pub(crate) meta: &'static str,
    pub(crate) areas: [(&'static str, &'static str); AREA_COUNT],
}

/// How many area files every locale has.
pub(crate) const AREA_COUNT: usize = 10;

macro_rules! area {
    ($locale:literal, $area:literal) => {
        (
            $area,
            include_str!(concat!(
                "../../../../locales/",
                $locale,
                "/",
                $area,
                ".toml"
            )),
        )
    };
}

macro_rules! sources {
    ($locale:literal) => {
        Sources {
            meta: include_str!(concat!("../../../../locales/", $locale, "/meta.toml")),
            areas: [
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
            ],
        }
    };
}

macro_rules! locales {
    ($($variant:ident => $id:literal, $plural:ident, $flag:ident;)+) => {
        /// A supported interface language. `en-US` is the source locale.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum Locale {
            $(#[serde(rename = $id)] $variant,)+
        }

        impl Locale {
            /// Every locale, complete or not, source locale first.
            pub const ALL: [Locale; [$($id),+].len()] = [$(Locale::$variant),+];

            /// The folder name under `locales/`.
            pub const fn id(self) -> &'static str {
                match self {
                    $(Locale::$variant => $id,)+
                }
            }

            /// How this language picks the form of a counted word.
            pub const fn plural_rule(self) -> PluralRule {
                match self {
                    $(Locale::$variant => PluralRule::$plural,)+
                }
            }

            /// The flag drawn for this language.
            pub const fn flag(self) -> &'static Flag {
                match self {
                    $(Locale::$variant => &flags::$flag,)+
                }
            }
        }

        pub(crate) const SOURCES: [Sources; Locale::ALL.len()] = [$(sources!($id)),+];
    };
}

locales! {
    EnUs => "en-US", OneOther, EN_US;
    PlPl => "pl-PL", Polish, PL_PL;
    DeDe => "de-DE", OneOther, DE_DE;
    FrFr => "fr-FR", ZeroOneOther, FR_FR;
    ItIt => "it-IT", OneOther, IT_IT;
    PtBr => "pt-BR", ZeroOneOther, PT_BR;
    UkUa => "uk-UA", EastSlavic, UK_UA;
    NlNl => "nl-NL", OneOther, NL_NL;
    EsEs => "es-ES", OneOther, ES_ES;
}

/// What `meta.toml` says about a locale.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Meta {
    /// The name of the language in itself, shown whatever language is on.
    pub own_name: String,
    /// Whether the translation is finished. Only complete locales are offered.
    pub complete: bool,
}

/// Parses the text of a `meta.toml`.
pub(crate) fn parse_meta(text: &str) -> Result<Meta, toml::de::Error> {
    toml::from_str(text)
}

impl Locale {
    pub(crate) const fn index(self) -> usize {
        self as usize
    }

    pub(crate) fn sources(self) -> &'static Sources {
        &SOURCES[self.index()]
    }

    fn meta(self) -> &'static Meta {
        static METAS: [OnceLock<Meta>; Locale::ALL.len()] =
            [const { OnceLock::new() }; Locale::ALL.len()];
        METAS[self.index()].get_or_init(|| match parse_meta(self.sources().meta) {
            Ok(meta) => meta,
            Err(error) => {
                tracing::error!(locale = self.id(), %error, "embedded meta.toml is broken");
                Meta {
                    own_name: self.id().to_owned(),
                    complete: false,
                }
            }
        })
    }

    /// The names of the area files every locale folder has, without `.toml`.
    pub fn area_names() -> [&'static str; AREA_COUNT] {
        Locale::EnUs.sources().areas.map(|(name, _)| name)
    }

    /// The name of the language in itself: `Deutsch`, `Polski`.
    pub fn own_name(self) -> &'static str {
        &self.meta().own_name
    }

    /// Whether the translation is finished. The source locale always is.
    pub fn is_complete(self) -> bool {
        self == Locale::EnUs || self.meta().complete
    }

    /// The locales the player can choose, in the order of [`Locale::ALL`].
    pub fn offered() -> Vec<Locale> {
        Self::ALL
            .into_iter()
            .filter(|locale| locale.is_complete())
            .collect()
    }

    /// This locale if it is offered, else the source locale: what a saved setting or the
    /// environment naming an unfinished translation turns into.
    pub fn or_source(self) -> Locale {
        if self.is_complete() {
            self
        } else {
            Locale::EnUs
        }
    }

    /// The locale with the given folder name.
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|locale| locale.id() == id)
    }

    /// The locale for a POSIX or BCP 47 language tag such as `de_AT.UTF-8`, `pt-PT` or `uk`,
    /// matched on the language part. `None` for a language without a locale.
    pub fn from_language_tag(tag: &str) -> Option<Self> {
        let language = tag
            .split(['_', '-', '.', '@'])
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();
        Self::ALL
            .into_iter()
            .find(|locale| locale.id().split('-').next() == Some(language.as_str()))
    }
}

#[cfg(test)]
mod tests;
