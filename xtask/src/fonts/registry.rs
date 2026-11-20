//! The card glyph registry `assets/cards/codepoints.toml`: keys, code points and reserved ranges (D-06, T-16).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use super::FontsError;

const FIRST_CODE: u32 = 0xE6BC;
const LAST_CODE: u32 = 0xE6FF;

/// Registry keys (`<fronts|backs>/<style>/<name>`) and their code points.
pub type Glyphs = BTreeMap<String, u32>;

pub fn read_registry(path: &Path) -> Result<Glyphs, FontsError> {
    let text = fs::read_to_string(path).map_err(|source| FontsError::Io {
        path: path.to_owned(),
        source,
    })?;
    parse_registry(path, &text)
}

pub fn parse_registry(path: &Path, text: &str) -> Result<Glyphs, FontsError> {
    let table: toml::Table = text
        .parse()
        .map_err(|error| registry_error(path, format!("not valid TOML: {error}")))?;
    let entries = table
        .get("glyphs")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| registry_error(path, "there is no [glyphs] table".to_owned()))?;
    let mut glyphs = Glyphs::new();
    let mut owners: BTreeMap<u32, String> = BTreeMap::new();
    for (key, value) in entries {
        if !valid_key(key) {
            return Err(registry_error(
                path,
                format!("`{key}` is not `<fronts|backs>/<style>/<name>` in lowercase ASCII"),
            ));
        }
        let code = value.as_str().and_then(parse_code).ok_or_else(|| {
            registry_error(
                path,
                format!("`{key}` needs a hexadecimal code point within U+E6BC–U+E6FF"),
            )
        })?;
        if let Some(other) = owners.insert(code, key.clone()) {
            return Err(registry_error(
                path,
                format!("`{key}` and `{other}` both use U+{code:04X}; a code point is used once"),
            ));
        }
        glyphs.insert(key.clone(), code);
    }
    check_reserved(path, table.get("reserved"))?;
    Ok(glyphs)
}

fn check_reserved(path: &Path, reserved: Option<&toml::Value>) -> Result<(), FontsError> {
    let Some(reserved) = reserved else {
        return Ok(());
    };
    let ranges = reserved.as_array().ok_or_else(|| {
        registry_error(path, "[[reserved]] must be an array of tables".to_owned())
    })?;
    if ranges.iter().all(reserved_range_is_valid) {
        Ok(())
    } else {
        Err(registry_error(
            path,
            "a [[reserved]] range needs `first` and `last` code points within U+E6BC–U+E6FF, first ≤ last"
                .to_owned(),
        ))
    }
}

fn reserved_range_is_valid(range: &toml::Value) -> bool {
    let Some(table) = range.as_table() else {
        return false;
    };
    let first = table
        .get("first")
        .and_then(toml::Value::as_str)
        .and_then(parse_code);
    let last = table
        .get("last")
        .and_then(toml::Value::as_str)
        .and_then(parse_code);
    matches!((first, last), (Some(first), Some(last)) if first <= last)
}

fn registry_error(path: &Path, reason: String) -> FontsError {
    FontsError::Registry {
        path: path.to_owned(),
        reason,
    }
}

fn parse_code(text: &str) -> Option<u32> {
    let code = u32::from_str_radix(text, 16).ok()?;
    (FIRST_CODE..=LAST_CODE).contains(&code).then_some(code)
}

fn valid_key(key: &str) -> bool {
    let parts: Vec<&str> = key.split('/').collect();
    matches!(
        parts.as_slice(),
        [group, style, name] if matches!(*group, "fronts" | "backs") && valid_name(style) && valid_name(name)
    )
}

pub(super) fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|first| first.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

#[cfg(test)]
mod tests {
    use super::{FontsError, parse_registry};
    use std::path::Path;

    const SEED_REGISTRY: &str = include_str!("../../../assets/cards/codepoints.toml");

    fn registry_path() -> &'static Path {
        Path::new("codepoints.toml")
    }

    #[test]
    fn test_seed_registry_has_the_style_guide_table() {
        let glyphs = parse_registry(registry_path(), SEED_REGISTRY).unwrap();
        assert_eq!(glyphs.len(), 25);
        assert_eq!(glyphs.get("fronts/international/spade"), Some(&0xE6BC));
        assert_eq!(glyphs.get("fronts/italian/coin"), Some(&0xE6C7));
        assert_eq!(glyphs.get("fronts/international/king"), Some(&0xE6D2));
        assert_eq!(glyphs.get("backs/ornament/mosaic-br"), Some(&0xE6E1));
    }

    #[test]
    fn test_registry_rejects_code_point_outside_range() {
        let text = "[glyphs]\n\"fronts/international/spade\" = \"E6BB\"\n";
        assert!(rejection(text).contains("U+E6BC–U+E6FF"));
        let plane16 = "[glyphs]\n\"fronts/international/spade\" = \"10E6C0\"\n";
        assert!(rejection(plane16).contains("fronts/international/spade"));
    }

    #[test]
    fn test_registry_rejects_duplicate_code_point() {
        let text = "[glyphs]\n\"fronts/international/spade\" = \"E6BC\"\n\"fronts/international/heart\" = \"E6BC\"\n";
        let reason = rejection(text);
        assert!(reason.contains("both use U+E6BC"), "{reason}");
    }

    #[test]
    fn test_registry_rejects_bad_key() {
        let text = "[glyphs]\n\"fronts/International/spade\" = \"E6BC\"\n";
        assert!(rejection(text).contains("fronts/International/spade"));
        let wrong_group = "[glyphs]\n\"tiles/international/spade\" = \"E6BC\"\n";
        assert!(rejection(wrong_group).contains("tiles/international/spade"));
    }

    #[test]
    fn test_registry_needs_glyphs_table_and_valid_reserved_ranges() {
        assert!(rejection("title = \"x\"\n").contains("[glyphs]"));
        let reversed = "[glyphs]\n\"backs/lattice/tile\" = \"E6D8\"\n\n[[reserved]]\nfirst = \"E6FF\"\nlast = \"E6E0\"\n";
        assert!(rejection(reversed).contains("[[reserved]]"));
    }

    fn rejection(text: &str) -> String {
        match parse_registry(registry_path(), text) {
            Err(FontsError::Registry { reason, .. }) => reason,
            other => panic!("expected a registry error, got {other:?}"),
        }
    }
}
