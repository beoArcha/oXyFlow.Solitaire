//! Doubles and helpers shared by the tests of this crate: shared-state storage and audio,
//! and a harness that drives `App` on a `TestBackend` terminal.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use oxysol_core::Variant;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{
    KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};

use crate::app::{App, Deps};
use crate::audio::{Audio, AudioSettings, AudioStatus, SoundCue, StyleInfo};
use crate::caps::{ColorDepth, TerminalCaps};
use crate::input::Input;
use crate::settings::Settings;
use crate::storage::memory::MemoryStorage;
use crate::storage::{GameId, Outcome, SavedGame, Storage, StorageError};

pub const TRUE_COLOR: TerminalCaps = TerminalCaps {
    color: ColorDepth::TrueColor,
    unicode: true,
    remote: false,
};

pub const PLAIN: TerminalCaps = TerminalCaps {
    color: ColorDepth::Ansi16,
    unicode: false,
    remote: false,
};

pub struct SharedStorage(pub Rc<RefCell<MemoryStorage>>);

impl Storage for SharedStorage {
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

    fn start_game(&mut self, variant: Variant, seed: u64) -> Result<GameId, StorageError> {
        self.0.borrow_mut().start_game(variant, seed)
    }

    fn finish_game(
        &mut self,
        id: GameId,
        outcome: Outcome,
        elapsed_ms: u64,
    ) -> Result<(), StorageError> {
        self.0.borrow_mut().finish_game(id, outcome, elapsed_ms)
    }
}

#[derive(Debug, Default)]
pub struct AudioLog {
    pub cues: Vec<SoundCue>,
    pub configured: Vec<AudioSettings>,
}

pub struct RecordingAudio(pub Rc<RefCell<AudioLog>>);

impl Audio for RecordingAudio {
    fn status(&self) -> AudioStatus {
        AudioStatus::Ready
    }

    fn configure(&mut self, settings: &AudioSettings) {
        self.0.borrow_mut().configured.push(settings.clone());
    }

    fn play(&mut self, cue: SoundCue) {
        self.0.borrow_mut().cues.push(cue);
    }
}

/// The sound styles of the application in tests: the two embedded ones, in their order.
pub fn sound_styles() -> Vec<StyleInfo> {
    [("default", "Default"), ("retro", "Retro")]
        .into_iter()
        .map(|(id, name)| StyleInfo {
            id: id.to_owned(),
            name: name.to_owned(),
        })
        .collect()
}

pub struct Harness {
    pub app: App,
    pub terminal: Terminal<TestBackend>,
    pub storage: Rc<RefCell<MemoryStorage>>,
    pub audio: Rc<RefCell<AudioLog>>,
    pub now: Instant,
}

impl Harness {
    pub fn new() -> Self {
        Self::with(MemoryStorage::new(), TRUE_COLOR, 120, 40)
    }

    /// A harness whose application was started with the command-line seed `seed`.
    pub fn starting_at_seed(seed: u64) -> Self {
        Self::build(MemoryStorage::new(), TRUE_COLOR, 120, 40, Some(seed))
    }

    pub fn with(storage: MemoryStorage, caps: TerminalCaps, width: u16, height: u16) -> Self {
        Self::build(storage, caps, width, height, None)
    }

    fn build(
        storage: MemoryStorage,
        caps: TerminalCaps,
        width: u16,
        height: u16,
        start_seed: Option<u64>,
    ) -> Self {
        let storage = Rc::new(RefCell::new(storage));
        let audio = Rc::new(RefCell::new(AudioLog::default()));
        let mut next_seed = 41_u64;
        let app = App::new(Deps {
            storage: Box::new(SharedStorage(Rc::clone(&storage))),
            audio: Box::new(RecordingAudio(Rc::clone(&audio))),
            caps,
            seeds: Box::new(move || {
                next_seed += 1;
                next_seed
            }),
            start_seed,
            sound_styles: sound_styles(),
        });
        Self {
            app,
            terminal: Terminal::new(TestBackend::new(width, height))
                .unwrap_or_else(|never| match never {}),
            storage,
            audio,
            now: Instant::now(),
        }
    }

    pub fn send(&mut self, input: Input) {
        self.app.handle(input, self.now);
    }

    pub fn press(&mut self, code: KeyCode) {
        self.send(Input::Key(KeyEvent::new(code, KeyModifiers::NONE)));
    }

    pub fn type_char(&mut self, c: char) {
        self.press(KeyCode::Char(c));
    }

    pub fn click(&mut self, column: u16, row: u16) {
        self.send(Input::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column,
            row,
            modifiers: KeyModifiers::NONE,
        }));
    }

    pub fn advance(&mut self, seconds: u64) {
        self.now += Duration::from_secs(seconds);
        self.send(Input::Tick);
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.terminal.backend_mut().resize(width, height);
        self.send(Input::Resize(width, height));
    }

    pub fn lines(&mut self) -> Vec<String> {
        let Harness { app, terminal, .. } = self;
        terminal
            .draw(|frame| app.draw(frame))
            .unwrap_or_else(|never| match never {});
        let buffer = terminal.backend().buffer();
        let area = buffer.area;
        (0..area.height)
            .map(|y| (0..area.width).map(|x| buffer[(x, y)].symbol()).collect())
            .collect()
    }

    pub fn text(&mut self) -> String {
        self.lines().join("\n")
    }

    pub fn find(&mut self, needle: &str) -> Option<(u16, u16)> {
        let lines = self.lines();
        lines.iter().enumerate().find_map(|(row, line)| {
            let byte = line.find(needle)?;
            let column = line[..byte].chars().count();
            Some((u16::try_from(column).ok()?, u16::try_from(row).ok()?))
        })
    }
}
