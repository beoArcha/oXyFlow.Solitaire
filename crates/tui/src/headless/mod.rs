//! Headless driver of the real `App` over `TestBackend` and [`VirtualClock`] (decision T-42).
//! Single-threaded and deterministic: in-memory storage, silent audio, recording clipboard,
//! sequential seed counter and synchronous solver.

#[cfg(test)]
mod snapshot_tests;
#[cfg(test)]
mod tests;

use std::cell::{Ref, RefCell};
use std::io;
use std::rc::Rc;
use std::time::Instant;

use oxysol_core::Variant;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;

use crate::app::snapshot::UiSnapshot;
use crate::app::{App, Deps, DisplayDeps, Zone};
use crate::audio::{AudioStatus, NullAudio, StyleInfo};
use crate::caps::{ColorDepth, TerminalCaps};
use crate::clipboard::{Clipboard, ClipboardError};
use crate::clock::{Clock, VirtualClock};
use crate::input::Input;
use crate::settings::Settings;
use crate::solver::{Proof, SyncSolver};
use crate::storage::memory::MemoryStorage;
use crate::storage::{
    AssistCounters, GameId, GameRecord, Outcome, SavedGame, Storage, StorageError,
};
use crate::terminal;
use crate::theme::UserTheme;

/// Configuration for building a [`Headless`] session.
#[derive(Debug, Clone)]
pub struct HeadlessConfig {
    pub cols: u16,
    pub rows: u16,
    pub caps: TerminalCaps,
    pub settings: Option<Settings>,
    pub start_seed: Option<u64>,
    pub seed_start: u64,
    pub sound_styles: Vec<StyleInfo>,
    pub version: &'static str,
    pub zone: Zone,
    pub base_time: Option<Instant>,
}

impl Default for HeadlessConfig {
    fn default() -> Self {
        Self {
            cols: 120,
            rows: 40,
            caps: TerminalCaps {
                color: ColorDepth::TrueColor,
                unicode: true,
                remote: false,
            },
            settings: None,
            start_seed: None,
            seed_start: 1,
            sound_styles: default_sound_styles(),
            version: env!("CARGO_PKG_VERSION"),
            zone: Zone::Utc,
            base_time: None,
        }
    }
}

fn default_sound_styles() -> Vec<StyleInfo> {
    [("default", "Default"), ("retro", "Retro")]
        .into_iter()
        .map(|(id, name)| StyleInfo {
            id: id.to_owned(),
            name: name.to_owned(),
        })
        .collect()
}

struct SharedMemoryStorage(Rc<RefCell<MemoryStorage>>);

impl Storage for SharedMemoryStorage {
    fn load_settings(&self) -> Result<Option<Settings>, StorageError> {
        self.0.borrow().load_settings()
    }

    fn save_settings(&mut self, settings: &Settings) -> Result<(), StorageError> {
        self.0.borrow_mut().save_settings(settings)
    }

    fn load_game(&self) -> Result<Option<SavedGame>, StorageError> {
        self.0.borrow().load_game()
    }

    fn save_game(&mut self, game: &SavedGame) -> Result<(), StorageError> {
        self.0.borrow_mut().save_game(game)
    }

    fn clear_game(&mut self) -> Result<(), StorageError> {
        self.0.borrow_mut().clear_game()
    }

    fn start_game(
        &mut self,
        variant: Variant,
        seed: u64,
        options: oxysol_core::GameOptions,
        undo_allowed: bool,
    ) -> Result<GameId, StorageError> {
        self.0
            .borrow_mut()
            .start_game(variant, seed, options, undo_allowed)
    }

    fn finish_game(
        &mut self,
        id: GameId,
        outcome: Outcome,
        elapsed_ms: u64,
        moves: u32,
    ) -> Result<(), StorageError> {
        self.0
            .borrow_mut()
            .finish_game(id, outcome, elapsed_ms, moves)
    }

    fn record_restart(&mut self, id: GameId) -> Result<(), StorageError> {
        self.0.borrow_mut().record_restart(id)
    }

    fn record_assist(&mut self, id: GameId, assist: AssistCounters) -> Result<(), StorageError> {
        self.0.borrow_mut().record_assist(id, assist)
    }

    fn record_proof(&mut self, id: GameId, proof: Proof) -> Result<(), StorageError> {
        self.0.borrow_mut().record_proof(id, proof)
    }

    fn history(&self) -> Result<Vec<GameRecord>, StorageError> {
        self.0.borrow().history()
    }
}

struct RecordingClipboard(Rc<RefCell<Vec<String>>>);

impl Clipboard for RecordingClipboard {
    fn copy(&mut self, text: &str) -> Result<(), ClipboardError> {
        self.0.borrow_mut().push(text.to_owned());
        Ok(())
    }
}

/// Deterministic, single-threaded driver of [`App`] over `TestBackend` and [`VirtualClock`].
pub struct Headless {
    app: App,
    terminal: Terminal<TestBackend>,
    clock: VirtualClock,
    frame_id: u64,
    storage: Rc<RefCell<MemoryStorage>>,
    clipboard: Rc<RefCell<Vec<String>>>,
}

impl Headless {
    /// Starts a headless session with `config`, drawing the initial frame (`frame_id = 0`).
    pub fn new(config: HeadlessConfig) -> io::Result<Self> {
        let mut memory = MemoryStorage::new();
        memory.settings = config.settings;
        let storage = Rc::new(RefCell::new(memory));
        let clipboard = Rc::new(RefCell::new(Vec::new()));
        let mut next_seed = config.seed_start;
        let clock = config
            .base_time
            .map_or_else(VirtualClock::new, VirtualClock::from_instant);
        let mut app = App::new(Deps {
            storage: Box::new(SharedMemoryStorage(Rc::clone(&storage))),
            audio: Box::new(NullAudio::new(AudioStatus::NotBuilt)),
            clipboard: Box::new(RecordingClipboard(Rc::clone(&clipboard))),
            caps: config.caps,
            seeds: Box::new(move || {
                let seed = next_seed;
                next_seed = next_seed.wrapping_add(1);
                seed
            }),
            start_seed: config.start_seed,
            sound_styles: config.sound_styles,
            user_theme: UserTheme::Missing,
            solver: Box::new(SyncSolver::default()),
            display: DisplayDeps::none(),
            version: config.version,
            zone: config.zone,
        });
        let mut terminal = Terminal::new(TestBackend::new(config.cols, config.rows))
            .unwrap_or_else(|never| match never {});
        app.handle(Input::Tick, clock.now());
        terminal
            .draw(|frame| app.draw(frame))
            .unwrap_or_else(|never| match never {});
        Ok(Self {
            app,
            terminal,
            clock,
            frame_id: 0,
            storage,
            clipboard,
        })
    }

    /// Starts `variant` at `seed` (or the next counter seed) and redraws frame `frame_id`.
    pub fn start_game(&mut self, variant: Variant, seed: Option<u64>, undo_allowed: bool) {
        self.app.handle(Input::Tick, self.clock.now());
        self.app.apply(crate::screens::Effect::Start {
            variant,
            seed,
            undo_allowed,
        });
        let Self { app, terminal, .. } = self;
        terminal
            .draw(|frame| app.draw(frame))
            .unwrap_or_else(|never| match never {});
    }

    /// Runs one synchronous step via the shared terminal step function and increments `frame_id`.
    pub fn step(&mut self, input: Input) -> io::Result<()> {
        let Self {
            app,
            terminal,
            clock,
            frame_id,
            ..
        } = self;
        terminal::step(app, clock, input, &mut |app| {
            terminal
                .draw(|frame| app.draw(frame))
                .map(|_| ())
                .unwrap_or_else(|never| match never {});
            Ok(())
        })?;
        *frame_id += 1;
        Ok(())
    }

    /// Resizes the backing `TestBackend` and steps with [`Input::Resize`].
    pub fn resize(&mut self, cols: u16, rows: u16) -> io::Result<()> {
        self.terminal.backend_mut().resize(cols, rows);
        self.step(Input::Resize(cols, rows))
    }

    /// Advances the virtual clock by `ms` milliseconds and steps with [`Input::Tick`].
    pub fn tick(&mut self, ms: u64) -> io::Result<()> {
        self.clock.advance_ms(ms);
        self.step(Input::Tick)
    }

    /// Current frame counter (0 after construction, +1 per [`Headless::step`]).
    pub fn frame_id(&self) -> u64 {
        self.frame_id
    }

    /// Ratatui cell buffer of the last rendered frame.
    pub fn buffer(&self) -> &Buffer {
        self.terminal.backend().buffer()
    }

    /// Plain text lines of the `cols × rows` grid.
    pub fn lines(&self) -> Vec<String> {
        let buffer = self.buffer();
        let area = buffer.area;
        (0..area.height)
            .map(|y| (0..area.width).map(|x| buffer[(x, y)].symbol()).collect())
            .collect()
    }

    /// Plain text `cols × rows` grid joined with newlines.
    pub fn text_frame(&self) -> String {
        self.lines().join("\n")
    }

    /// Read-only snapshot of the current UI state.
    pub fn snapshot(&self) -> UiSnapshot {
        self.app.snapshot()
    }

    /// The virtual clock driving this session.
    pub fn clock(&self) -> &VirtualClock {
        &self.clock
    }

    /// The underlying application.
    pub fn app(&self) -> &App {
        &self.app
    }

    /// Mutable access to the underlying application.
    pub fn app_mut(&mut self) -> &mut App {
        &mut self.app
    }

    /// Whether the application requested exit.
    pub fn should_quit(&self) -> bool {
        self.app.should_quit()
    }

    /// Texts written to the recording clipboard, in order.
    pub fn clipboard_copies(&self) -> Vec<String> {
        self.clipboard.borrow().clone()
    }

    /// Shared in-memory storage backing this session.
    pub fn storage(&self) -> Ref<'_, MemoryStorage> {
        self.storage.borrow()
    }
}
