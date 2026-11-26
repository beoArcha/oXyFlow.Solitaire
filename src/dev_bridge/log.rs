//! Append-only JSON-lines trace log (`--dev-log <path>`) for `--dev-bridge` sessions.

use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::Path;

use oxysol_tui::{FocusSnapshot, OverlayKind, ScreenKind, UiSnapshot};
use serde::Serialize;

use super::BridgeError;
use super::protocol::{BridgeEvent, Response};

/// Append-only JSON-lines writer for bridge post-mortem inspection.
pub struct DevLog {
    writer: BufWriter<File>,
}

/// One structured entry written as a single JSON line to `--dev-log`.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LogEntry {
    Command {
        frame_id: Option<u64>,
        cmd: Option<String>,
        raw: String,
    },
    Transition {
        frame_id: u64,
        event: &'static str,
        #[serde(skip_serializing_if = "Option::is_none")]
        from: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        to: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cols: Option<u16>,
        #[serde(skip_serializing_if = "Option::is_none")]
        rows: Option<u16>,
    },
    Focus {
        frame_id: u64,
        from: Option<FocusSnapshot>,
        to: Option<FocusSnapshot>,
    },
    Move {
        frame_id: u64,
        accepted: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        moves: Option<usize>,
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    Error {
        frame_id: Option<u64>,
        source: &'static str,
        message: String,
    },
}

impl DevLog {
    /// Opens or creates `path` in append mode, failing immediately if unwritable.
    pub fn open(path: &Path) -> Result<Self, BridgeError> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|source| BridgeError::LogOpen {
                path: path.display().to_string(),
                source,
            })?;
        Ok(Self {
            writer: BufWriter::new(file),
        })
    }

    /// Appends structured entries for one processed command and its response, then flushes.
    pub fn record_step(
        &mut self,
        raw: &str,
        cmd: Option<&str>,
        was_started: bool,
        after_state: Option<&UiSnapshot>,
        response: &Response,
    ) -> io::Result<()> {
        let cmd_entry = LogEntry::Command {
            frame_id: response.frame_id,
            cmd: cmd.map(str::to_owned),
            raw: raw.to_owned(),
        };
        self.write_entry(&cmd_entry)?;

        if let Some(err) = &response.error {
            let err_entry = LogEntry::Error {
                frame_id: response.frame_id,
                source: "protocol",
                message: err.clone(),
            };
            self.write_entry(&err_entry)?;
        }

        let Some(frame_id) = response.frame_id else {
            return self.writer.flush();
        };
        if cmd == Some("start") && response.ok {
            self.record_start(
                frame_id,
                was_started,
                after_state.or(response.state.as_ref()),
            )?;
        }
        for event in &response.events {
            self.record_event(frame_id, event)?;
        }
        self.writer.flush()
    }

    fn record_start(
        &mut self,
        frame_id: u64,
        was_started: bool,
        state: Option<&UiSnapshot>,
    ) -> io::Result<()> {
        let screen = state.map(|snap| screen_name(snap.screen).to_owned());
        self.write_entry(&LogEntry::Transition {
            frame_id,
            event: if was_started { "restarted" } else { "started" },
            from: None,
            to: screen,
            cols: None,
            rows: None,
        })?;
        if let Some(snap) = state
            && let Some(focus) = &snap.focus
        {
            self.write_entry(&LogEntry::Focus {
                frame_id,
                from: None,
                to: Some(focus.clone()),
            })?;
        }
        Ok(())
    }

    fn record_event(&mut self, frame_id: u64, event: &BridgeEvent) -> io::Result<()> {
        match event {
            BridgeEvent::ScreenChanged { from, to } => self.write_entry(&LogEntry::Transition {
                frame_id,
                event: "screen_changed",
                from: Some(screen_name(*from).to_owned()),
                to: Some(screen_name(*to).to_owned()),
                cols: None,
                rows: None,
            }),
            BridgeEvent::ModeChanged { from, to } => self.write_entry(&LogEntry::Transition {
                frame_id,
                event: "mode_changed",
                from: from.clone(),
                to: to.clone(),
                cols: None,
                rows: None,
            }),
            BridgeEvent::OverlayChanged { from, to } => self.write_entry(&LogEntry::Transition {
                frame_id,
                event: "overlay_changed",
                from: from.map(|kind| overlay_name(kind).to_owned()),
                to: to.map(|kind| overlay_name(kind).to_owned()),
                cols: None,
                rows: None,
            }),
            BridgeEvent::FocusChanged { from, to } => self.write_entry(&LogEntry::Focus {
                frame_id,
                from: from.clone(),
                to: to.clone(),
            }),
            BridgeEvent::HoverChanged { from, to } => self.write_entry(&LogEntry::Transition {
                frame_id,
                event: "hover_changed",
                from: from.map(|pos| format!("{},{}", pos.x, pos.y)),
                to: to.map(|pos| format!("{},{}", pos.x, pos.y)),
                cols: None,
                rows: None,
            }),
            BridgeEvent::MoveApplied { moves } => self.write_entry(&LogEntry::Move {
                frame_id,
                accepted: true,
                moves: Some(*moves),
                message: None,
            }),
            BridgeEvent::MoveRejected { message } => {
                self.write_entry(&LogEntry::Move {
                    frame_id,
                    accepted: false,
                    moves: None,
                    message: Some(message.clone()),
                })?;
                self.write_entry(&LogEntry::Error {
                    frame_id: Some(frame_id),
                    source: "engine",
                    message: message.clone(),
                })
            }
            BridgeEvent::StatusChanged { message } => self.write_entry(&LogEntry::Transition {
                frame_id,
                event: "status_changed",
                from: None,
                to: Some(message.clone()),
                cols: None,
                rows: None,
            }),
            BridgeEvent::GameWon => self.write_entry(&LogEntry::Transition {
                frame_id,
                event: "game_won",
                from: None,
                to: None,
                cols: None,
                rows: None,
            }),
            BridgeEvent::Resized { cols, rows } => self.write_entry(&LogEntry::Transition {
                frame_id,
                event: "resized",
                from: None,
                to: None,
                cols: Some(*cols),
                rows: Some(*rows),
            }),
            BridgeEvent::Quit => self.write_entry(&LogEntry::Transition {
                frame_id,
                event: "quit",
                from: None,
                to: None,
                cols: None,
                rows: None,
            }),
        }
    }

    fn write_entry(&mut self, entry: &LogEntry) -> io::Result<()> {
        let json = serde_json::to_string(entry).map_err(io::Error::other)?;
        writeln!(self.writer, "{json}")
    }
}

fn screen_name(screen: ScreenKind) -> &'static str {
    match screen {
        ScreenKind::MainMenu => "main_menu",
        ScreenKind::GameSelect => "game_select",
        ScreenKind::Board => "board",
        ScreenKind::Settings => "settings",
        ScreenKind::Stats => "stats",
    }
}

fn overlay_name(overlay: OverlayKind) -> &'static str {
    match overlay {
        OverlayKind::Help => "help",
        OverlayKind::HowToPlay => "how_to_play",
        OverlayKind::CommandLine => "command_line",
        OverlayKind::Guide => "guide",
        OverlayKind::Moves => "moves",
        OverlayKind::Quick => "quick",
        OverlayKind::Pause => "pause",
        OverlayKind::Confirm => "confirm",
        OverlayKind::GraphicsCheck => "graphics_check",
        OverlayKind::Alert => "alert",
        OverlayKind::Search => "search",
        OverlayKind::Win => "win",
    }
}
