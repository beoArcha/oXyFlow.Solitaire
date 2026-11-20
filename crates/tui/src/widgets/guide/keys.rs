//! The `Keys`, `Mouse` and `Goal` sections of the guide panel: the keys in a table generated from
//! the key table, the mouse gestures and the goal of the game. Keys usable at this moment are
//! marked with a symbol and drawn bold, so they stand out without color.

use oxysol_core::Variant;
use ratatui::crossterm::event::KeyCode;
use ratatui::style::Modifier;
use ratatui::text::Line;

use super::Guide;
use crate::action::Action;
use crate::i18n::t;
use crate::input::{binding_for, primary_key};
use crate::screens::ViewCtx;
use crate::theme::Token;
use crate::widgets::text::{clip, wrap};

const KEY_GAP: usize = 2;
const MARKER_WIDTH: usize = 2;
const MOUSE_CLICK: &str = "board.guide.mouse.click";
const MOUSE_DOUBLE: &str = "board.guide.mouse.double_click";
const MOUSE_STOCK: &str = "board.guide.mouse.stock";
const MOUSE_RIGHT: &str = "board.guide.mouse.right_click";

struct Entry {
    actions: &'static [Action],
    label: &'static str,
}

const fn entry(actions: &'static [Action], label: &'static str) -> Entry {
    Entry { actions, label }
}

fn entries(variant: Variant, undo_allowed: bool) -> Vec<Entry> {
    let pyramid = matches!(variant, Variant::Pyramid(_));
    let mut rows = vec![
        entry(
            &[Action::Up, Action::Down, Action::Left, Action::Right],
            "board.guide.move",
        ),
        entry(
            &[Action::Select],
            if pyramid {
                "board.guide.select_pair"
            } else {
                "board.guide.select"
            },
        ),
        entry(&[Action::Back], "board.guide.back"),
        entry(&[Action::QuickMove], "board.guide.quick"),
    ];
    if !pyramid {
        rows.push(entry(
            &[Action::RunMore, Action::RunLess],
            "board.guide.run",
        ));
    }
    rows.push(entry(
        &[Action::ZoomIn, Action::ZoomOut, Action::ZoomAuto],
        "board.guide.zoom",
    ));
    match variant {
        Variant::FreeCell => {}
        Variant::Spider(_) => rows.push(entry(&[Action::Stock], "board.guide.deal_row")),
        _ => rows.push(entry(&[Action::Stock], "board.guide.stock")),
    }
    if undo_allowed {
        rows.push(entry(
            &[Action::Undo, Action::Redo],
            "board.guide.undo_redo",
        ));
    }
    rows.extend([
        entry(&[Action::Hint], "board.guide.hint"),
        entry(&[Action::CopySeed], "board.guide.copy_seed"),
        entry(&[Action::Rules], "board.guide.rules"),
        entry(&[Action::Help], "board.guide.help"),
    ]);
    rows
}

fn mouse_gestures(variant: Variant) -> Vec<&'static str> {
    let mut gestures = vec![MOUSE_CLICK, MOUSE_DOUBLE];
    gestures.extend((variant != Variant::FreeCell).then_some(MOUSE_STOCK));
    gestures.push(MOUSE_RIGHT);
    gestures
}

const fn goal_key(variant: Variant) -> &'static str {
    match variant {
        Variant::Klondike(_) => "board.guide.goal.klondike",
        Variant::Spider(_) => "board.guide.goal.spider",
        Variant::FreeCell => "board.guide.goal.freecell",
        Variant::Pyramid(_) => "board.guide.goal.pyramid",
    }
}

fn is_arrow(action: Action) -> bool {
    binding_for(action)
        .and_then(|binding| binding.keys.first())
        .is_some_and(|spec| {
            matches!(
                spec.code,
                KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right
            )
        })
}

/// The keys of a group of actions for display: `Arrows` when they are the four arrow keys, else
/// the primary key of each action joined by `/`.
fn keys_text(actions: &[Action]) -> String {
    if actions.len() > 1 && actions.iter().all(|action| is_arrow(*action)) {
        return t("board.guide.arrows").into_owned();
    }
    actions
        .iter()
        .filter_map(|action| primary_key(*action))
        .collect::<Vec<_>>()
        .join("/")
}

fn row(
    entry: &Entry,
    key_width: usize,
    width: usize,
    guide: &Guide<'_>,
    ctx: &ViewCtx<'_>,
) -> Line<'static> {
    let usable = entry
        .actions
        .iter()
        .any(|action| guide.usable.contains(action));
    let (marker, style) = if usable {
        (
            ctx.glyphs.focus_left,
            ctx.theme
                .style(Token::Text, Token::Surface)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        (" ", ctx.theme.style(Token::TextMuted, Token::Surface))
    };
    let keys = keys_text(entry.actions);
    let label_width = width.saturating_sub(MARKER_WIDTH + key_width + KEY_GAP);
    let text = format!(
        "{marker} {keys:<key_width$}{}{}",
        " ".repeat(KEY_GAP),
        clip(&t(entry.label), label_width)
    );
    Line::styled(text, style)
}

/// The lines of the `Goal` section below its heading.
pub(super) fn goal(variant: Variant, width: usize, ctx: &ViewCtx<'_>) -> Vec<Line<'static>> {
    let plain = ctx.theme.style(Token::Text, Token::Surface);
    wrap(&t(goal_key(variant)), width)
        .into_iter()
        .map(|text| Line::styled(text, plain))
        .collect()
}

/// The lines of the `Keys` section below its heading.
pub(super) fn keys(guide: &Guide<'_>, width: usize, ctx: &ViewCtx<'_>) -> Vec<Line<'static>> {
    let rows = entries(guide.variant, guide.undo_allowed);
    let key_width = rows
        .iter()
        .map(|entry| keys_text(entry.actions).chars().count())
        .max()
        .unwrap_or(0);
    rows.iter()
        .map(|entry| row(entry, key_width, width, guide, ctx))
        .collect()
}

/// The lines of the `Mouse` section below its heading.
pub(super) fn mouse(variant: Variant, width: usize, ctx: &ViewCtx<'_>) -> Vec<Line<'static>> {
    let muted = ctx.theme.style(Token::TextMuted, Token::Surface);
    mouse_gestures(variant)
        .iter()
        .map(|key| Line::styled(clip(&t(key), width), muted))
        .collect()
}
