//! The building blocks of board guidance: `Part`, `Guidance`, the fixed tail, and the closed
//! states (won, stuck).

use oxysol_core::Session;

use super::priority;
use crate::action::Action;
use crate::i18n::{t, tf};
use crate::input::primary_key;
use crate::widgets::footer::Segment;
use crate::widgets::text::clip;

/// One thing the player can do, as a short text with its key; the part with the highest
/// `drop_order` goes first when the line is too wide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Part {
    pub text: String,
    pub drop_order: u8,
    /// The action this part runs when clicked; `None` for display-only parts like `Arrows: move`.
    pub action: Option<Action>,
}

/// The moment on the board: what is in progress, what can be done, and which actions work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Guidance {
    pub now: String,
    /// Shorter wordings of `now`, longest first, for when the row is narrow.
    pub shorter: Vec<String>,
    pub possible: Vec<Part>,
    /// Parts after the possible ones: restart, options, the key list and the way out. They are
    /// dropped by `drop_order` like the others.
    pub tail: Vec<Part>,
    pub usable: Vec<Action>,
}

pub(super) const ALWAYS: [Action; 14] = [
    Action::Up,
    Action::Down,
    Action::Left,
    Action::Right,
    Action::Help,
    Action::Rules,
    Action::Guide,
    Action::Moves,
    Action::Options,
    Action::Restart,
    Action::NewGame,
    Action::Pause,
    Action::Menu,
    Action::Quit,
];

/// Builds a clickable `Do:` segment for `action` from its localized template.
pub(super) fn part(action: Action, key: &str, drop_order: u8) -> Part {
    let name = primary_key(action).unwrap_or_default();
    Part {
        text: tf(key, &[("key", &name)]),
        drop_order,
        action: Some(action),
    }
}

/// Builds a display-only `Do:` segment from its localized key.
pub(super) fn plain(key: &str, drop_order: u8) -> Part {
    Part {
        text: t(key).into_owned(),
        drop_order,
        action: None,
    }
}

/// Builds the `Do:` segment for changing the run depth.
pub(super) fn run_part(drop_order: u8) -> Part {
    let keys = [Action::RunMore, Action::RunLess]
        .into_iter()
        .filter_map(primary_key)
        .collect::<Vec<_>>()
        .join("/");
    Part {
        text: tf("board.action.run", &[("keys", &keys)]),
        drop_order,
        action: None,
    }
}

fn join(parts: &[Part]) -> String {
    parts
        .iter()
        .map(|part| part.text.as_str())
        .collect::<Vec<_>>()
        .join(&t("board.action.separator"))
}

impl Guidance {
    /// The `Now:` row text for `width` columns: the first wording that fits, else the shortest one
    /// cut to `width`.
    pub(super) fn now_line(&self, width: usize) -> String {
        let lines: Vec<String> = std::iter::once(&self.now)
            .chain(&self.shorter)
            .map(|text| tf("board.action.now", &[("text", text)]))
            .collect();
        let fitting = lines.iter().find(|line| line.chars().count() <= width);
        match (fitting, lines.last()) {
            (Some(line), _) | (None, Some(line)) => clip(line, width),
            (None, None) => String::new(),
        }
    }

    /// The `Do:` row as segments: the prefix, the parts that fit `width` columns (least useful
    /// dropped first, the tail always kept) with their separators, and the suffix. Parts with an
    /// action are buttons.
    pub(super) fn do_segments(&self, width: usize) -> Vec<Segment> {
        let template = t("board.action.possible");
        let (before, after) = template.split_once("{text}").unwrap_or(("", ""));
        let separator = t("board.action.separator");
        let mut segments = vec![Segment::plain(before)];
        for (index, part) in self.visible_parts(width).iter().enumerate() {
            if index > 0 {
                segments.push(Segment::plain(separator.as_ref()));
            }
            segments.push(match part.action {
                Some(action) => Segment::button(action, part.text.as_str()),
                None => Segment::plain(part.text.as_str()),
            });
        }
        segments.push(Segment::plain(after));
        segments
    }

    fn visible_parts(&self, width: usize) -> Vec<Part> {
        let mut parts = self.possible.clone();
        parts.extend(self.tail.iter().cloned());
        while !self.do_row_fits(&parts, width) {
            let Some(index) = parts
                .iter()
                .enumerate()
                .max_by_key(|(_, part)| part.drop_order)
                .map(|(index, _)| index)
            else {
                break;
            };
            parts.remove(index);
        }
        parts
    }

    fn do_row_fits(&self, parts: &[Part], width: usize) -> bool {
        tf("board.action.possible", &[("text", &join(parts))])
            .chars()
            .count()
            <= width
    }

    fn closed(now_key: &str, possible: Vec<Part>, mut usable: Vec<Action>) -> Self {
        usable.extend(ALWAYS);
        Self {
            now: t(now_key).into_owned(),
            shorter: Vec::new(),
            possible,
            tail: tail(true),
            usable,
        }
    }
}

/// The parts that end every `Do:` row: restart, options, pause and moves, the key list, and unless
/// Esc is taken by a pick, the way to the menu, which is kept longest of the tail.
pub(super) fn tail(with_menu: bool) -> Vec<Part> {
    let mut parts = vec![
        part(Action::Restart, "board.action.restart", priority::RESTART),
        part(Action::Options, "board.action.options", priority::OPTIONS),
        part(Action::Pause, "board.action.pause", priority::PAUSE),
        part(Action::Moves, "board.action.moves", priority::MOVES),
        part(Action::Help, "board.action.help", priority::HELP),
    ];
    if with_menu {
        parts.push(part(Action::Back, "board.action.menu", priority::MENU));
    }
    parts
}

/// Guidance for a won board.
pub(super) fn won() -> Guidance {
    Guidance::closed(
        "board.action.won",
        vec![part(
            Action::NewGame,
            "board.action.new_game",
            priority::PRIMARY,
        )],
        Vec::new(),
    )
}

/// Guidance for a stuck board without legal moves.
pub(super) fn stuck(session: &Session, undo_allowed: bool) -> Guidance {
    let mut possible = Vec::new();
    let mut usable = Vec::new();
    if undo_allowed && session.can_undo() {
        possible.push(part(Action::Undo, "board.action.undo", priority::UNDO));
        usable.push(Action::Undo);
    }
    possible.push(part(
        Action::NewGame,
        "board.action.new_game",
        priority::PRIMARY,
    ));
    Guidance::closed("board.action.stuck", possible, usable)
}
