//! Doubles and helpers shared by the tests of this crate: shared-state storage and audio,
//! and a harness that drives `App` on a `TestBackend` terminal.

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::time::{Duration, Instant};

use oxysol_core::Variant;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{
    KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::layout::{Position, Rect};

use crate::app::{App, Deps, DisplayDeps, Zone};
use crate::audio::{Audio, AudioSettings, AudioStatus, SoundCue, StyleInfo};
use crate::caps::{ColorDepth, TerminalCaps};
use crate::clipboard::{Clipboard, ClipboardError};
use crate::input::Input;
use crate::settings::{CardBack, Settings};
use crate::solver::{Answer, Proof, Query, Solver};
use crate::storage::memory::MemoryStorage;
use crate::storage::{
    AssistCounters, GameId, GameRecord, Outcome, SavedGame, Storage, StorageError,
};
use crate::theme::{Theme, Token, UserTheme};

/// The version string the test harness gives the menu.
pub const TEST_VERSION: &str = "1.2.3-test";

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

#[derive(Debug, Default)]
pub struct ClipboardLog {
    pub copies: Vec<String>,
    pub failing: bool,
}

pub struct RecordingClipboard(pub Rc<RefCell<ClipboardLog>>);

impl Clipboard for RecordingClipboard {
    fn copy(&mut self, text: &str) -> Result<(), ClipboardError> {
        let mut log = self.0.borrow_mut();
        if log.failing {
            return Err(ClipboardError::Unavailable);
        }
        log.copies.push(text.to_owned());
        Ok(())
    }
}

/// What the stub solver is told to do and what it was asked.
#[derive(Debug, Default)]
pub struct StubState {
    /// Every query ever submitted, in order.
    pub submitted: Vec<Query>,
    pending: VecDeque<Query>,
    /// The n-th submitted query (counting from 1) is proven winnable.
    pub accept_nth: Option<usize>,
    /// Verdicts by seed, which win over `accept_nth`.
    pub verdicts: HashMap<u64, Proof>,
    /// Without it, every poll answers all pending queries; with it, only the released ones.
    pub held: bool,
    released: usize,
    pub cancels: u32,
}

impl StubState {
    /// Lets the next `count` pending queries be answered by the next poll (with `held`).
    pub fn release(&mut self, count: usize) {
        self.released += count;
    }

    pub fn pending(&self) -> usize {
        self.pending.len()
    }

    fn verdict(&self, query: Query) -> Proof {
        if let Some(proof) = self.verdicts.get(&query.seed) {
            return *proof;
        }
        let number = self
            .submitted
            .iter()
            .position(|sent| sent.id == query.id)
            .map(|index| index + 1);
        if number.is_some() && number == self.accept_nth {
            Proof::Winnable
        } else {
            Proof::Unknown
        }
    }
}

pub struct StubSolver(pub Rc<RefCell<StubState>>);

impl Solver for StubSolver {
    fn submit(&mut self, query: Query) {
        let mut state = self.0.borrow_mut();
        state.submitted.push(query);
        state.pending.push_back(query);
    }

    fn cancel(&mut self) {
        let mut state = self.0.borrow_mut();
        state.pending.clear();
        state.cancels += 1;
    }

    fn poll(&mut self) -> Vec<Answer> {
        let mut state = self.0.borrow_mut();
        let count = if state.held {
            let count = state.released.min(state.pending.len());
            state.released -= count;
            count
        } else {
            state.pending.len()
        };
        let queries: Vec<Query> = state.pending.drain(..count).collect();
        queries
            .into_iter()
            .map(|query| Answer {
                id: query.id,
                proof: state.verdict(query),
            })
            .collect()
    }
}

pub struct Harness {
    pub app: App,
    pub terminal: Terminal<TestBackend>,
    pub caps: TerminalCaps,
    pub storage: Rc<RefCell<MemoryStorage>>,
    pub audio: Rc<RefCell<AudioLog>>,
    pub clipboard: Rc<RefCell<ClipboardLog>>,
    pub solver: Rc<RefCell<StubState>>,
    pub now: Instant,
}

impl Harness {
    pub fn new() -> Self {
        Self::with(MemoryStorage::new(), TRUE_COLOR, 120, 40)
    }

    /// A harness whose application was started with the command-line seed `seed`.
    pub fn starting_at_seed(seed: u64) -> Self {
        Self::build(
            MemoryStorage::new(),
            TRUE_COLOR,
            120,
            40,
            Some(seed),
            None,
            Zone::Utc,
            UserTheme::Missing,
        )
    }

    /// A harness whose application was given `display` for the full-screen switch.
    pub fn with_display(storage: MemoryStorage, display: DisplayDeps) -> Self {
        Self::build(
            storage,
            TRUE_COLOR,
            120,
            40,
            None,
            Some(display),
            Zone::Utc,
            UserTheme::Missing,
        )
    }

    /// A harness with a loaded (or failed) user theme.
    pub fn with_user_theme(storage: MemoryStorage, user_theme: UserTheme) -> Self {
        Self::build(
            storage,
            TRUE_COLOR,
            120,
            40,
            None,
            None,
            Zone::Utc,
            user_theme,
        )
    }

    pub fn with(storage: MemoryStorage, caps: TerminalCaps, width: u16, height: u16) -> Self {
        Self::build(
            storage,
            caps,
            width,
            height,
            None,
            None,
            Zone::Utc,
            UserTheme::Missing,
        )
    }

    /// Like [`Harness::with`], with the statistics history shown in `zone`.
    pub fn with_zone(
        storage: MemoryStorage,
        caps: TerminalCaps,
        width: u16,
        height: u16,
        zone: Zone,
    ) -> Self {
        Self::build(
            storage,
            caps,
            width,
            height,
            None,
            None,
            zone,
            UserTheme::Missing,
        )
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "test harness builder keeps explicit fixtures"
    )]
    pub(crate) fn build(
        storage: MemoryStorage,
        caps: TerminalCaps,
        width: u16,
        height: u16,
        start_seed: Option<u64>,
        display: Option<DisplayDeps>,
        zone: Zone,
        user_theme: UserTheme,
    ) -> Self {
        let storage = Rc::new(RefCell::new(storage));
        let audio = Rc::new(RefCell::new(AudioLog::default()));
        let clipboard = Rc::new(RefCell::new(ClipboardLog::default()));
        let solver = Rc::new(RefCell::new(StubState::default()));
        let mut next_seed = 41_u64;
        let app = App::new(Deps {
            storage: Box::new(SharedStorage(Rc::clone(&storage))),
            audio: Box::new(RecordingAudio(Rc::clone(&audio))),
            clipboard: Box::new(RecordingClipboard(Rc::clone(&clipboard))),
            caps,
            seeds: Box::new(move || {
                next_seed += 1;
                next_seed
            }),
            start_seed,
            sound_styles: sound_styles(),
            user_theme,
            solver: Box::new(StubSolver(Rc::clone(&solver))),
            display: display.unwrap_or_else(DisplayDeps::none),
            version: TEST_VERSION,
            zone,
        });
        Self {
            app,
            terminal: Terminal::new(TestBackend::new(width, height))
                .unwrap_or_else(|never| match never {}),
            caps,
            storage,
            audio,
            clipboard,
            solver,
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

    fn draw_and_theme(&mut self) -> Theme {
        let Harness {
            app,
            terminal,
            caps,
            ..
        } = self;
        terminal
            .draw(|frame| app.draw(frame))
            .unwrap_or_else(|never| match never {});
        let s = app.settings();
        Theme::for_settings(s.theme, app.user_theme_palette(), s.color_depth(caps))
            .with_color_vision(s.color_vision)
            .with_low_vision(s.low_vision)
    }

    /// The lines of the screen with the preview pane of the settings screen blanked, so a test that
    /// looks for a focus marker or an arrow meets the list only.
    pub fn lines_without_pane(&mut self) -> Vec<String> {
        let theme = self.draw_and_theme();
        lines_without_pane(self.terminal.backend().buffer(), &theme)
    }

    /// The look of the preview pane of the settings screen, cell by cell.
    pub fn pane_cells(&mut self) -> Vec<String> {
        let theme = self.draw_and_theme();
        pane_cells(self.terminal.backend().buffer(), &theme)
    }

    pub fn find(&mut self, needle: &str) -> Option<(u16, u16)> {
        let lines = self.lines();
        lines.iter().enumerate().find_map(|(row, line)| {
            let byte = line.find(needle)?;
            let column = line[..byte].chars().count();
            Some((u16::try_from(column).ok()?, u16::try_from(row).ok()?))
        })
    }

    /// Steps `Down` in the game selection list until `variant` has the focus.
    pub fn focus_game_variant(&mut self, variant: Variant) {
        let label = crate::i18n::t(crate::screens::variant_label_key(variant)).into_owned();
        let max_steps = Variant::ALL.len() + 16;
        for _ in 0..max_steps {
            if self
                .lines()
                .iter()
                .any(|l| is_focused_variant_line(l, &label))
            {
                return;
            }
            self.press(KeyCode::Down);
        }
        panic!("variant {label:?} was not focused within {max_steps} steps");
    }
}

fn is_focused_variant_line(line: &str, label: &str) -> bool {
    let Some(at) = line.find('►').or_else(|| line.find('>')) else {
        return false;
    };
    let rest = line[at..].trim_start_matches(['►', '>', ' ']);
    !rest.starts_with('[')
        && rest
            .splitn(3, ' ')
            .nth(2)
            .map(str::trim)
            .is_some_and(|n| n == label || n.starts_with(&format!("{label}  ")))
}

fn find_pane_title(buffer: &Buffer) -> Option<(u16, u16, u16)> {
    let title = crate::i18n::t("settings.pane.title");
    let width = u16::try_from(title.chars().count()).ok()?;
    let area = buffer.area;
    (area.top()..area.bottom()).find_map(|y| {
        let row: String = (area.left()..area.right())
            .map(|x| buffer[(x, y)].symbol())
            .collect();
        let byte = row.find(&*title)?;
        let col = area.left() + u16::try_from(row[..byte].chars().count()).ok()?;
        Some((col, y, width))
    })
}

fn same_bg_pane_bounds(buffer: &Buffer, theme: &Theme) -> Option<Rect> {
    let (title_x, title_y, title_w) = find_pane_title(buffer)?;
    let back_bgs = [CardBack::Theme, CardBack::Monokai, CardBack::Dracula].map(|back| {
        theme
            .card_back_style(back, (Token::CardBackMark, Token::CardBack))
            .bg
    });
    let area = buffer.area;
    let back = ((title_y + 1)..area.bottom())
        .flat_map(|y| (area.left()..area.right()).map(move |x| (x, y)))
        .filter(|(x, y)| back_bgs.contains(&Some(buffer[(*x, *y)].bg)))
        .fold(None, |bounds: Option<Rect>, (x, y)| {
            let cell = Rect::new(x, y, 1, 1);
            Some(bounds.map_or(cell, |bounds| bounds.union(cell)))
        })?;
    let rings = back.top().saturating_sub(title_y + 1);
    let left = back.left().saturating_sub(rings + 1).min(title_x);
    let right = (2 * title_x + title_w)
        .saturating_sub(left)
        .min(area.right())
        .max(title_x + title_w);
    let bottom = (back.bottom() + rings + 1).min(area.bottom());
    Some(Rect::new(
        left,
        title_y,
        right.saturating_sub(left),
        bottom.saturating_sub(title_y),
    ))
}

/// The bounding box of the cells on the table colour of `theme`: the preview pane of the settings
/// screen, the only part of that screen on the table colour. `None` when no cell is.
pub fn pane_bounds(buffer: &Buffer, theme: &Theme) -> Option<Rect> {
    let table = theme.style(Token::TableText, Token::Table).bg;
    let background = theme.style(Token::Text, Token::Background).bg;
    if table == background {
        return same_bg_pane_bounds(buffer, theme);
    }
    let area = buffer.area;
    let cells = (0..area.height)
        .flat_map(|y| (0..area.width).map(move |x| (x, y)))
        .filter(|(x, y)| table == Some(buffer[(*x, *y)].bg));
    cells.fold(None, |bounds: Option<Rect>, (x, y)| {
        let cell = Rect::new(x, y, 1, 1);
        Some(bounds.map_or(cell, |bounds| bounds.union(cell)))
    })
}

/// The lines of `buffer` with the preview pane of the settings screen blanked.
pub fn lines_without_pane(buffer: &Buffer, theme: &Theme) -> Vec<String> {
    let pane = pane_bounds(buffer, theme).unwrap_or_default();
    let area = buffer.area;
    (0..area.height)
        .map(|y| {
            (0..area.width)
                .map(|x| {
                    if pane.contains(Position::new(x, y)) {
                        " "
                    } else {
                        buffer[(x, y)].symbol()
                    }
                })
                .collect()
        })
        .collect()
}

/// Every cell of the preview pane with its colours, row by row: two panes are equal when they look
/// the same.
pub fn pane_cells(buffer: &Buffer, theme: &Theme) -> Vec<String> {
    let pane = pane_bounds(buffer, theme).unwrap_or_default();
    (pane.top()..pane.bottom())
        .flat_map(|y| (pane.left()..pane.right()).map(move |x| (x, y)))
        .map(|(x, y)| {
            let cell = &buffer[(x, y)];
            format!("{}|{:?}|{:?}", cell.symbol(), cell.fg, cell.bg)
        })
        .collect()
}
