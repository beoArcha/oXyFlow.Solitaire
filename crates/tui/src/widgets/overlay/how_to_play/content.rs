//! The text of a page as display rows: section headings, word-wrapped paragraphs and bullets, and
//! the controls with their keys read from the key table, so a rebound key shows up here too.

use super::pages::{Control, page};
use super::state::Rules;
use crate::action::Action;
use crate::i18n::{t, tf};
use crate::input::{binding_for, primary_key};
pub use crate::widgets::text::wrap;

/// Rows of text visible at once at 100 by 30, the smallest window: the first value before any event.
pub const VIEW_ROWS: usize = 20;
/// Columns of text per row.
pub const TEXT_WIDTH: usize = 76;

const BULLET: &str = "- ";
const KEY_GAP: usize = 2;
const GROUP_SEPARATOR: &str = "/";
const ALTERNATIVE_SEPARATOR: &str = " / ";

/// How a row is styled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Heading,
    Body,
    Key,
}

/// One display row: a lead (bullet or keys, or blanks for a continuation) and the text after it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub lead: String,
    pub text: String,
    pub kind: Kind,
}

fn placeholders() -> Vec<(&'static str, String)> {
    [
        ("stock", Action::Stock),
        ("hint", Action::Hint),
        ("undo", Action::Undo),
        ("quick", Action::QuickMove),
        ("rules", Action::Rules),
    ]
    .into_iter()
    .map(|(name, action)| (name, primary_key(action).unwrap_or_default()))
    .collect()
}

/// The keys of `actions` for display: every key of a lone action (`Enter / Space`), the primary
/// key of each action of a group (`Left/Right/Up/Down`).
pub fn keys_of(actions: &[Action]) -> String {
    match actions {
        [single] => binding_for(*single)
            .map(|binding| {
                binding
                    .keys
                    .iter()
                    .map(|spec| spec.display())
                    .collect::<Vec<_>>()
                    .join(ALTERNATIVE_SEPARATOR)
            })
            .unwrap_or_default(),
        many => many
            .iter()
            .filter_map(|action| primary_key(*action))
            .collect::<Vec<_>>()
            .join(GROUP_SEPARATOR),
    }
}

fn push_led(rows: &mut Vec<Row>, lead: &str, kind: Kind, text: &str) {
    let lead_width = lead.chars().count();
    let lines = wrap(text, TEXT_WIDTH.saturating_sub(lead_width));
    for (index, line) in lines.into_iter().enumerate() {
        let first = index == 0;
        rows.push(Row {
            lead: if first {
                lead.to_owned()
            } else {
                " ".repeat(lead_width)
            },
            text: line,
            kind: if first { kind } else { Kind::Body },
        });
    }
}

fn push_heading(rows: &mut Vec<Row>, key: &str) {
    if !rows.is_empty() {
        rows.push(Row {
            lead: String::new(),
            text: String::new(),
            kind: Kind::Body,
        });
    }
    rows.push(Row {
        lead: String::new(),
        text: t(key).into_owned(),
        kind: Kind::Heading,
    });
}

fn push_paragraphs(rows: &mut Vec<Row>, keys: &[&str], bullets: bool, params: &[(&str, &str)]) {
    let lead = if bullets { BULLET } else { "" };
    for key in keys {
        push_led(rows, lead, Kind::Body, &tf(key, params));
    }
}

fn push_controls(rows: &mut Vec<Row>, list: &[Control], note: &str, params: &[(&str, &str)]) {
    let keys: Vec<String> = list
        .iter()
        .map(|control| keys_of(control.actions))
        .collect();
    let key_width = keys
        .iter()
        .map(|keys| keys.chars().count())
        .max()
        .unwrap_or(0)
        + KEY_GAP;
    for (control, keys) in list.iter().zip(&keys) {
        let lead = format!("{keys:<key_width$}");
        push_led(rows, &lead, Kind::Key, &tf(control.text, params));
    }
    push_led(rows, "", Kind::Body, &tf(note, params));
}

/// Every row of the page of `rules`: goal, layout, moves, controls and tips.
pub fn rows(rules: Rules) -> Vec<Row> {
    let page = page(rules);
    let owned = placeholders();
    let params: Vec<(&str, &str)> = owned
        .iter()
        .map(|(name, value)| (*name, value.as_str()))
        .collect();
    let mut rows = Vec::new();
    push_heading(&mut rows, "rules.section.goal");
    push_paragraphs(&mut rows, page.goal, false, &params);
    push_heading(&mut rows, "rules.section.layout");
    push_paragraphs(&mut rows, page.layout, true, &params);
    push_heading(&mut rows, "rules.section.moves");
    push_paragraphs(&mut rows, page.moves, true, &params);
    push_heading(&mut rows, "rules.section.controls");
    push_controls(&mut rows, page.controls, page.controls_note, &params);
    push_heading(&mut rows, "rules.section.tips");
    push_paragraphs(&mut rows, page.tips, true, &params);
    rows
}
