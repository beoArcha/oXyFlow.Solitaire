//! The rows of the game menu: the favourites group first, then one group per family, each with a
//! heading. A favourite appears only in the favourites group. A search text keeps the variants
//! whose name contains it, ignoring case, and drops the groups left empty.

use oxysol_core::Variant;

use super::catalog::{Family, variants_of};
use crate::i18n::t;
use crate::screens::variant_label_key;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Heading {
    Favourites,
    Family(Family),
}

impl Heading {
    pub(super) const fn title_key(self) -> &'static str {
        match self {
            Heading::Favourites => "menu.favourites",
            Heading::Family(family) => family.title_key(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Row {
    Heading(Heading),
    Item(Variant),
}

fn matches_query(variant: Variant, query: &str) -> bool {
    let needle = query.trim().to_lowercase();
    needle.is_empty()
        || t(variant_label_key(variant))
            .to_lowercase()
            .contains(&needle)
}

fn group(rows: &mut Vec<Row>, heading: Heading, variants: Vec<Variant>) {
    if variants.is_empty() {
        return;
    }
    rows.push(Row::Heading(heading));
    rows.extend(variants.into_iter().map(Row::Item));
}

/// The rows for the given favourites and search text.
pub(super) fn build(favourites: &[Variant], query: &str) -> Vec<Row> {
    let shown = |variant: &Variant| matches_query(*variant, query);
    let mut rows = Vec::new();
    let starred: Vec<Variant> = Variant::ALL
        .into_iter()
        .filter(|variant| favourites.contains(variant) && shown(variant))
        .collect();
    group(&mut rows, Heading::Favourites, starred);
    for family in Family::ALL {
        let members: Vec<Variant> = variants_of(family)
            .filter(|variant| !favourites.contains(variant) && shown(variant))
            .collect();
        group(&mut rows, Heading::Family(family), members);
    }
    rows
}

/// The focusable variants of `rows`, top to bottom.
pub(super) fn items(rows: &[Row]) -> Vec<Variant> {
    rows.iter()
        .filter_map(|row| match row {
            Row::Item(variant) => Some(*variant),
            Row::Heading(_) => None,
        })
        .collect()
}
