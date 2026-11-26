//! JSON-lines wire protocol types and snapshot event diffing for `--dev-bridge`.

use oxysol_tui::{CellPos, FocusSnapshot, OverlayKind, ScreenKind, UiSnapshot};
use serde::{Deserialize, Serialize};

const fn default_cols() -> u16 {
    120
}

const fn default_rows() -> u16 {
    40
}

const fn default_true() -> bool {
    true
}

fn default_colors() -> String {
    "truecolor".to_owned()
}

fn default_lang() -> String {
    "en-US".to_owned()
}

fn default_theme() -> String {
    "dark".to_owned()
}

/// Parameters of the `start` command.
#[derive(Debug, Clone, Deserialize)]
pub struct StartArgs {
    #[serde(default = "default_cols")]
    pub cols: u16,
    #[serde(default = "default_rows")]
    pub rows: u16,
    pub game: Option<String>,
    pub seed: Option<u64>,
    #[serde(default)]
    pub ascii: bool,
    #[serde(default = "default_colors")]
    pub colors: String,
    #[serde(default = "default_lang")]
    pub lang: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub include_text: bool,
    #[serde(default = "default_true")]
    pub include_state: bool,
}

/// Parameters of the `key` command.
#[derive(Debug, Clone, Deserialize)]
pub struct KeyArgs {
    pub key: String,
    #[serde(default)]
    pub modifiers: Vec<String>,
    #[serde(default)]
    pub include_text: bool,
    #[serde(default = "default_true")]
    pub include_state: bool,
}

/// Parameters of the `mouse` command.
#[derive(Debug, Clone, Deserialize)]
pub struct MouseArgs {
    pub action: String,
    pub x: u16,
    pub y: u16,
    #[serde(default)]
    pub modifiers: Vec<String>,
    #[serde(default)]
    pub include_text: bool,
    #[serde(default = "default_true")]
    pub include_state: bool,
}

/// Parameters of the `resize` command.
#[derive(Debug, Clone, Deserialize)]
pub struct ResizeArgs {
    pub cols: u16,
    pub rows: u16,
    #[serde(default)]
    pub include_text: bool,
    #[serde(default = "default_true")]
    pub include_state: bool,
}

/// Parameters of the `tick` command.
#[derive(Debug, Clone, Deserialize)]
pub struct TickArgs {
    #[serde(default)]
    pub ms: u64,
    #[serde(default)]
    pub include_text: bool,
    #[serde(default = "default_true")]
    pub include_state: bool,
}

/// Parameters of the `snapshot` command.
#[derive(Debug, Clone, Deserialize)]
pub struct SnapshotArgs {
    #[serde(default = "default_true")]
    pub include_text: bool,
    #[serde(default = "default_true")]
    pub include_state: bool,
}

/// Incoming JSON-line command.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum Command {
    Start(StartArgs),
    Key(KeyArgs),
    Mouse(MouseArgs),
    Resize(ResizeArgs),
    Tick(TickArgs),
    Snapshot(SnapshotArgs),
    Quit,
}

/// High-level state transitions observed during a step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BridgeEvent {
    ScreenChanged {
        from: ScreenKind,
        to: ScreenKind,
    },
    ModeChanged {
        from: Option<String>,
        to: Option<String>,
    },
    OverlayChanged {
        from: Option<OverlayKind>,
        to: Option<OverlayKind>,
    },
    FocusChanged {
        from: Option<FocusSnapshot>,
        to: Option<FocusSnapshot>,
    },
    HoverChanged {
        from: Option<CellPos>,
        to: Option<CellPos>,
    },
    MoveApplied {
        moves: usize,
    },
    MoveRejected {
        message: String,
    },
    StatusChanged {
        message: String,
    },
    GameWon,
    Resized {
        cols: u16,
        rows: u16,
    },
    Quit,
}

/// Outgoing JSON-line response.
#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub frame_id: Option<u64>,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<UiSnapshot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_frame: Option<String>,
    pub events: Vec<BridgeEvent>,
}

impl Response {
    pub(super) fn error(frame_id: Option<u64>, message: impl Into<String>) -> Self {
        Self {
            frame_id,
            ok: false,
            error: Some(message.into()),
            state: None,
            text_frame: None,
            events: Vec::new(),
        }
    }
}

/// Computes semantic state-transition events between two snapshots.
pub fn diff_events(before: &UiSnapshot, after: &UiSnapshot) -> Vec<BridgeEvent> {
    let mut events = Vec::new();
    if before.screen != after.screen {
        events.push(BridgeEvent::ScreenChanged {
            from: before.screen,
            to: after.screen,
        });
    }
    if before.mode != after.mode {
        events.push(BridgeEvent::ModeChanged {
            from: before.mode.clone(),
            to: after.mode.clone(),
        });
    }
    if before.active_overlay != after.active_overlay {
        events.push(BridgeEvent::OverlayChanged {
            from: before.active_overlay,
            to: after.active_overlay,
        });
    }
    if before.focus != after.focus {
        events.push(BridgeEvent::FocusChanged {
            from: before.focus.clone(),
            to: after.focus.clone(),
        });
    }
    if before.hover != after.hover {
        events.push(BridgeEvent::HoverChanged {
            from: before.hover,
            to: after.hover,
        });
    }
    push_game_events(before, after, &mut events);
    if after.last_error != before.last_error
        && let Some(message) = &after.last_error
    {
        events.push(BridgeEvent::MoveRejected {
            message: message.clone(),
        });
    }
    if after.status_line != before.status_line
        && let Some(message) = &after.status_line
    {
        events.push(BridgeEvent::StatusChanged {
            message: message.clone(),
        });
    }
    events
}

fn push_game_events(before: &UiSnapshot, after: &UiSnapshot, events: &mut Vec<BridgeEvent>) {
    let Some(after_stats) = after.game_stats else {
        return;
    };
    let prev_moves = before.game_stats.map_or(0, |stats| stats.moves);
    let prev_won = before.game_stats.is_some_and(|stats| stats.won);
    if before.game_stats.is_some() && after_stats.moves != prev_moves {
        events.push(BridgeEvent::MoveApplied {
            moves: after_stats.moves,
        });
    }
    if !prev_won && after_stats.won {
        events.push(BridgeEvent::GameWon);
    }
}
