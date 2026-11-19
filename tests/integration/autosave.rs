//! Autosave, resume and history: `App` over a real `SqliteStorage` in a temporary directory.
//! A scripted win is covered at unit level (`oxysol-tui`, `screens::board::tests::persist`); a
//! full game cannot be driven from outside, because the cursor is internal to the board.

use std::time::{Duration, Instant};

use oxysol::paths::AppPaths;
use oxysol::storage::SqliteStorage;
use oxysol_core::{DrawCount, PileSnapshot, Session, Variant};
use oxysol_tui::caps::{ColorDepth, TerminalCaps};
use oxysol_tui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use oxysol_tui::{
    App, AudioStatus, Deps, GameId, GameRecord, Input, Modal, NullAudio, NullClipboard, NullSolver,
    Outcome, Proof, SavedGame, Screen, Settings, Storage, StorageError,
};
use rusqlite::Connection;
use tempfile::TempDir;

const KLONDIKE: Variant = Variant::Klondike(DrawCount::One);
const SEED: u64 = 77;
const CAPS: TerminalCaps = TerminalCaps {
    color: ColorDepth::TrueColor,
    unicode: true,
    remote: false,
};

struct Player {
    app: App,
    now: Instant,
}

fn temp_paths() -> (TempDir, AppPaths) {
    let Ok(dir) = tempfile::tempdir() else {
        std::process::abort();
    };
    let paths = AppPaths::in_dir(dir.path());
    (dir, paths)
}

fn open(paths: &AppPaths) -> SqliteStorage {
    let Ok(storage) = SqliteStorage::open(paths) else {
        std::process::abort();
    };
    storage
}

fn connect(paths: &AppPaths) -> Connection {
    let Ok(conn) = Connection::open(paths.database_file()) else {
        std::process::abort();
    };
    conn
}

fn launch(storage: Box<dyn Storage>) -> Player {
    let app = App::new(Deps {
        storage,
        audio: Box::new(NullAudio::new(AudioStatus::NotBuilt)),
        clipboard: Box::new(NullClipboard),
        caps: CAPS,
        seeds: Box::new(|| SEED),
        start_seed: None,
        sound_styles: Vec::new(),
        solver: Box::new(NullSolver::default()),
    });
    Player {
        app,
        now: Instant::now(),
    }
}

impl Player {
    fn press(&mut self, code: KeyCode) {
        let key = KeyEvent::new(code, KeyModifiers::NONE);
        self.app.handle(Input::Key(key), self.now);
    }

    fn type_char(&mut self, c: char) {
        self.press(KeyCode::Char(c));
    }

    fn wait(&mut self, seconds: u64) {
        self.now += Duration::from_secs(seconds);
        self.app.handle(Input::Tick, self.now);
    }

    fn deal_first_game(&mut self) {
        self.press(KeyCode::Enter);
        self.press(KeyCode::Enter);
    }

    fn piles(&self) -> Vec<PileSnapshot> {
        self.app
            .board()
            .session()
            .map(|session| session.game().piles())
            .unwrap_or_default()
    }

    fn position(&self) -> Option<usize> {
        self.app.board().session().map(|s| s.record().cursor)
    }

    fn has_alert(&self) -> bool {
        matches!(self.app.modal(), Some(Modal::Alert { .. }))
    }
}

fn play_some_moves(player: &mut Player) {
    player.deal_first_game();
    player.type_char('s');
    player.wait(30);
    player.type_char('s');
    player.type_char('s');
    player.type_char('u');
    player.wait(12);
}

#[test]
fn resume_in_a_new_app_restores_piles_history_seed_and_time() {
    let (_dir, paths) = temp_paths();
    let mut first = launch(Box::new(open(&paths)));
    play_some_moves(&mut first);
    let piles = first.piles();
    assert_eq!(first.position(), Some(2));
    let played = first.app.board().elapsed(first.now);
    assert_eq!(played, Duration::from_secs(42));
    first.press(KeyCode::Esc);
    drop(first);

    let mut second = launch(Box::new(open(&paths)));
    assert_eq!(second.app.screen(), Screen::MainMenu);
    assert!(second.app.modal().is_none());
    second.press(KeyCode::Enter);
    assert_eq!(second.app.screen(), Screen::Board);
    assert_eq!(second.piles(), piles);
    assert_eq!(second.position(), Some(2));
    assert!(
        second
            .app
            .board()
            .session()
            .is_some_and(|s| s.seed() == SEED && s.variant() == KLONDIKE)
    );
    assert_eq!(second.app.board().elapsed(second.now), played);

    second.type_char('r');
    assert_eq!(second.position(), Some(3));
    second.type_char('u');
    second.type_char('u');
    assert_eq!(second.position(), Some(1));
}

#[test]
fn quitting_keeps_the_save_and_leaves_the_history_row_open() {
    let (_dir, paths) = temp_paths();
    let mut player = launch(Box::new(open(&paths)));
    play_some_moves(&mut player);
    player.type_char('q');
    player.press(KeyCode::Left);
    player.press(KeyCode::Enter);
    assert!(player.app.should_quit());
    drop(player);

    let storage = open(&paths);
    assert!(matches!(storage.load_game(), Ok(Some(ref g)) if g.record.cursor == 2));
    assert!(matches!(
        storage.history(),
        Ok(ref r) if r.len() == 1 && r[0].seed == SEED && r[0].outcome.is_none()
    ));
}

#[test]
fn dealing_over_a_save_records_the_old_game_as_abandoned() {
    let (_dir, paths) = temp_paths();
    let mut player = launch(Box::new(open(&paths)));
    play_some_moves(&mut player);
    player.press(KeyCode::Esc);
    player.press(KeyCode::Down);
    player.press(KeyCode::Enter);
    player.press(KeyCode::Enter);
    assert_eq!(player.position(), Some(0));
    drop(player);

    let storage = open(&paths);
    let Ok(rows) = storage.history() else {
        std::process::abort();
    };
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].seed, SEED);
    assert_eq!(rows[0].outcome, Some(Outcome::Abandoned));
    assert_eq!(rows[0].elapsed_ms, Some(42_000));
    assert_eq!(rows[1].outcome, None);
    assert!(matches!(storage.load_game(), Ok(Some(ref g)) if g.record.cursor == 0));
}

#[test]
fn a_corrupt_save_alerts_is_removed_and_the_menu_stays() {
    let (_dir, paths) = temp_paths();
    let mut player = launch(Box::new(open(&paths)));
    play_some_moves(&mut player);
    drop(player);
    let conn = connect(&paths);
    let broken = conn.execute("UPDATE saved_game SET record = 'not json' WHERE id = 1", []);
    assert_eq!(broken.ok(), Some(1));
    drop(conn);

    let mut player = launch(Box::new(open(&paths)));
    assert_eq!(player.app.screen(), Screen::MainMenu);
    assert!(player.has_alert());
    player.press(KeyCode::Enter);
    assert!(player.app.modal().is_none());
    assert_eq!(player.app.screen(), Screen::MainMenu);
    drop(player);

    assert!(matches!(open(&paths).load_game(), Ok(None)));
}

#[test]
fn an_unreplayable_save_alerts_and_is_removed() {
    let (_dir, paths) = temp_paths();
    let mut player = launch(Box::new(open(&paths)));
    play_some_moves(&mut player);
    drop(player);
    let mut record = Session::new(KLONDIKE, SEED).record();
    record.cursor = 9;
    let Ok(json) = serde_json::to_string(&record) else {
        std::process::abort();
    };
    let conn = connect(&paths);
    let updated = conn.execute("UPDATE saved_game SET record = ?1 WHERE id = 1", [json]);
    assert_eq!(updated.ok(), Some(1));
    drop(conn);

    let mut player = launch(Box::new(open(&paths)));
    player.press(KeyCode::Enter);
    assert_eq!(player.app.screen(), Screen::MainMenu);
    assert!(player.has_alert());
    drop(player);

    let storage = open(&paths);
    assert!(matches!(storage.load_game(), Ok(None)));
    assert!(matches!(storage.history(), Ok(ref r) if r[0].outcome == Some(Outcome::Abandoned)));
}

struct FailingSaves(SqliteStorage);

impl Storage for FailingSaves {
    fn load_settings(&self) -> Result<Option<Settings>, StorageError> {
        self.0.load_settings()
    }

    fn save_settings(&mut self, settings: &Settings) -> Result<(), StorageError> {
        self.0.save_settings(settings)
    }

    fn load_game(&self) -> Result<Option<SavedGame>, StorageError> {
        self.0.load_game()
    }

    fn save_game(&mut self, _game: &SavedGame) -> Result<(), StorageError> {
        Err(StorageError::Backend {
            message: "disk is full".to_owned(),
        })
    }

    fn clear_game(&mut self) -> Result<(), StorageError> {
        self.0.clear_game()
    }

    fn start_game(
        &mut self,
        variant: Variant,
        seed: u64,
        undo_allowed: bool,
    ) -> Result<GameId, StorageError> {
        self.0.start_game(variant, seed, undo_allowed)
    }

    fn finish_game(
        &mut self,
        id: GameId,
        outcome: Outcome,
        elapsed_ms: u64,
        moves: u32,
    ) -> Result<(), StorageError> {
        self.0.finish_game(id, outcome, elapsed_ms, moves)
    }

    fn record_restart(&mut self, id: GameId) -> Result<(), StorageError> {
        self.0.record_restart(id)
    }

    fn record_proof(&mut self, id: GameId, proof: Proof) -> Result<(), StorageError> {
        self.0.record_proof(id, proof)
    }

    fn history(&self) -> Result<Vec<GameRecord>, StorageError> {
        self.0.history()
    }
}

#[test]
fn a_storage_that_fails_on_write_alerts_once_and_the_game_goes_on() {
    let (_dir, paths) = temp_paths();
    let mut player = launch(Box::new(FailingSaves(open(&paths))));
    player.deal_first_game();
    let mut alerts = usize::from(player.has_alert());
    if player.has_alert() {
        player.press(KeyCode::Enter);
    }
    for key in ['s', 's', 's', 'u', 's'] {
        player.type_char(key);
        if player.has_alert() {
            alerts += 1;
            player.press(KeyCode::Enter);
        }
    }
    assert_eq!(alerts, 1);
    assert_eq!(player.app.screen(), Screen::Board);
    assert_eq!(player.position(), Some(3));
    player.press(KeyCode::Esc);
    assert!(!player.has_alert());
}

#[test]
fn a_restart_without_undo_is_stored_at_once_and_survives_quit_and_resume() {
    let (_dir, paths) = temp_paths();
    let mut player = launch(Box::new(open(&paths)));
    player.press(KeyCode::Enter);
    player.type_char('u');
    player.press(KeyCode::Enter);
    player.type_char('s');
    player.type_char('R');
    player.press(KeyCode::Left);
    player.press(KeyCode::Enter);
    assert_eq!(player.app.board().restarts(), 1);
    let stored = open(&paths).history();
    assert!(matches!(stored, Ok(ref rows) if rows[0].counters.restarts == 1));
    player.type_char('s');
    drop(player);

    let mut second = launch(Box::new(open(&paths)));
    second.press(KeyCode::Enter);
    assert_eq!(second.app.board().restarts(), 1);
    assert!(!second.app.board().undo_allowed());
}
