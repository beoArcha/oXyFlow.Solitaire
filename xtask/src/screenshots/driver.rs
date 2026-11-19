//! Drives the real `App` on a Ratatui `TestBackend` terminal, with in-memory services.

use std::time::{Duration, Instant};

use oxysol_tui::audio::{AudioStatus, NullAudio, StyleInfo};
use oxysol_tui::caps::{ColorDepth, TerminalCaps};
use oxysol_tui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use oxysol_tui::storage::memory::MemoryStorage;
use oxysol_tui::theme::{Theme, Token};
use oxysol_tui::{App, Deps, Input, NullClipboard, NullSolver, SavedGame, Settings};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::style::Color;

use super::grid::{Grid, Rgb};

const CAPS: TerminalCaps = TerminalCaps {
    color: ColorDepth::TrueColor,
    unicode: true,
    remote: false,
};

pub struct Setup {
    pub settings: Settings,
    pub seed: u64,
    pub saved: Option<SavedGame>,
    pub width: u16,
    pub height: u16,
}

pub struct Driver {
    app: App,
    terminal: Terminal<TestBackend>,
    now: Instant,
}

impl Driver {
    pub fn new(setup: Setup) -> Self {
        let storage = MemoryStorage {
            settings: Some(setup.settings),
            game: setup.saved,
            ..MemoryStorage::default()
        };
        let seed = setup.seed;
        let app = App::new(Deps {
            storage: Box::new(storage),
            audio: Box::new(NullAudio::new(AudioStatus::Ready)),
            clipboard: Box::new(NullClipboard),
            caps: CAPS,
            seeds: Box::new(move || seed),
            start_seed: None,
            sound_styles: sound_styles(),
            solver: Box::new(NullSolver::default()),
        });
        Self {
            app,
            terminal: Terminal::new(TestBackend::new(setup.width, setup.height))
                .unwrap_or_else(|never| match never {}),
            now: Instant::now(),
        }
    }

    pub fn app(&self) -> &App {
        &self.app
    }

    pub fn key(&mut self, code: KeyCode) {
        let event = KeyEvent::new(code, KeyModifiers::NONE);
        self.app.handle(Input::Key(event), self.now);
    }

    pub fn keys(&mut self, codes: &[KeyCode]) {
        for code in codes {
            self.key(*code);
        }
    }

    pub fn text(&mut self, text: &str) {
        for c in text.chars() {
            self.key(KeyCode::Char(c));
        }
    }

    pub fn wait(&mut self, millis: u64) {
        self.now += Duration::from_millis(millis);
        self.app.handle(Input::Tick, self.now);
    }

    pub fn frame(&mut self) -> Grid {
        let Self { app, terminal, .. } = self;
        terminal
            .draw(|frame| app.draw(frame))
            .unwrap_or_else(|never| match never {});
        let theme = Theme::new(app.settings().theme, ColorDepth::TrueColor);
        let fg = rgb(theme.color(Token::Text));
        let bg = rgb(theme.color(Token::Background));
        Grid::from_buffer(terminal.backend().buffer(), fg, bg)
    }
}

fn rgb(color: Color) -> Rgb {
    match color {
        Color::Rgb(r, g, b) => Rgb(r, g, b),
        _ => Rgb(0, 0, 0),
    }
}

fn sound_styles() -> Vec<StyleInfo> {
    [("default", "Default"), ("retro", "Retro")]
        .into_iter()
        .map(|(id, name)| StyleInfo {
            id: id.to_owned(),
            name: name.to_owned(),
        })
        .collect()
}
