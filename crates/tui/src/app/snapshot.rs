//! Read-only, serializable UI state snapshot for the headless dev bridge (decision T-42).

use oxysol_core::{PileRef, Variant};
use ratatui::layout::Rect;
use serde::Serialize;

use super::{App, Modal, Screen};
use crate::action::Action;
use crate::i18n::t;
use crate::screens::game_select::DevGameSelect;
use crate::screens::settings::tabs::Tab;
use crate::screens::variant_label_key;
use crate::settings::DockSide;
use crate::widgets::overlay;

pub(crate) struct DevMainMenu {
    pub mode: &'static str,
    pub focus_target: &'static str,
    pub focus_index: Option<usize>,
    pub label_key: Option<&'static str>,
    pub tooltip_visible: bool,
    pub notice_key: Option<&'static str>,
    pub error_key: Option<&'static str>,
    pub game_select: Option<DevGameSelect>,
}

impl DevMainMenu {
    pub(crate) fn items(
        (picker_open, selector_focus, banner_yes): (bool, bool, Option<bool>),
        index: usize,
        label_key: Option<&'static str>,
        tooltip_visible: bool,
    ) -> Self {
        let (mode, focus_target, focus_index, key) = if picker_open {
            ("language_picker", "language_selector_list", None, None)
        } else if selector_focus {
            ("items", "language_selector", None, None)
        } else if let Some(yes) = banner_yes {
            let target = if yes { "banner_yes" } else { "banner_not_now" };
            ("items", target, None, None)
        } else {
            ("items", "menu_item", Some(index), label_key)
        };
        Self {
            mode,
            focus_target,
            focus_index,
            label_key: key,
            tooltip_visible,
            notice_key: None,
            error_key: None,
            game_select: None,
        }
    }

    pub(crate) fn game(mode: &'static str, state: DevGameSelect) -> Self {
        Self {
            mode,
            focus_target: state.focus_target,
            focus_index: state.focus_index,
            label_key: None,
            tooltip_visible: false,
            notice_key: None,
            error_key: None,
            game_select: Some(state),
        }
    }

    pub(crate) fn seed(
        (mode, focus_target, focus_index, notice_key, error_key): (
            &'static str,
            &'static str,
            Option<usize>,
            Option<&'static str>,
            Option<&'static str>,
        ),
    ) -> Self {
        Self {
            mode,
            focus_target,
            focus_index,
            label_key: None,
            tooltip_visible: false,
            notice_key,
            error_key,
            game_select: None,
        }
    }
}

/// Serializable mirror of [`Screen`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScreenKind {
    MainMenu,
    GameSelect,
    Board,
    Settings,
    Stats,
}

impl From<Screen> for ScreenKind {
    fn from(screen: Screen) -> Self {
        match screen {
            Screen::MainMenu => Self::MainMenu,
            Screen::GameSelect => Self::GameSelect,
            Screen::Board => Self::Board,
            Screen::Settings => Self::Settings,
            Screen::Stats => Self::Stats,
        }
    }
}

/// Which modal dialog is open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OverlayKind {
    Help,
    HowToPlay,
    CommandLine,
    Guide,
    Moves,
    Quick,
    Pause,
    Confirm,
    GraphicsCheck,
    Alert,
    Search,
    Win,
}

impl OverlayKind {
    fn of(modal: &Modal) -> Self {
        match modal {
            Modal::Help(_) => Self::Help,
            Modal::HowToPlay(_) => Self::HowToPlay,
            Modal::CommandLine(_) => Self::CommandLine,
            Modal::Guide(_) => Self::Guide,
            Modal::Moves(_) => Self::Moves,
            Modal::Quick(_) => Self::Quick,
            Modal::Pause { .. } => Self::Pause,
            Modal::Confirm { .. } => Self::Confirm,
            Modal::GraphicsCheck(_) => Self::GraphicsCheck,
            Modal::Alert { .. } => Self::Alert,
            Modal::Search(_) => Self::Search,
            Modal::Win { .. } => Self::Win,
        }
    }
}

/// Rectangle in terminal cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct RectSnapshot {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl From<Rect> for RectSnapshot {
    fn from(r: Rect) -> Self {
        Self {
            x: r.x,
            y: r.y,
            width: r.width,
            height: r.height,
        }
    }
}

/// Cell coordinate on the terminal grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct CellPos {
    pub x: u16,
    pub y: u16,
}

/// Docked help window placement and focus state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct DockSnapshot {
    pub side: DockSide,
    pub docked: bool,
    pub focused: bool,
}

/// Which element currently holds keyboard focus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FocusSnapshot {
    pub area: String,
    pub target: String,
    pub index: Option<usize>,
    pub label: Option<String>,
}

/// Active board game counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct GameStats {
    pub variant: Variant,
    pub seed: u64,
    pub moves: usize,
    pub elapsed_ms: u64,
    pub undo_allowed: bool,
    pub restarts: u32,
    pub won: bool,
}

/// Summary of one board action from the board's move list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LegalAction {
    pub action: String,
    pub label: String,
    pub key: String,
    pub active: bool,
    pub reason: Option<String>,
}

/// Complete read-only snapshot of the application's UI state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UiSnapshot {
    pub screen: ScreenKind,
    pub mode: Option<String>,
    pub active_overlay: Option<OverlayKind>,
    pub overlay_rect: Option<RectSnapshot>,
    pub dock_mode: Option<DockSnapshot>,
    pub focus: Option<FocusSnapshot>,
    pub hover: Option<CellPos>,
    pub tooltip_visible: bool,
    pub status_line: Option<String>,
    pub last_error: Option<String>,
    pub game_stats: Option<GameStats>,
    pub legal_actions: Option<Vec<LegalAction>>,
}

struct ScreenData {
    mode: Option<String>,
    focus: Option<FocusSnapshot>,
    tooltip_visible: bool,
    status_line: Option<String>,
    last_error: Option<String>,
}

impl App {
    /// Read-only snapshot of the current UI state. Never mutates application state.
    pub fn snapshot(&self) -> UiSnapshot {
        let area = self.screen_rect();
        let overlay_rect = self
            .modal
            .as_ref()
            .and_then(|m| {
                overlay::rect_for(
                    area,
                    m,
                    &self.board,
                    self.now,
                    self.settings.help_dock,
                    self.settings.interface_size,
                )
            })
            .map(RectSnapshot::from);
        let data = self.screen_snapshot();
        let focus = if self.modal_takes_keys() {
            self.modal.as_ref().map(modal_focus)
        } else {
            data.focus
        };
        let last_error = data.last_error.or_else(|| match &self.modal {
            Some(Modal::Alert { message }) => Some(t(message).into_owned()),
            _ => None,
        });
        UiSnapshot {
            screen: ScreenKind::from(self.screen),
            mode: data.mode,
            active_overlay: self.modal.as_ref().map(OverlayKind::of),
            overlay_rect,
            dock_mode: self.dock_snapshot(),
            focus,
            hover: self.pointer.map(|(x, y)| CellPos { x, y }),
            tooltip_visible: self.modal.is_none() && data.tooltip_visible,
            status_line: data.status_line,
            last_error,
            game_stats: self.game_stats_snapshot(),
            legal_actions: self.legal_actions_snapshot(),
        }
    }

    fn dock_snapshot(&self) -> Option<DockSnapshot> {
        let modal = self.modal.as_ref()?;
        if !overlay::dock::dock_view::is_dockable(modal) {
            return None;
        }
        let docked = self.dock_split().is_some();
        Some(DockSnapshot {
            side: self.settings.help_dock,
            docked,
            focused: docked && self.dock.focused,
        })
    }

    fn game_stats_snapshot(&self) -> Option<GameStats> {
        let session = (self.screen == Screen::Board)
            .then(|| self.board.session())
            .flatten()?;
        let elapsed_ms =
            u64::try_from(self.board.elapsed(self.now).as_millis()).unwrap_or(u64::MAX);
        Some(GameStats {
            variant: session.variant(),
            seed: session.seed(),
            moves: session.moves_made(),
            elapsed_ms,
            undo_allowed: self.board.undo_allowed(),
            restarts: self.board.restarts(),
            won: self.board.is_won(),
        })
    }

    fn legal_actions_snapshot(&self) -> Option<Vec<LegalAction>> {
        (self.screen == Screen::Board).then(|| {
            self.board
                .move_entries()
                .into_iter()
                .map(|e| LegalAction {
                    action: action_name(e.action).to_owned(),
                    label: e.label,
                    key: e.key,
                    active: e.active,
                    reason: e.reason,
                })
                .collect()
        })
    }

    fn screen_snapshot(&self) -> ScreenData {
        match self.screen {
            Screen::MainMenu => self.main_menu_snapshot(),
            Screen::GameSelect => {
                let mode = if self.select.searching() {
                    "searching"
                } else {
                    "play"
                };
                game_select_data(
                    "game_select",
                    mode,
                    self.select.dev_state(&self.settings.favourites),
                )
            }
            Screen::Board => self.board_snapshot(),
            Screen::Settings => self.settings_snapshot(),
            Screen::Stats => self.stats_snapshot(),
        }
    }

    fn main_menu_snapshot(&self) -> ScreenData {
        let m = self
            .menu
            .dev_state(self.saved.is_some(), &self.settings.favourites);
        if let Some(gs) = m.game_select {
            let mode = if gs.searching { "searching" } else { m.mode };
            return game_select_data("main_menu", mode, gs);
        }
        ScreenData {
            mode: Some(m.mode.to_owned()),
            focus: Some(FocusSnapshot {
                area: "main_menu".to_owned(),
                target: m.focus_target.to_owned(),
                index: m.focus_index,
                label: m.label_key.map(|k| t(k).into_owned()),
            }),
            tooltip_visible: m.tooltip_visible,
            status_line: m.notice_key.map(|k| t(k).into_owned()),
            last_error: m.error_key.map(|k| t(k).into_owned()),
        }
    }

    fn board_snapshot(&self) -> ScreenData {
        let b = self.board.dev_state();
        let mode = if b.covered {
            "paused"
        } else if self.board.is_won() {
            "won"
        } else if b.finishing {
            "finishing"
        } else if self.board.carrying() {
            "carrying"
        } else {
            "browsing"
        };
        let (status_line, last_error) = match b.event {
            Some((text, true)) => (b.now, Some(text)),
            Some((text, false)) => (Some(text), None),
            None => (b.now, None),
        };
        ScreenData {
            mode: Some(mode.to_owned()),
            focus: self.board_focus(b.panel_focus),
            tooltip_visible: false,
            status_line,
            last_error,
        }
    }

    fn board_focus(&self, panel_focus: Option<usize>) -> Option<FocusSnapshot> {
        if let Some(index) = panel_focus {
            let entries = self.board.move_entries();
            let entry = entries.get(index);
            return Some(FocusSnapshot {
                area: "board_moves_panel".to_owned(),
                target: entry
                    .map(|e| action_name(e.action).to_owned())
                    .unwrap_or_else(|| "entry".to_owned()),
                index: Some(index),
                label: entry.map(|e| e.label.clone()),
            });
        }
        let f = self.board.marks().focus?;
        Some(FocusSnapshot {
            area: "board".to_owned(),
            target: pile_name_id(f.pile),
            index: Some(f.index),
            label: Some(crate::announce::pile_name(f.pile)),
        })
    }

    fn settings_snapshot(&self) -> ScreenData {
        let (tab, index, field, previewing) = self.settings_screen.dev_state();
        let mode = match tab {
            Tab::AudioVisual => "audio_visual",
            Tab::Popular => "popular",
            Tab::Gameplay => "gameplay",
        };
        ScreenData {
            mode: Some(mode.to_owned()),
            focus: field.map(|f| FocusSnapshot {
                area: "settings".to_owned(),
                target: f.label_key().to_owned(),
                index: Some(index),
                label: Some(t(f.label_key()).into_owned()),
            }),
            tooltip_visible: false,
            status_line: previewing.then(|| t("settings.preview.note").into_owned()),
            last_error: None,
        }
    }

    fn stats_snapshot(&self) -> ScreenData {
        let s = self.stats.dev_state();
        ScreenData {
            mode: Some("history".to_owned()),
            focus: Some(FocusSnapshot {
                area: "stats".to_owned(),
                target: s.focus_target.to_owned(),
                index: s.focus_index,
                label: None,
            }),
            tooltip_visible: false,
            status_line: s.status_line,
            last_error: s.last_error,
        }
    }
}

fn game_select_data(area: &str, mode: &str, state: DevGameSelect) -> ScreenData {
    ScreenData {
        mode: Some(mode.to_owned()),
        focus: Some(FocusSnapshot {
            area: area.to_owned(),
            target: state.focus_target.to_owned(),
            index: state.focus_index,
            label: state.variant.map(|v| t(variant_label_key(v)).into_owned()),
        }),
        tooltip_visible: false,
        status_line: None,
        last_error: None,
    }
}

fn modal_focus(modal: &Modal) -> FocusSnapshot {
    let (target, index) = overlay::dev_focus(modal);
    FocusSnapshot {
        area: "overlay".to_owned(),
        target: target.to_owned(),
        index,
        label: None,
    }
}

fn pile_name_id(pile: PileRef) -> String {
    match pile {
        PileRef::Stock => "stock".to_owned(),
        PileRef::Waste => "waste".to_owned(),
        PileRef::Foundation(i) => format!("foundation_{i}"),
        PileRef::Tableau(i) => format!("tableau_{i}"),
        PileRef::Cell(i) => format!("cell_{i}"),
        PileRef::Pyramid { row, index } => format!("pyramid_{row}_{index}"),
    }
}

fn action_name(action: Action) -> &'static str {
    action
        .label_key()
        .strip_prefix("help.action.")
        .unwrap_or_default()
}
