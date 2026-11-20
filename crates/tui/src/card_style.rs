//! Card styles (D-06, P5-02): the fronts and backs the card widget draws, read from the `style.toml`
//! of every folder under `assets/cards/fronts/` and `assets/cards/backs/`. The build script embeds
//! the folders, so a new style needs no code change. A front gives the suit symbols, court labels and
//! emblems of the three glyph modes; a back gives the tile that fills it and a 2 × 2 mosaic. A
//! style that fails its checks is left out and logged; an unknown id in the settings falls back to
//! the default style.

use std::sync::OnceLock;

use oxysol_core::{Rank, Suit};

use crate::settings::GlyphMode;
use crate::theme::Token;

mod manifest;
#[cfg(test)]
mod tests;

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/card_styles.rs"));
}

/// The front style used when none is saved or the saved one no longer exists.
pub const DEFAULT_FRONT: &str = "international";
/// The back style used when none is saved or the saved one no longer exists.
pub const DEFAULT_BACK: &str = "lattice";
/// First code point of the oXySol card font.
pub const FONT_FIRST: u32 = 0xE6BC;
/// Last code point of the oXySol card font.
pub const FONT_LAST: u32 = 0xE6FF;

/// Why a style or the glyph registry was rejected.
#[derive(Debug, thiserror::Error)]
pub enum CardStyleError {
    #[error("the glyph registry cannot be read")]
    Registry(#[source] toml::de::Error),
    #[error("registry entry `{key}` has the code point `{code}`, which is not in U+E6BC to U+E6FF")]
    RegistryCode { key: String, code: String },
    #[error("style `{id}` cannot be read")]
    Toml {
        id: String,
        #[source]
        source: toml::de::Error,
    },
    #[error("style `{id}` names the font glyph `{key}`, which the registry does not have")]
    UnknownGlyph { id: String, key: String },
    #[error("style `{id}`: `{value}` is not allowed in `{field}`")]
    Invalid {
        id: String,
        field: String,
        value: String,
    },
}

/// How one suit of a front looks in the three glyph modes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuitStyle {
    pub name_key: String,
    font: Option<String>,
    unicode: String,
    ascii: String,
    pub token: Token,
}

/// The labels a front gives to ace and court ranks; `None` keeps the localized label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ranks {
    ace: Option<String>,
    jack: Option<String>,
    queen: Option<String>,
    king: Option<String>,
}

/// A card front style.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontStyle {
    pub id: String,
    pub name_key: String,
    suits: [SuitStyle; 4],
    ranks: Ranks,
    courts: Option<[String; 3]>,
}

impl FrontStyle {
    /// The style of a suit; the order is clubs, diamonds, hearts, spades.
    pub fn suit(&self, suit: Suit) -> &SuitStyle {
        &self.suits[suit_index(suit)]
    }

    /// Whether the style has Private Use Area glyphs for its suits.
    pub fn has_font(&self) -> bool {
        self.suits.iter().all(|suit| suit.font.is_some())
    }

    /// The four suit symbols of a glyph mode; Font mode falls back to Unicode for a style without
    /// font glyphs.
    pub fn suit_symbols(&self, mode: GlyphMode) -> [&str; 4] {
        self.suits.each_ref().map(|suit| match mode {
            GlyphMode::Font => suit.font.as_deref().unwrap_or(&suit.unicode),
            GlyphMode::Unicode => &suit.unicode,
            GlyphMode::Ascii => &suit.ascii,
        })
    }

    /// The label this style gives to a rank, or `None` to keep the localized one.
    pub fn rank_label(&self, rank: Rank) -> Option<&str> {
        match rank {
            Rank::Ace => self.ranks.ace.as_deref(),
            Rank::Jack => self.ranks.jack.as_deref(),
            Rank::Queen => self.ranks.queen.as_deref(),
            Rank::King => self.ranks.king.as_deref(),
            _ => None,
        }
    }

    /// The Private Use Area glyph that stands for the court rank.
    pub fn court_emblem(&self, rank: Rank) -> Option<&str> {
        let [jack, queen, king] = self.courts.as_ref()?;
        match rank {
            Rank::Jack => Some(jack),
            Rank::Queen => Some(queen),
            Rank::King => Some(king),
            _ => None,
        }
    }

    /// The Private Use Area glyph of a suit.
    pub fn suit_glyph(&self, suit: Suit) -> Option<&str> {
        self.suit(suit).font.as_deref()
    }
}

const fn suit_index(suit: Suit) -> usize {
    match suit {
        Suit::Clubs => 0,
        Suit::Diamonds => 1,
        Suit::Hearts => 2,
        Suit::Spades => 3,
    }
}

/// The tile that fills a back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tile {
    font: Option<String>,
    unicode: String,
    ascii: String,
}

/// The 2 × 2 decoration in the middle of a large back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mosaic {
    font: Option<[String; 4]>,
    unicode: [String; 2],
    ascii: [String; 2],
}

/// A card back style.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackStyle {
    pub id: String,
    pub name_key: String,
    tile: Tile,
    mosaic: Mosaic,
    /// Token of the pattern.
    pub fg: Token,
    /// Token of the background.
    pub bg: Token,
}

impl BackStyle {
    /// The tile of a glyph mode; Font mode falls back to Unicode for a style without a font glyph.
    pub fn tile(&self, mode: GlyphMode) -> &str {
        match mode {
            GlyphMode::Font => self.tile.font.as_deref().unwrap_or(&self.tile.unicode),
            GlyphMode::Unicode => &self.tile.unicode,
            GlyphMode::Ascii => &self.tile.ascii,
        }
    }

    /// The two rows of the mosaic in a glyph mode.
    pub fn mosaic_rows(&self, mode: GlyphMode) -> [String; 2] {
        match (mode, &self.mosaic.font) {
            (GlyphMode::Font, Some([tl, tr, bl, br])) => [format!("{tl}{tr}"), format!("{bl}{br}")],
            (GlyphMode::Ascii, _) => self.mosaic.ascii.clone(),
            _ => self.mosaic.unicode.clone(),
        }
    }
}

/// Every style that passed its checks, the default first and the rest in the order of their ids.
#[derive(Debug, Default)]
pub struct CardStyles {
    fronts: Vec<FrontStyle>,
    backs: Vec<BackStyle>,
}

/// The result of reading a set of manifests: the styles that passed and why the others did not.
#[derive(Debug)]
pub struct Loaded {
    pub styles: CardStyles,
    pub problems: Vec<CardStyleError>,
}

fn keep<T>(
    result: Result<T, CardStyleError>,
    kept: &mut Vec<T>,
    problems: &mut Vec<CardStyleError>,
) {
    match result {
        Ok(style) => kept.push(style),
        Err(error) => problems.push(error),
    }
}

fn default_first<T>(styles: &mut [T], is_default: impl Fn(&T) -> bool) {
    if let Some(position) = styles.iter().position(is_default) {
        styles[..=position].rotate_right(1);
    }
}

impl CardStyles {
    /// Reads manifests given as `(folder id, text)` against the registry text. A style that fails is
    /// reported in `problems` and left out.
    pub fn load(fronts: &[(&str, &str)], backs: &[(&str, &str)], codepoints: &str) -> Loaded {
        let mut problems = Vec::new();
        let registry = manifest::Registry::parse(codepoints).unwrap_or_else(|error| {
            problems.push(error);
            manifest::Registry::default()
        });
        let mut styles = CardStyles::default();
        for (id, text) in fronts {
            keep(
                manifest::front(id, text, &registry),
                &mut styles.fronts,
                &mut problems,
            );
        }
        for (id, text) in backs {
            keep(
                manifest::back(id, text, &registry),
                &mut styles.backs,
                &mut problems,
            );
        }
        styles.fronts.sort_by(|a, b| a.id.cmp(&b.id));
        styles.backs.sort_by(|a, b| a.id.cmp(&b.id));
        default_first(&mut styles.fronts, |style| style.id == DEFAULT_FRONT);
        default_first(&mut styles.backs, |style| style.id == DEFAULT_BACK);
        Loaded { styles, problems }
    }

    /// The embedded styles, read once. A style that fails its checks is logged and left out.
    pub fn embedded() -> &'static CardStyles {
        static STYLES: OnceLock<CardStyles> = OnceLock::new();
        STYLES.get_or_init(|| {
            let loaded = Self::load(
                embedded::EMBEDDED_FRONTS,
                embedded::EMBEDDED_BACKS,
                embedded::CODEPOINTS,
            );
            for problem in &loaded.problems {
                tracing::warn!(%problem, "a card style is left out");
            }
            loaded.styles
        })
    }

    /// The front styles, the default first.
    pub fn fronts(&self) -> &[FrontStyle] {
        &self.fronts
    }

    /// The back styles, the default first.
    pub fn backs(&self) -> &[BackStyle] {
        &self.backs
    }

    /// The front with this id, else the default front, else the first one.
    pub fn front_or_default(&self, id: &str) -> Option<&FrontStyle> {
        self.fronts
            .iter()
            .find(|style| style.id == id)
            .or_else(|| self.fronts.first())
    }

    /// The back with this id, else the default back, else the first one.
    pub fn back_or_default(&self, id: &str) -> Option<&BackStyle> {
        self.backs
            .iter()
            .find(|style| style.id == id)
            .or_else(|| self.backs.first())
    }

    /// Whether a front with this id exists.
    pub fn has_front(&self, id: &str) -> bool {
        self.fronts.iter().any(|style| style.id == id)
    }

    /// Whether a back with this id exists.
    pub fn has_back(&self, id: &str) -> bool {
        self.backs.iter().any(|style| style.id == id)
    }
}
