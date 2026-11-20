//! The `style.toml` of a front and of a back as the files spell them, and the checks that turn them
//! into [`FrontStyle`] and [`BackStyle`]: a font key must exist in the registry and lie in the
//! range of the oXySol font, symbols must be ones the glyph set may draw, colors must be tokens.

use std::collections::HashMap;

use serde::Deserialize;

use super::{
    BackStyle, CardStyleError, FONT_FIRST, FONT_LAST, FrontStyle, Mosaic, Ranks, SuitStyle, Tile,
};
use crate::glyphs::is_wgl4;
use crate::theme::Token;

/// The glyph registry `assets/cards/codepoints.toml`: a key and the character of its code point.
#[derive(Debug, Default)]
pub struct Registry(HashMap<String, char>);

#[derive(Deserialize)]
struct RawRegistry {
    glyphs: HashMap<String, String>,
}

impl Registry {
    /// Reads the `[glyphs]` table of the registry.
    pub fn parse(text: &str) -> Result<Self, CardStyleError> {
        let raw: RawRegistry = toml::from_str(text).map_err(CardStyleError::Registry)?;
        let mut glyphs = HashMap::new();
        for (key, code) in raw.glyphs {
            let character = u32::from_str_radix(&code, 16)
                .ok()
                .filter(|code| (FONT_FIRST..=FONT_LAST).contains(code))
                .and_then(char::from_u32)
                .ok_or_else(|| CardStyleError::RegistryCode {
                    key: key.clone(),
                    code,
                })?;
            glyphs.insert(key, character);
        }
        Ok(Self(glyphs))
    }

    fn glyph(&self, id: &str, key: &str) -> Result<String, CardStyleError> {
        self.0
            .get(key)
            .map(char::to_string)
            .ok_or_else(|| CardStyleError::UnknownGlyph {
                id: id.to_owned(),
                key: key.to_owned(),
            })
    }

    fn optional(&self, id: &str, key: Option<&String>) -> Result<Option<String>, CardStyleError> {
        key.map(|key| self.glyph(id, key)).transpose()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFront {
    name: String,
    #[serde(default)]
    ranks: RawRanks,
    suits: RawSuits,
    courts: Option<RawCourts>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct RawRanks {
    ace: Option<String>,
    jack: Option<String>,
    queen: Option<String>,
    king: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSuits {
    clubs: RawSuit,
    diamonds: RawSuit,
    hearts: RawSuit,
    spades: RawSuit,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSuit {
    name: String,
    font: Option<String>,
    unicode: String,
    ascii: String,
    color: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCourts {
    jack: String,
    queen: String,
    king: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBack {
    name: String,
    fg: String,
    bg: String,
    tile: RawTile,
    mosaic: RawMosaic,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTile {
    font: Option<String>,
    unicode: String,
    ascii: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMosaic {
    font: Option<Vec<String>>,
    unicode: Vec<String>,
    ascii: Vec<String>,
}

fn invalid(id: &str, field: &str, value: &str) -> CardStyleError {
    CardStyleError::Invalid {
        id: id.to_owned(),
        field: field.to_owned(),
        value: value.to_owned(),
    }
}

fn token(id: &str, field: &str, name: &str, allowed: &[Token]) -> Result<Token, CardStyleError> {
    Token::from_name(name)
        .filter(|token| allowed.is_empty() || allowed.contains(token))
        .ok_or_else(|| invalid(id, field, name))
}

fn unicode_symbols(id: &str, field: &str, text: &str, count: usize) -> Result<(), CardStyleError> {
    if text.chars().count() == count && text.chars().all(is_wgl4) {
        Ok(())
    } else {
        Err(invalid(id, field, text))
    }
}

fn ascii_symbols(id: &str, field: &str, text: &str, count: usize) -> Result<(), CardStyleError> {
    if text.chars().count() == count && text.chars().all(|c| c.is_ascii_graphic()) {
        Ok(())
    } else {
        Err(invalid(id, field, text))
    }
}

fn suit_style(id: &str, raw: &RawSuit, registry: &Registry) -> Result<SuitStyle, CardStyleError> {
    unicode_symbols(id, "suits.unicode", &raw.unicode, 1)?;
    if raw.ascii.len() != 1 || !raw.ascii.chars().all(|c| c.is_ascii_lowercase()) {
        return Err(invalid(id, "suits.ascii", &raw.ascii));
    }
    Ok(SuitStyle {
        name_key: raw.name.clone(),
        font: registry.optional(id, raw.font.as_ref())?,
        unicode: raw.unicode.clone(),
        ascii: raw.ascii.clone(),
        token: token(
            id,
            "suits.color",
            &raw.color,
            &[Token::CardRed, Token::CardBlack],
        )?,
    })
}

fn rank_label(
    id: &str,
    name: &str,
    label: Option<String>,
) -> Result<Option<String>, CardStyleError> {
    match label {
        Some(text)
            if text.is_empty()
                || text.len() > 2
                || !text.chars().all(|c| c.is_ascii_uppercase()) =>
        {
            Err(invalid(id, name, &text))
        }
        other => Ok(other),
    }
}

fn distinct(id: &str, field: &str, values: [&str; 4]) -> Result<(), CardStyleError> {
    for (index, value) in values.iter().enumerate() {
        if values[index + 1..].contains(value) {
            return Err(invalid(id, field, value));
        }
    }
    Ok(())
}

fn courts(
    id: &str,
    raw: Option<&RawCourts>,
    registry: &Registry,
) -> Result<Option<[String; 3]>, CardStyleError> {
    raw.map(|courts| {
        Ok([
            registry.glyph(id, &courts.jack)?,
            registry.glyph(id, &courts.queen)?,
            registry.glyph(id, &courts.king)?,
        ])
    })
    .transpose()
}

/// Reads and checks the manifest of the front style `id`.
pub fn front(id: &str, text: &str, registry: &Registry) -> Result<FrontStyle, CardStyleError> {
    let raw: RawFront = toml::from_str(text).map_err(|source| CardStyleError::Toml {
        id: id.to_owned(),
        source,
    })?;
    let suits = [
        suit_style(id, &raw.suits.clubs, registry)?,
        suit_style(id, &raw.suits.diamonds, registry)?,
        suit_style(id, &raw.suits.hearts, registry)?,
        suit_style(id, &raw.suits.spades, registry)?,
    ];
    let with_font = suits.iter().filter(|suit| suit.font.is_some()).count();
    if with_font != 0 && with_font != suits.len() {
        return Err(invalid(id, "suits.font", "some suits only"));
    }
    distinct(
        id,
        "suits.unicode",
        suits.each_ref().map(|suit| suit.unicode.as_str()),
    )?;
    distinct(
        id,
        "suits.ascii",
        suits.each_ref().map(|suit| suit.ascii.as_str()),
    )?;
    Ok(FrontStyle {
        id: id.to_owned(),
        name_key: raw.name,
        suits,
        ranks: Ranks {
            ace: rank_label(id, "ranks.ace", raw.ranks.ace)?,
            jack: rank_label(id, "ranks.jack", raw.ranks.jack)?,
            queen: rank_label(id, "ranks.queen", raw.ranks.queen)?,
            king: rank_label(id, "ranks.king", raw.ranks.king)?,
        },
        courts: courts(id, raw.courts.as_ref(), registry)?,
    })
}

fn mosaic_rows(
    id: &str,
    field: &str,
    rows: &[String],
    ascii: bool,
) -> Result<[String; 2], CardStyleError> {
    let [top, bottom] = rows else {
        return Err(invalid(id, field, "two rows are needed"));
    };
    for row in [top, bottom] {
        if ascii {
            ascii_symbols(id, field, row, 2)?;
        } else {
            unicode_symbols(id, field, row, 2)?;
        }
    }
    Ok([top.clone(), bottom.clone()])
}

fn mosaic_font(
    id: &str,
    keys: Option<&Vec<String>>,
    registry: &Registry,
) -> Result<Option<[String; 4]>, CardStyleError> {
    let Some(keys) = keys else { return Ok(None) };
    let [tl, tr, bl, br] = keys.as_slice() else {
        return Err(invalid(id, "mosaic.font", "four keys are needed"));
    };
    Ok(Some([
        registry.glyph(id, tl)?,
        registry.glyph(id, tr)?,
        registry.glyph(id, bl)?,
        registry.glyph(id, br)?,
    ]))
}

/// Reads and checks the manifest of the back style `id`.
pub fn back(id: &str, text: &str, registry: &Registry) -> Result<BackStyle, CardStyleError> {
    let raw: RawBack = toml::from_str(text).map_err(|source| CardStyleError::Toml {
        id: id.to_owned(),
        source,
    })?;
    unicode_symbols(id, "tile.unicode", &raw.tile.unicode, 1)?;
    ascii_symbols(id, "tile.ascii", &raw.tile.ascii, 1)?;
    Ok(BackStyle {
        id: id.to_owned(),
        name_key: raw.name,
        tile: Tile {
            font: registry.optional(id, raw.tile.font.as_ref())?,
            unicode: raw.tile.unicode,
            ascii: raw.tile.ascii,
        },
        mosaic: Mosaic {
            font: mosaic_font(id, raw.mosaic.font.as_ref(), registry)?,
            unicode: mosaic_rows(id, "mosaic.unicode", &raw.mosaic.unicode, false)?,
            ascii: mosaic_rows(id, "mosaic.ascii", &raw.mosaic.ascii, true)?,
        },
        fg: token(id, "fg", &raw.fg, &[])?,
        bg: token(id, "bg", &raw.bg, &[])?,
    })
}
