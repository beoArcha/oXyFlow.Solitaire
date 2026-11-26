//! The rows of the game menu: the favourites group first, then one group per family, each with a
//! heading. A favourite appears only in the favourites group. A search text keeps the variants
//! whose name contains it, ignoring case, opens every group with a match without changing what is
//! folded in settings, and drops the groups left empty.

use oxysol_core::Variant;

use super::catalog::{Family, variants_of};
use crate::i18n::{t, tf};
use crate::screens::variant_label_key;
use crate::settings::GameGroup;

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

    pub(super) const fn group(self) -> GameGroup {
        match self {
            Heading::Favourites => GameGroup::Favourites,
            Heading::Family(family) => family.group(),
        }
    }

    pub(super) fn of(variant: Variant, favourites: &[Variant]) -> Self {
        if favourites.contains(&variant) {
            Self::Favourites
        } else {
            Self::Family(Family::of(variant))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Row {
    Heading {
        heading: Heading,
        folded: bool,
        count: usize,
    },
    Item(Variant),
    /// An empty line below the favourites group, before the first family.
    Gap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FocusItem {
    Heading(Heading),
    Variant(Variant),
}

impl Row {
    pub(super) const fn focus_item(self) -> Option<FocusItem> {
        match self {
            Row::Heading { heading, .. } => Some(FocusItem::Heading(heading)),
            Row::Item(variant) => Some(FocusItem::Variant(variant)),
            Row::Gap => None,
        }
    }

    pub(super) const fn is_focusable(self) -> bool {
        !matches!(self, Row::Gap)
    }
}

pub(super) fn heading_text(heading: Heading, folded: bool, count: usize) -> String {
    let title = t(heading.title_key());
    if folded {
        let count_text = count.to_string();
        let label = tf(
            "menu.family_folded",
            &[("title", &title), ("count", &count_text)],
        );
        format!("[+] {label}")
    } else {
        format!("[-] {title}")
    }
}

fn matches_query(variant: Variant, query: &str) -> bool {
    let needle = query.trim().to_lowercase();
    needle.is_empty()
        || t(variant_label_key(variant))
            .to_lowercase()
            .contains(&needle)
}

fn group(
    rows: &mut Vec<Row>,
    heading: Heading,
    variants: Vec<Variant>,
    folded: &[GameGroup],
    filtering: bool,
) {
    if variants.is_empty() {
        return;
    }
    let is_folded = !filtering && folded.contains(&heading.group());
    rows.push(Row::Heading {
        heading,
        folded: is_folded,
        count: variants.len(),
    });
    if !is_folded {
        rows.extend(variants.into_iter().map(Row::Item));
    }
}

/// The rows for the given favourites, folded groups and search text.
pub(super) fn build(favourites: &[Variant], folded: &[GameGroup], query: &str) -> Vec<Row> {
    let filtering = !query.trim().is_empty();
    let shown = |variant: &Variant| matches_query(*variant, query);
    let mut rows = Vec::new();
    let starred: Vec<Variant> = Variant::ALL
        .into_iter()
        .filter(|variant| favourites.contains(variant) && shown(variant))
        .collect();
    group(&mut rows, Heading::Favourites, starred, folded, filtering);
    let mut families = Vec::new();
    for family in Family::ALL {
        let members: Vec<Variant> = variants_of(family)
            .filter(|variant| !favourites.contains(variant) && shown(variant))
            .collect();
        group(
            &mut families,
            Heading::Family(family),
            members,
            folded,
            filtering,
        );
    }
    if !rows.is_empty() && !families.is_empty() {
        rows.push(Row::Gap);
    }
    rows.extend(families);
    rows
}

/// The visible variants of `rows`, top to bottom.
pub(super) fn items(rows: &[Row]) -> Vec<Variant> {
    rows.iter()
        .filter_map(|row| match row {
            Row::Item(variant) => Some(*variant),
            Row::Heading { .. } | Row::Gap => None,
        })
        .collect()
}

/// Index in `rows` of the first variant, or first focusable row if all visible groups are folded.
pub(super) fn default_position(rows: &[Row]) -> Option<usize> {
    rows.iter()
        .position(|row| matches!(row, Row::Item(_)))
        .or_else(|| rows.iter().position(|row| row.is_focusable()))
}

/// Resolves `focused` to its row index in `rows`, falling back to its group heading if folded,
/// or the default row.
pub(super) fn resolve_position(
    rows: &[Row],
    focused: Option<FocusItem>,
    favourites: &[Variant],
) -> Option<usize> {
    if let Some(target) = focused {
        if let Some(pos) = rows.iter().position(|row| row.focus_item() == Some(target)) {
            return Some(pos);
        }
        if let FocusItem::Variant(variant) = target {
            let group_heading = FocusItem::Heading(Heading::of(variant, favourites));
            if let Some(pos) = rows
                .iter()
                .position(|row| row.focus_item() == Some(group_heading))
            {
                return Some(pos);
            }
        }
    }
    default_position(rows)
}
